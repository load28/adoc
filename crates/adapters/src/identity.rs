use std::{collections::HashSet, sync::Arc, time::Duration};

use adoc_application::identity::{
    Clock, IdentityError, OidcAuthorizationInput, OidcExchangeInput, OidcProvider, SecureRandom,
    TokenHash, VerifiedExternalIdentity,
};
use adoc_ports::BoxFuture;
use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tokio::sync::RwLock;
use uuid::{NoContext, Timestamp, Uuid};

const DISCOVERY_URL: &str = "https://accounts.google.com/.well-known/openid-configuration";
const GOOGLE_ISSUER: &str = "https://accounts.google.com";
const PROVIDER_DOCUMENT_MAX_BYTES: usize = 256 * 1024;
const TOKEN_RESPONSE_MAX_BYTES: usize = 64 * 1024;

#[derive(Clone, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Clone, Default)]
pub struct SystemSecureRandom;

impl SecureRandom for SystemSecureRandom {
    fn bytes(&self, output: &mut [u8]) -> Result<(), IdentityError> {
        getrandom::fill(output).map_err(|_| IdentityError::Internal)
    }

    fn uuid_v7(&self, now: DateTime<Utc>) -> Result<Uuid, IdentityError> {
        let seconds = u64::try_from(now.timestamp()).map_err(|_| IdentityError::Internal)?;
        Ok(Uuid::new_v7(Timestamp::from_unix(
            NoContext,
            seconds,
            now.timestamp_subsec_nanos(),
        )))
    }
}

#[derive(Clone)]
pub struct GoogleOidcProvider {
    client: Client,
    client_id: Arc<str>,
    client_secret: Arc<str>,
    redirect_uri: Arc<str>,
    metadata: Arc<RwLock<Option<Arc<ProviderMetadata>>>>,
    jwks: Arc<RwLock<Option<JwksCache>>>,
}

impl GoogleOidcProvider {
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Result<Self, IdentityError> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| IdentityError::ProviderUnavailable)?;
        let redirect = Url::parse(&redirect_uri).map_err(|_| IdentityError::Internal)?;
        if !matches!(redirect.scheme(), "https" | "http") || redirect.host_str().is_none() {
            return Err(IdentityError::Internal);
        }
        Ok(Self {
            client,
            client_id: Arc::from(client_id),
            client_secret: Arc::from(client_secret),
            redirect_uri: Arc::from(redirect_uri),
            metadata: Arc::new(RwLock::new(None)),
            jwks: Arc::new(RwLock::new(None)),
        })
    }

    async fn metadata(&self) -> Result<Arc<ProviderMetadata>, IdentityError> {
        if let Some(metadata) = self.metadata.read().await.as_ref() {
            return Ok(metadata.clone());
        }
        let response = self
            .client
            .get(DISCOVERY_URL)
            .send()
            .await
            .map_err(|_| IdentityError::ProviderUnavailable)?
            .error_for_status()
            .map_err(|_| IdentityError::ProviderUnavailable)?;
        let metadata = bounded_json(response, PROVIDER_DOCUMENT_MAX_BYTES)
            .await
            .map_err(|_| IdentityError::ProviderUnavailable)?;
        validate_metadata(&metadata)?;
        let metadata = Arc::new(metadata);
        *self.metadata.write().await = Some(metadata.clone());
        Ok(metadata)
    }

    async fn decoding_key(
        &self,
        kid: &str,
        force_refresh: bool,
    ) -> Result<DecodingKey, IdentityError> {
        let keys = self.jwks(force_refresh).await?;
        let key = keys
            .keys
            .iter()
            .find(|key| {
                key.kid == kid
                    && key.kty == "RSA"
                    && key
                        .alg
                        .as_deref()
                        .is_none_or(|algorithm| algorithm == "RS256")
            })
            .ok_or(IdentityError::InvalidCallback)?;
        DecodingKey::from_rsa_components(&key.n, &key.e).map_err(|_| IdentityError::InvalidCallback)
    }

    async fn jwks(&self, force_refresh: bool) -> Result<JwkSet, IdentityError> {
        if !force_refresh {
            let cache = self.jwks.read().await;
            if let Some(cache) = cache
                .as_ref()
                .filter(|cache| cache.valid_until > std::time::Instant::now())
            {
                return Ok(cache.value.clone());
            }
        }
        let metadata = self.metadata().await?;
        let response = self
            .client
            .get(&metadata.jwks_uri)
            .send()
            .await
            .map_err(|_| IdentityError::ProviderUnavailable)?
            .error_for_status()
            .map_err(|_| IdentityError::ProviderUnavailable)?;
        let max_age = response
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok())
            .and_then(cache_max_age)
            .unwrap_or(Duration::from_secs(300))
            .min(Duration::from_secs(3600));
        let value: JwkSet = bounded_json(response, PROVIDER_DOCUMENT_MAX_BYTES)
            .await
            .map_err(|_| IdentityError::ProviderUnavailable)?;
        *self.jwks.write().await = Some(JwksCache {
            value: value.clone(),
            valid_until: std::time::Instant::now() + max_age,
        });
        Ok(value)
    }

    async fn verified_claims(
        &self,
        id_token: &str,
        expected_nonce_hash: &TokenHash,
    ) -> Result<IdClaims, IdentityError> {
        let header = decode_header(id_token).map_err(|_| IdentityError::InvalidCallback)?;
        if header.alg != Algorithm::RS256 {
            return Err(IdentityError::InvalidCallback);
        }
        let kid = header.kid.ok_or(IdentityError::InvalidCallback)?;
        let key = match self.decoding_key(&kid, false).await {
            Ok(key) => key,
            Err(IdentityError::InvalidCallback) => self.decoding_key(&kid, true).await?,
            Err(error) => return Err(error),
        };
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.client_id.as_ref()]);
        validation.set_issuer(&[GOOGLE_ISSUER]);
        validation.validate_exp = true;
        let claims = decode::<IdClaims>(id_token, &key, &validation)
            .map_err(|_| IdentityError::InvalidCallback)?
            .claims;
        if !claims.email_verified || claims.iat > Utc::now().timestamp() + 60 {
            return Err(IdentityError::InvalidCallback);
        }
        let nonce_hash = Sha256::digest(claims.nonce.as_bytes());
        if nonce_hash[..].ct_eq(&expected_nonce_hash.0).unwrap_u8() != 1 {
            return Err(IdentityError::InvalidCallback);
        }
        Ok(claims)
    }
}

impl OidcProvider for GoogleOidcProvider {
    fn authorization_url<'a>(
        &'a self,
        input: &'a OidcAuthorizationInput,
    ) -> BoxFuture<'a, Result<String, IdentityError>> {
        Box::pin(async move {
            let metadata = self.metadata().await?;
            let mut url = Url::parse(&metadata.authorization_endpoint)
                .map_err(|_| IdentityError::ProviderUnavailable)?;
            url.query_pairs_mut()
                .append_pair("client_id", &self.client_id)
                .append_pair("redirect_uri", &self.redirect_uri)
                .append_pair("response_type", "code")
                .append_pair("scope", "openid email profile")
                .append_pair("state", &input.state)
                .append_pair("nonce", &input.nonce)
                .append_pair("code_challenge", &input.code_challenge)
                .append_pair("code_challenge_method", "S256");
            Ok(url.into())
        })
    }

    fn exchange<'a>(
        &'a self,
        input: OidcExchangeInput,
    ) -> BoxFuture<'a, Result<VerifiedExternalIdentity, IdentityError>> {
        Box::pin(async move {
            let metadata = self.metadata().await?;
            let response = self
                .client
                .post(&metadata.token_endpoint)
                .form(&[
                    ("grant_type", "authorization_code"),
                    ("code", input.code.as_str()),
                    ("client_id", self.client_id.as_ref()),
                    ("client_secret", self.client_secret.as_ref()),
                    ("redirect_uri", self.redirect_uri.as_ref()),
                    ("code_verifier", input.pkce_verifier.as_str()),
                ])
                .send()
                .await
                .map_err(|_| IdentityError::ProviderUnavailable)?;
            if !response.status().is_success() {
                return Err(if response.status().is_server_error() {
                    IdentityError::ProviderUnavailable
                } else {
                    IdentityError::InvalidCallback
                });
            }
            let token: TokenResponse = bounded_json(response, TOKEN_RESPONSE_MAX_BYTES)
                .await
                .map_err(|_| IdentityError::InvalidCallback)?;
            let claims = self
                .verified_claims(&token.id_token, &input.expected_nonce_hash)
                .await?;
            VerifiedExternalIdentity::google(&claims.iss, &claims.sub, &claims.email, &claims.name)
                .map_err(|_| IdentityError::InvalidCallback)
        })
    }
}

#[derive(Debug, Deserialize)]
struct ProviderMetadata {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
    id_token_signing_alg_values_supported: Vec<String>,
}

#[derive(Clone)]
struct JwksCache {
    value: JwkSet,
    valid_until: std::time::Instant,
}

#[derive(Clone, Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

#[derive(Clone, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    alg: Option<String>,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    id_token: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct IdClaims {
    iss: String,
    sub: String,
    email: String,
    email_verified: bool,
    name: String,
    nonce: String,
    iat: i64,
    #[allow(dead_code)]
    exp: i64,
    #[allow(dead_code)]
    aud: serde_json::Value,
}

fn validate_metadata(metadata: &ProviderMetadata) -> Result<(), IdentityError> {
    if metadata.issuer != GOOGLE_ISSUER
        || !metadata
            .id_token_signing_alg_values_supported
            .iter()
            .any(|algorithm| algorithm == "RS256")
    {
        return Err(IdentityError::ProviderUnavailable);
    }
    let allowed_hosts = HashSet::from([
        "accounts.google.com",
        "oauth2.googleapis.com",
        "www.googleapis.com",
    ]);
    for endpoint in [
        &metadata.authorization_endpoint,
        &metadata.token_endpoint,
        &metadata.jwks_uri,
    ] {
        let url = Url::parse(endpoint).map_err(|_| IdentityError::ProviderUnavailable)?;
        if url.scheme() != "https"
            || !url
                .host_str()
                .is_some_and(|host| allowed_hosts.contains(host))
        {
            return Err(IdentityError::ProviderUnavailable);
        }
    }
    Ok(())
}

fn cache_max_age(value: &str) -> Option<Duration> {
    value.split(',').map(str::trim).find_map(|directive| {
        directive
            .strip_prefix("max-age=")
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_secs)
    })
}

async fn bounded_json<T: DeserializeOwned>(
    mut response: reqwest::Response,
    maximum: usize,
) -> Result<T, ()> {
    if response
        .content_length()
        .is_some_and(|length| length > maximum as u64)
    {
        return Err(());
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| ())? {
        if body.len().saturating_add(chunk.len()) > maximum {
            return Err(());
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use jsonwebtoken::{EncodingKey, Header, encode};
    use rsa::{RsaPrivateKey, pkcs1::EncodeRsaPrivateKey, traits::PublicKeyParts};

    use super::*;

    #[test]
    fn metadata_rejects_non_google_and_non_https_endpoints() {
        let metadata = ProviderMetadata {
            issuer: GOOGLE_ISSUER.into(),
            authorization_endpoint: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token_endpoint: "http://oauth2.googleapis.com/token".into(),
            jwks_uri: "https://www.googleapis.com/oauth2/v3/certs".into(),
            id_token_signing_alg_values_supported: vec!["RS256".into()],
        };
        assert!(validate_metadata(&metadata).is_err());
    }

    #[test]
    fn cache_control_is_parsed_without_accepting_other_directives() {
        assert_eq!(
            cache_max_age("public, max-age=300"),
            Some(Duration::from_secs(300))
        );
        assert_eq!(cache_max_age("no-cache"), None);
    }

    #[tokio::test]
    async fn id_token_rejects_signature_audience_issuer_nonce_and_claim_failures() {
        let private = RsaPrivateKey::new(&mut rand::thread_rng(), 2048).unwrap();
        let public = private.to_public_key();
        let provider = GoogleOidcProvider::new(
            "client-id".into(),
            "client-secret".into(),
            "http://localhost/callback".into(),
        )
        .unwrap();
        *provider.jwks.write().await = Some(JwksCache {
            value: JwkSet {
                keys: vec![Jwk {
                    kid: "test-key".into(),
                    kty: "RSA".into(),
                    alg: Some("RS256".into()),
                    n: URL_SAFE_NO_PAD.encode(public.n().to_bytes_be()),
                    e: URL_SAFE_NO_PAD.encode(public.e().to_bytes_be()),
                }],
            },
            valid_until: std::time::Instant::now() + Duration::from_secs(60),
        });
        let now = Utc::now().timestamp();
        let nonce = "nonce";
        let nonce_hash = TokenHash(Sha256::digest(nonce.as_bytes()).into());
        let claims = IdClaims {
            iss: GOOGLE_ISSUER.into(),
            sub: "subject".into(),
            email: "person@example.com".into(),
            email_verified: true,
            name: "Person".into(),
            nonce: nonce.into(),
            iat: now,
            exp: now + 300,
            aud: serde_json::json!("client-id"),
        };
        let der = private.to_pkcs1_der().unwrap();
        let encoding = EncodingKey::from_rsa_der(der.as_bytes());
        let header = Header {
            kid: Some("test-key".into()),
            ..Header::new(Algorithm::RS256)
        };
        let token = encode(&header, &claims, &encoding).unwrap();
        assert!(provider.verified_claims(&token, &nonce_hash).await.is_ok());

        let mut invalid = claims.clone();
        invalid.aud = serde_json::json!("other-client");
        assert_invalid(
            &provider,
            encode(&header, &invalid, &encoding).unwrap(),
            &nonce_hash,
        )
        .await;
        invalid = claims.clone();
        invalid.iss = "https://issuer.invalid".into();
        assert_invalid(
            &provider,
            encode(&header, &invalid, &encoding).unwrap(),
            &nonce_hash,
        )
        .await;
        invalid = claims.clone();
        invalid.email_verified = false;
        assert_invalid(
            &provider,
            encode(&header, &invalid, &encoding).unwrap(),
            &nonce_hash,
        )
        .await;
        invalid = claims.clone();
        invalid.iat = now + 61;
        assert_invalid(
            &provider,
            encode(&header, &invalid, &encoding).unwrap(),
            &nonce_hash,
        )
        .await;
        assert_invalid(&provider, token.clone(), &TokenHash([9; 32])).await;
        assert_invalid(&provider, format!("{token}x"), &nonce_hash).await;
    }

    async fn assert_invalid(provider: &GoogleOidcProvider, token: String, nonce: &TokenHash) {
        assert!(matches!(
            provider.verified_claims(&token, nonce).await,
            Err(IdentityError::InvalidCallback)
        ));
    }
}

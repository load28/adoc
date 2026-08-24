use std::{fmt, sync::Arc};

pub use adoc_identity::{
    LOGIN_FLOW_MINUTES, Locale, PreferenceInput, ReturnPath, SessionLifetime, Theme,
    UserPreferences, UserSummary, VerifiedExternalIdentity,
};
use adoc_ports::BoxFuture;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use thiserror::Error;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Eq, PartialEq)]
pub struct TokenHash(pub [u8; 32]);

impl fmt::Debug for TokenHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TokenHash([REDACTED])")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashCandidate {
    pub key_id: String,
    pub hash: TokenHash,
}

#[derive(Clone)]
pub struct SigningKey {
    pub id: String,
    pub value: Arc<[u8]>,
}

impl fmt::Debug for SigningKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SigningKey")
            .field("id", &self.id)
            .field("value", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct KeyRing {
    current: SigningKey,
    previous: Option<SigningKey>,
}

impl KeyRing {
    pub fn new(current: SigningKey, previous: Option<SigningKey>) -> Result<Self, IdentityError> {
        if current.value.len() < 32 || previous.as_ref().is_some_and(|key| key.value.len() < 32) {
            return Err(IdentityError::Internal);
        }
        if previous.as_ref().is_some_and(|key| key.id == current.id) {
            return Err(IdentityError::Internal);
        }
        Ok(Self { current, previous })
    }

    pub fn current_id(&self) -> &str {
        &self.current.id
    }

    pub fn hash_current(&self, value: &str) -> HashCandidate {
        hash_with(&self.current, value)
    }

    pub fn candidates(&self, value: &str) -> Vec<HashCandidate> {
        std::iter::once(&self.current)
            .chain(self.previous.iter())
            .map(|key| hash_with(key, value))
            .collect()
    }
}

fn hash_with(key: &SigningKey, value: &str) -> HashCandidate {
    let mut mac = HmacSha256::new_from_slice(&key.value).expect("validated HMAC key");
    mac.update(value.as_bytes());
    HashCandidate {
        key_id: key.id.clone(),
        hash: TokenHash(mac.finalize().into_bytes().into()),
    }
}

#[derive(Clone, Debug)]
pub struct LoginFlowRecord {
    pub state_hash: HashCandidate,
    pub marker_hash: TokenHash,
    pub nonce_hash: TokenHash,
    pub pkce_verifier: String,
    pub return_to: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct ConsumedLoginFlow {
    pub nonce_hash: TokenHash,
    pub pkce_verifier: String,
    pub return_to: String,
}

#[derive(Clone, Debug)]
pub struct NewSessionRecord {
    pub hash: HashCandidate,
    pub lifetime: SessionLifetime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserCommandReceipt {
    pub operation_id: &'static str,
    pub key: String,
    pub request_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct SessionPrincipal {
    pub user: UserSummary,
    pub session_hash: TokenHash,
}

#[derive(Clone, Debug)]
pub struct EstablishedSession {
    pub user: UserSummary,
    pub token: String,
    pub session_hash: TokenHash,
    pub idle_expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct LoginStart {
    pub authorization_url: String,
    pub marker: String,
}

#[derive(Clone, Debug)]
pub struct LoginCompletion {
    pub return_to: String,
    pub session: EstablishedSession,
}

#[derive(Clone, Debug)]
pub struct OidcAuthorizationInput {
    pub state: String,
    pub nonce: String,
    pub code_challenge: String,
}

#[derive(Clone, Debug)]
pub struct OidcExchangeInput {
    pub code: String,
    pub pkce_verifier: String,
    pub expected_nonce_hash: TokenHash,
}

pub trait OidcProvider: Send + Sync {
    fn authorization_url<'a>(
        &'a self,
        input: &'a OidcAuthorizationInput,
    ) -> BoxFuture<'a, Result<String, IdentityError>>;
    fn exchange<'a>(
        &'a self,
        input: OidcExchangeInput,
    ) -> BoxFuture<'a, Result<VerifiedExternalIdentity, IdentityError>>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoginRateLimitScope {
    Start,
    Callback,
}

pub trait LoginRateLimiter: Send + Sync {
    fn check<'a>(
        &'a self,
        scope: LoginRateLimitScope,
        signals: Vec<TokenHash>,
    ) -> BoxFuture<'a, Result<(), IdentityError>>;
}

pub trait IdentityRepository: Send + Sync {
    fn create_login_flow<'a>(
        &'a self,
        flow: LoginFlowRecord,
    ) -> BoxFuture<'a, Result<(), IdentityError>>;
    fn consume_login_flow<'a>(
        &'a self,
        state: Vec<HashCandidate>,
        marker: Vec<HashCandidate>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<ConsumedLoginFlow, IdentityError>>;
    fn establish_identity<'a>(
        &'a self,
        identity: VerifiedExternalIdentity,
        proposed_user_id: Uuid,
        session: NewSessionRecord,
        revoke: Vec<HashCandidate>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<UserSummary, IdentityError>>;
    fn authenticate<'a>(
        &'a self,
        candidates: Vec<HashCandidate>,
        now: DateTime<Utc>,
        proposed_idle_expires_at: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<SessionPrincipal, IdentityError>>;
    fn revoke<'a>(
        &'a self,
        candidates: Vec<HashCandidate>,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<(), IdentityError>>;
    fn preferences<'a>(
        &'a self,
        user_id: Uuid,
    ) -> BoxFuture<'a, Result<UserPreferences, IdentityError>>;
    fn update_preferences<'a>(
        &'a self,
        user_id: Uuid,
        expected_revision: i64,
        input: PreferenceInput,
        command: UserCommandReceipt,
        now: DateTime<Utc>,
    ) -> BoxFuture<'a, Result<UserPreferences, IdentityError>>;
}

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub trait SecureRandom: Send + Sync {
    fn bytes(&self, output: &mut [u8]) -> Result<(), IdentityError>;
    fn uuid_v7(&self, now: DateTime<Utc>) -> Result<Uuid, IdentityError>;
}

#[derive(Clone)]
pub struct IdentityService {
    repository: Arc<dyn IdentityRepository>,
    provider: Arc<dyn OidcProvider>,
    clock: Arc<dyn Clock>,
    random: Arc<dyn SecureRandom>,
    rate_limiter: Arc<dyn LoginRateLimiter>,
    flow_keys: KeyRing,
    session_keys: KeyRing,
    session_idle: Duration,
}

pub struct IdentityServicePorts {
    pub repository: Arc<dyn IdentityRepository>,
    pub provider: Arc<dyn OidcProvider>,
    pub clock: Arc<dyn Clock>,
    pub random: Arc<dyn SecureRandom>,
    pub rate_limiter: Arc<dyn LoginRateLimiter>,
}

pub struct IdentityServiceSecurity {
    pub flow_keys: KeyRing,
    pub session_keys: KeyRing,
    pub session_idle: Duration,
}

impl IdentityService {
    pub fn new(ports: IdentityServicePorts, security: IdentityServiceSecurity) -> Self {
        Self {
            repository: ports.repository,
            provider: ports.provider,
            clock: ports.clock,
            random: ports.random,
            rate_limiter: ports.rate_limiter,
            flow_keys: security.flow_keys,
            session_keys: security.session_keys,
            session_idle: security.session_idle,
        }
    }

    pub async fn start_login(
        &self,
        return_to: Option<&str>,
        peer_signal: &str,
    ) -> Result<LoginStart, IdentityError> {
        self.check_rate_limit(LoginRateLimitScope::Start, &[peer_signal])
            .await?;
        let return_to = ReturnPath::parse(return_to).map_err(|_| IdentityError::Validation)?;
        let state = self.random_token()?;
        let marker = self.random_token()?;
        let nonce = self.random_token()?;
        let pkce_verifier = self.random_token()?;
        let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(pkce_verifier.as_bytes()));
        let now = self.clock.now();
        self.repository
            .create_login_flow(LoginFlowRecord {
                state_hash: self.flow_keys.hash_current(&state),
                marker_hash: self.flow_keys.hash_current(&marker).hash,
                nonce_hash: TokenHash(Sha256::digest(nonce.as_bytes()).into()),
                pkce_verifier,
                return_to: return_to.as_str().to_owned(),
                created_at: now,
                expires_at: now + Duration::minutes(LOGIN_FLOW_MINUTES),
            })
            .await?;
        let authorization_url = self
            .provider
            .authorization_url(&OidcAuthorizationInput {
                state,
                nonce,
                code_challenge,
            })
            .await?;
        Ok(LoginStart {
            authorization_url,
            marker,
        })
    }

    pub async fn complete_login(
        &self,
        state: &str,
        marker: &str,
        code: &str,
        existing_session: Option<&str>,
        peer_signal: &str,
    ) -> Result<LoginCompletion, IdentityError> {
        if state.len() > 2048 || marker.len() > 2048 || code.is_empty() || code.len() > 4096 {
            return Err(IdentityError::InvalidCallback);
        }
        self.check_rate_limit(LoginRateLimitScope::Callback, &[peer_signal, marker])
            .await?;
        let now = self.clock.now();
        let flow = self
            .repository
            .consume_login_flow(
                self.flow_keys.candidates(state),
                self.flow_keys.candidates(marker),
                now,
            )
            .await?;
        let identity = self
            .provider
            .exchange(OidcExchangeInput {
                code: code.to_owned(),
                pkce_verifier: flow.pkce_verifier,
                expected_nonce_hash: flow.nonce_hash,
            })
            .await?;
        let token = self.random_token()?;
        let session_hash = self.session_keys.hash_current(&token);
        let lifetime = SessionLifetime::new(now, self.session_idle);
        let user = self
            .repository
            .establish_identity(
                identity,
                self.random.uuid_v7(now)?,
                NewSessionRecord {
                    hash: session_hash.clone(),
                    lifetime,
                },
                existing_session
                    .map(|value| self.session_keys.candidates(value))
                    .unwrap_or_default(),
                now,
            )
            .await?;
        Ok(LoginCompletion {
            return_to: flow.return_to,
            session: EstablishedSession {
                user,
                token,
                session_hash: session_hash.hash,
                idle_expires_at: lifetime.idle_expires_at,
            },
        })
    }

    pub async fn authenticate(&self, token: &str) -> Result<SessionPrincipal, IdentityError> {
        if token.is_empty() || token.len() > 2048 {
            return Err(IdentityError::AuthenticationRequired);
        }
        let now = self.clock.now();
        self.repository
            .authenticate(
                self.session_keys.candidates(token),
                now,
                now + self.session_idle,
            )
            .await
    }

    pub async fn logout(&self, token: &str) -> Result<(), IdentityError> {
        self.repository
            .revoke(self.session_keys.candidates(token), self.clock.now())
            .await
    }

    pub async fn preferences(&self, user_id: Uuid) -> Result<UserPreferences, IdentityError> {
        self.repository.preferences(user_id).await
    }

    pub async fn update_preferences(
        &self,
        user_id: Uuid,
        expected_revision: i64,
        input: PreferenceInput,
        idempotency_key: &str,
    ) -> Result<UserPreferences, IdentityError> {
        let input = input.validate().map_err(|_| IdentityError::Validation)?;
        if !(16..=128).contains(&idempotency_key.len()) {
            return Err(IdentityError::Validation);
        }
        let now = self.clock.now();
        let request_hash = hex::encode(Sha256::digest(
            format!(
                "{}\n{}\n{}\n{}",
                expected_revision,
                input.locale.as_str(),
                input.timezone,
                input.theme.as_str()
            )
            .as_bytes(),
        ));
        self.repository
            .update_preferences(
                user_id,
                expected_revision,
                input,
                UserCommandReceipt {
                    operation_id: "updateUserPreferences",
                    key: idempotency_key.to_owned(),
                    request_hash,
                    created_at: now,
                    expires_at: now + Duration::hours(24),
                },
                now,
            )
            .await
    }

    fn random_token(&self) -> Result<String, IdentityError> {
        let mut bytes = [0_u8; 32];
        self.random.bytes(&mut bytes)?;
        Ok(URL_SAFE_NO_PAD.encode(bytes))
    }

    async fn check_rate_limit(
        &self,
        scope: LoginRateLimitScope,
        raw_signals: &[&str],
    ) -> Result<(), IdentityError> {
        if raw_signals
            .iter()
            .any(|signal| signal.is_empty() || signal.len() > 2048)
        {
            return Err(IdentityError::InvalidCallback);
        }
        let signals = raw_signals
            .iter()
            .map(|signal| self.flow_keys.hash_current(signal).hash)
            .collect();
        self.rate_limiter.check(scope, signals).await
    }
}

#[derive(Clone, Debug)]
pub struct CsrfProtector {
    keys: KeyRing,
}

impl CsrfProtector {
    pub fn new(keys: KeyRing) -> Self {
        Self { keys }
    }

    pub fn issue(
        &self,
        session_hash: &TokenHash,
        random: &dyn SecureRandom,
    ) -> Result<String, IdentityError> {
        let mut nonce = [0_u8; 32];
        random.bytes(&mut nonce)?;
        let key_id = self.keys.current_id();
        let nonce = URL_SAFE_NO_PAD.encode(nonce);
        let mac_input = csrf_mac_input(key_id, session_hash, &nonce);
        let mac = self.keys.hash_current(&mac_input).hash;
        Ok(format!(
            "{key_id}.{nonce}.{}",
            URL_SAFE_NO_PAD.encode(mac.0)
        ))
    }

    pub fn validate(
        &self,
        session_hash: &TokenHash,
        cookie: &str,
        header: &str,
    ) -> Result<(), IdentityError> {
        if cookie.len() > 1024 || cookie.as_bytes().ct_eq(header.as_bytes()).unwrap_u8() != 1 {
            return Err(IdentityError::CsrfInvalid);
        }
        let parts = cookie.split('.').collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err(IdentityError::CsrfInvalid);
        }
        let supplied = URL_SAFE_NO_PAD
            .decode(parts[2])
            .map_err(|_| IdentityError::CsrfInvalid)?;
        let mac_input = csrf_mac_input(parts[0], session_hash, parts[1]);
        let valid = self
            .keys
            .candidates(&mac_input)
            .into_iter()
            .filter(|candidate| candidate.key_id == parts[0])
            .any(|candidate| supplied.as_slice().ct_eq(&candidate.hash.0).unwrap_u8() == 1);
        if valid {
            Ok(())
        } else {
            Err(IdentityError::CsrfInvalid)
        }
    }
}

fn csrf_mac_input(key_id: &str, session_hash: &TokenHash, nonce: &str) -> String {
    format!(
        "{key_id}.{}.{nonce}",
        URL_SAFE_NO_PAD.encode(session_hash.0)
    )
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum IdentityError {
    #[error("identity input is invalid")]
    Validation,
    #[error("authentication is required")]
    AuthenticationRequired,
    #[error("authentication callback is invalid")]
    InvalidCallback,
    #[error("CSRF validation failed")]
    CsrfInvalid,
    #[error("resource revision conflicts")]
    RevisionConflict { current_revision: i64 },
    #[error("idempotency key was reused with another request")]
    IdempotencyKeyReused,
    #[error("identity provider is unavailable")]
    ProviderUnavailable,
    #[error("identity storage is unavailable")]
    StorageUnavailable,
    #[error("login rate limit exceeded")]
    RateLimited,
    #[error("identity operation failed")]
    Internal,
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::TimeZone;

    use super::*;

    struct FixedRandom;

    impl SecureRandom for FixedRandom {
        fn bytes(&self, output: &mut [u8]) -> Result<(), IdentityError> {
            output.fill(7);
            Ok(())
        }

        fn uuid_v7(&self, _now: DateTime<Utc>) -> Result<Uuid, IdentityError> {
            Ok(Uuid::nil())
        }
    }

    fn keys() -> KeyRing {
        KeyRing::new(
            SigningKey {
                id: "current".into(),
                value: Arc::from([1_u8; 32]),
            },
            Some(SigningKey {
                id: "previous".into(),
                value: Arc::from([2_u8; 32]),
            }),
        )
        .unwrap()
    }

    #[test]
    fn token_hash_and_key_debug_are_redacted() {
        let hash = keys().hash_current("secret-token");
        assert!(!format!("{hash:?}").contains("secret-token"));
        assert!(!format!("{:?}", keys()).contains("010101"));
    }

    #[test]
    fn csrf_is_bound_to_exact_session_and_header_cookie_pair() {
        let protector = CsrfProtector::new(keys());
        let session = TokenHash([3; 32]);
        let token = protector.issue(&session, &FixedRandom).unwrap();
        assert!(!token.contains(&URL_SAFE_NO_PAD.encode(session.0)));
        assert!(protector.validate(&session, &token, &token).is_ok());
        assert!(
            protector
                .validate(&TokenHash([4; 32]), &token, &token)
                .is_err()
        );
        assert!(protector.validate(&session, &token, "different").is_err());
    }

    struct FixedClock(DateTime<Utc>);

    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    struct IncrementingRandom(Mutex<u8>);

    impl SecureRandom for IncrementingRandom {
        fn bytes(&self, output: &mut [u8]) -> Result<(), IdentityError> {
            let mut value = self.0.lock().unwrap();
            *value += 1;
            output.fill(*value);
            Ok(())
        }

        fn uuid_v7(&self, _now: DateTime<Utc>) -> Result<Uuid, IdentityError> {
            Ok(Uuid::parse_str("00000000-0000-7000-8000-000000000001").unwrap())
        }
    }

    #[derive(Default)]
    struct FakeRepository {
        flow: Mutex<Option<LoginFlowRecord>>,
        revoked: Mutex<bool>,
    }

    impl IdentityRepository for FakeRepository {
        fn create_login_flow<'a>(
            &'a self,
            flow: LoginFlowRecord,
        ) -> BoxFuture<'a, Result<(), IdentityError>> {
            Box::pin(async move {
                *self.flow.lock().unwrap() = Some(flow);
                Ok(())
            })
        }

        fn consume_login_flow<'a>(
            &'a self,
            state: Vec<HashCandidate>,
            marker: Vec<HashCandidate>,
            now: DateTime<Utc>,
        ) -> BoxFuture<'a, Result<ConsumedLoginFlow, IdentityError>> {
            Box::pin(async move {
                let flow = self
                    .flow
                    .lock()
                    .unwrap()
                    .take()
                    .ok_or(IdentityError::InvalidCallback)?;
                if flow.expires_at <= now
                    || !state
                        .iter()
                        .any(|candidate| candidate.hash == flow.state_hash.hash)
                    || !marker
                        .iter()
                        .any(|candidate| candidate.hash == flow.marker_hash)
                {
                    return Err(IdentityError::InvalidCallback);
                }
                Ok(ConsumedLoginFlow {
                    nonce_hash: flow.nonce_hash,
                    pkce_verifier: flow.pkce_verifier,
                    return_to: flow.return_to,
                })
            })
        }

        fn establish_identity<'a>(
            &'a self,
            identity: VerifiedExternalIdentity,
            proposed_user_id: Uuid,
            _session: NewSessionRecord,
            revoke: Vec<HashCandidate>,
            _now: DateTime<Utc>,
        ) -> BoxFuture<'a, Result<UserSummary, IdentityError>> {
            Box::pin(async move {
                *self.revoked.lock().unwrap() = !revoke.is_empty();
                Ok(UserSummary {
                    id: proposed_user_id,
                    email: identity.email.as_str().to_owned(),
                    display_name: identity.display_name.as_str().to_owned(),
                    locale: Locale::Ko,
                    timezone: "Asia/Seoul".into(),
                })
            })
        }

        fn authenticate<'a>(
            &'a self,
            _candidates: Vec<HashCandidate>,
            _now: DateTime<Utc>,
            _proposed_idle_expires_at: DateTime<Utc>,
        ) -> BoxFuture<'a, Result<SessionPrincipal, IdentityError>> {
            Box::pin(async { Err(IdentityError::AuthenticationRequired) })
        }

        fn revoke<'a>(
            &'a self,
            _candidates: Vec<HashCandidate>,
            _now: DateTime<Utc>,
        ) -> BoxFuture<'a, Result<(), IdentityError>> {
            Box::pin(async { Ok(()) })
        }

        fn preferences<'a>(
            &'a self,
            _user_id: Uuid,
        ) -> BoxFuture<'a, Result<UserPreferences, IdentityError>> {
            Box::pin(async { Err(IdentityError::AuthenticationRequired) })
        }

        fn update_preferences<'a>(
            &'a self,
            _user_id: Uuid,
            _expected_revision: i64,
            _input: PreferenceInput,
            _command: UserCommandReceipt,
            _now: DateTime<Utc>,
        ) -> BoxFuture<'a, Result<UserPreferences, IdentityError>> {
            Box::pin(async { Err(IdentityError::AuthenticationRequired) })
        }
    }

    struct FakeProvider;

    impl OidcProvider for FakeProvider {
        fn authorization_url<'a>(
            &'a self,
            input: &'a OidcAuthorizationInput,
        ) -> BoxFuture<'a, Result<String, IdentityError>> {
            Box::pin(async move {
                assert_eq!(input.code_challenge.len(), 43);
                Ok(format!(
                    "https://accounts.google.com/auth?state={}",
                    input.state
                ))
            })
        }

        fn exchange<'a>(
            &'a self,
            input: OidcExchangeInput,
        ) -> BoxFuture<'a, Result<VerifiedExternalIdentity, IdentityError>> {
            Box::pin(async move {
                assert_eq!(input.code, "code");
                VerifiedExternalIdentity::google(
                    "https://accounts.google.com",
                    "subject",
                    "person@example.com",
                    "Person",
                )
                .map_err(|_| IdentityError::InvalidCallback)
            })
        }
    }

    struct AllowRateLimit;

    impl LoginRateLimiter for AllowRateLimit {
        fn check<'a>(
            &'a self,
            _scope: LoginRateLimitScope,
            _signals: Vec<TokenHash>,
        ) -> BoxFuture<'a, Result<(), IdentityError>> {
            Box::pin(async { Ok(()) })
        }
    }

    #[tokio::test]
    async fn login_flow_is_one_shot_and_rotates_existing_browser_session() {
        let now = Utc.with_ymd_and_hms(2026, 1, 15, 9, 0, 0).unwrap();
        let repository = Arc::new(FakeRepository::default());
        let service = IdentityService::new(
            IdentityServicePorts {
                repository: repository.clone(),
                provider: Arc::new(FakeProvider),
                clock: Arc::new(FixedClock(now)),
                random: Arc::new(IncrementingRandom(Mutex::new(0))),
                rate_limiter: Arc::new(AllowRateLimit),
            },
            IdentityServiceSecurity {
                flow_keys: keys(),
                session_keys: keys(),
                session_idle: Duration::hours(12),
            },
        );
        let start = service
            .start_login(Some("/workspaces"), "192.0.2.10")
            .await
            .unwrap();
        let state = start.authorization_url.split("state=").nth(1).unwrap();
        let completion = service
            .complete_login(
                state,
                &start.marker,
                "code",
                Some("old-session"),
                "192.0.2.10",
            )
            .await
            .unwrap();
        assert_eq!(completion.return_to, "/workspaces");
        assert!(*repository.revoked.lock().unwrap());
        assert!(matches!(
            service
                .complete_login(state, &start.marker, "code", None, "192.0.2.10")
                .await,
            Err(IdentityError::InvalidCallback)
        ));
    }
}

use std::{net::SocketAddr, sync::Arc};

use adoc_adapters::{
    identity::{GoogleOidcProvider, SystemClock, SystemSecureRandom},
    postgres::{PostgresIdentityRepository, PostgresStore},
    rate_limit::RedisLoginRateLimiter,
};
use adoc_application::identity::{
    CsrfProtector, IdentityError, IdentityService, IdentityServicePorts, IdentityServiceSecurity,
    KeyRing, PreferenceInput, SessionPrincipal, SigningKey,
};
use adoc_configuration::{AppConfig, RotatingSecret};
use axum::{
    Json, Router,
    extract::{ConnectInfo, FromRequestParts, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header, request::Parts},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post, put},
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::HealthState;

const SESSION_COOKIE: &str = "adoc_session";
const CSRF_COOKIE: &str = "adoc_csrf";
const LOGIN_COOKIE: &str = "adoc_login";

#[derive(Clone)]
pub(crate) struct IdentityRuntime {
    service: Arc<IdentityService>,
    csrf: CsrfProtector,
    random: Arc<SystemSecureRandom>,
    public_origin: Arc<str>,
    cookie_max_age_seconds: u64,
}

impl IdentityRuntime {
    pub(crate) async fn new(
        config: &AppConfig,
        store: &PostgresStore,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let public_origin = config
            .common
            .public_origin
            .as_ref()
            .ok_or("API requires ADOC_PUBLIC_ORIGIN")?;
        let origin = public_origin.origin().ascii_serialization();
        let redirect_uri = format!("{origin}/api/v1/auth/google/callback");
        let provider = GoogleOidcProvider::new(
            config
                .auth
                .google_client_id
                .as_ref()
                .ok_or("Google client ID is unavailable")?
                .value
                .expose()
                .to_owned(),
            config
                .auth
                .google_client_secret
                .as_ref()
                .ok_or("Google client secret is unavailable")?
                .value
                .expose()
                .to_owned(),
            redirect_uri,
        )?;
        let flow_keys = key_ring(&config.auth.token_hash_pepper)?;
        let session_keys = key_ring(
            config
                .auth
                .session_hmac
                .as_ref()
                .ok_or("session HMAC key is unavailable")?,
        )?;
        let csrf_keys = key_ring(
            config
                .auth
                .csrf_hmac
                .as_ref()
                .ok_or("CSRF key is unavailable")?,
        )?;
        let random = Arc::new(SystemSecureRandom);
        let rate_limiter = Arc::new(
            RedisLoginRateLimiter::connect(
                config.dependencies.redis_url.value.expose(),
                &config.dependencies.queue_namespace,
            )
            .await?,
        );
        Ok(Self {
            service: Arc::new(IdentityService::new(
                IdentityServicePorts {
                    repository: Arc::new(PostgresIdentityRepository::new(store)),
                    provider: Arc::new(provider),
                    clock: Arc::new(SystemClock),
                    random: random.clone(),
                    rate_limiter,
                },
                IdentityServiceSecurity {
                    flow_keys,
                    session_keys,
                    session_idle: chrono::Duration::from_std(config.auth.session_ttl)
                        .map_err(|_| "session TTL is out of range")?,
                },
            )),
            csrf: CsrfProtector::new(csrf_keys),
            random,
            public_origin: Arc::from(origin),
            cookie_max_age_seconds: config.auth.session_ttl.as_secs(),
        })
    }
}

pub(crate) fn key_ring(secret: &RotatingSecret) -> Result<KeyRing, IdentityError> {
    let current = SigningKey {
        id: secret.current_id().to_owned(),
        value: Arc::from(secret.current().expose().as_bytes()),
    };
    let previous = secret.previous().map(|(id, value)| SigningKey {
        id: id.to_owned(),
        value: Arc::from(value.expose().as_bytes()),
    });
    KeyRing::new(current, previous)
}

pub(crate) fn identity_routes() -> Router<HealthState> {
    Router::new()
        .route("/auth/google/start", get(begin_google_login))
        .route("/auth/google/callback", get(complete_google_login))
        .route("/session", get(get_session))
        .route("/session/logout", post(logout))
        .route("/preferences", get(get_user_preferences))
        .route("/preferences", put(update_user_preferences))
}

#[derive(Deserialize)]
struct LoginStartQuery {
    #[serde(rename = "returnTo")]
    return_to: Option<String>,
}

async fn begin_google_login(
    State(state): State<HealthState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Query(query): Query<LoginStartQuery>,
) -> Result<Response, Problem> {
    let start = state
        .identity
        .service
        .start_login(query.return_to.as_deref(), &peer.ip().to_string())
        .await
        .map_err(Problem::from)?;
    let mut response = Redirect::temporary(&start.authorization_url).into_response();
    append_set_cookie(&mut response, login_cookie(&start.marker))?;
    Ok(response)
}

#[derive(Deserialize)]
struct LoginCallbackQuery {
    code: String,
    state: String,
}

async fn complete_google_login(
    State(state): State<HealthState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<LoginCallbackQuery>,
) -> Result<Response, Problem> {
    let marker = cookie(&headers, LOGIN_COOKIE)
        .ok_or_else(|| Problem::from(IdentityError::InvalidCallback))?;
    let completion = state
        .identity
        .service
        .complete_login(
            &query.state,
            marker,
            &query.code,
            cookie(&headers, SESSION_COOKIE),
            &peer.ip().to_string(),
        )
        .await
        .map_err(Problem::from)?;
    let csrf = state
        .identity
        .csrf
        .issue(
            &completion.session.session_hash,
            state.identity.random.as_ref(),
        )
        .map_err(Problem::from)?;
    let mut response = Redirect::to(&completion.return_to).into_response();
    append_set_cookie(
        &mut response,
        session_cookie(
            &completion.session.token,
            state.identity.cookie_max_age_seconds,
        ),
    )?;
    append_set_cookie(
        &mut response,
        csrf_cookie(&csrf, state.identity.cookie_max_age_seconds),
    )?;
    append_set_cookie(
        &mut response,
        clear_cookie(LOGIN_COOKIE, "/api/v1/auth/google"),
    )?;
    Ok(response)
}

pub(crate) struct Authenticated {
    pub(crate) principal: SessionPrincipal,
    pub(crate) token: String,
}

impl FromRequestParts<HealthState> for Authenticated {
    type Rejection = Problem;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &HealthState,
    ) -> Result<Self, Self::Rejection> {
        let token = cookie(&parts.headers, SESSION_COOKIE)
            .ok_or_else(|| Problem::from(IdentityError::AuthenticationRequired))?
            .to_owned();
        let principal = state
            .identity
            .service
            .authenticate(&token)
            .await
            .map_err(Problem::from)?;
        Ok(Self { principal, token })
    }
}

async fn get_session(
    State(state): State<HealthState>,
    authenticated: Authenticated,
) -> Result<Response, Problem> {
    let csrf = state
        .identity
        .csrf
        .issue(
            &authenticated.principal.session_hash,
            state.identity.random.as_ref(),
        )
        .map_err(Problem::from)?;
    let user = authenticated.principal.user;
    let workspaces = state
        .governance
        .service
        .list_workspaces(user.id)
        .await
        .map_err(Problem::from)?;
    let mut response = Json(json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "displayName": user.display_name,
            "locale": user.locale,
            "timezone": user.timezone,
        },
        "workspaces": workspaces,
    }))
    .into_response();
    append_set_cookie(
        &mut response,
        csrf_cookie(&csrf, state.identity.cookie_max_age_seconds),
    )?;
    Ok(response)
}

async fn logout(
    State(state): State<HealthState>,
    headers: HeaderMap,
    authenticated: Authenticated,
) -> Result<Response, Problem> {
    validate_command(&state.identity, &headers, &authenticated)?;
    idempotency_key(&headers)?;
    state
        .identity
        .service
        .logout(&authenticated.token)
        .await
        .map_err(Problem::from)?;
    let mut response = StatusCode::NO_CONTENT.into_response();
    append_set_cookie(&mut response, clear_cookie(SESSION_COOKIE, "/"))?;
    append_set_cookie(&mut response, clear_cookie(CSRF_COOKIE, "/"))?;
    Ok(response)
}

async fn get_user_preferences(
    State(state): State<HealthState>,
    authenticated: Authenticated,
) -> Result<Json<serde_json::Value>, Problem> {
    let preferences = state
        .identity
        .service
        .preferences(authenticated.principal.user.id)
        .await
        .map_err(Problem::from)?;
    Ok(Json(preferences_json(preferences)))
}

async fn update_user_preferences(
    State(state): State<HealthState>,
    headers: HeaderMap,
    authenticated: Authenticated,
    Json(input): Json<PreferenceInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    validate_command(&state.identity, &headers, &authenticated)?;
    let idempotency_key = idempotency_key(&headers)?;
    let expected_revision = expected_revision(&headers)?;
    let preferences = state
        .identity
        .service
        .update_preferences(
            authenticated.principal.user.id,
            expected_revision,
            input,
            idempotency_key,
        )
        .await
        .map_err(Problem::from)?;
    Ok(Json(preferences_json(preferences)))
}

fn preferences_json(preferences: adoc_application::identity::UserPreferences) -> serde_json::Value {
    json!({
        "locale": preferences.locale,
        "timezone": preferences.timezone,
        "theme": preferences.theme,
        "revision": preferences.revision,
    })
}

pub(crate) fn validate_command(
    runtime: &IdentityRuntime,
    headers: &HeaderMap,
    authenticated: &Authenticated,
) -> Result<(), Problem> {
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Problem::from(IdentityError::CsrfInvalid))?;
    if origin != runtime.public_origin.as_ref() {
        return Err(Problem::from(IdentityError::CsrfInvalid));
    }
    let csrf_cookie =
        cookie(headers, CSRF_COOKIE).ok_or_else(|| Problem::from(IdentityError::CsrfInvalid))?;
    let csrf_header = headers
        .get("x-csrf-token")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Problem::from(IdentityError::CsrfInvalid))?;
    runtime
        .csrf
        .validate(
            &authenticated.principal.session_hash,
            csrf_cookie,
            csrf_header,
        )
        .map_err(Problem::from)
}

pub(crate) fn idempotency_key(headers: &HeaderMap) -> Result<&str, Problem> {
    headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| (16..=128).contains(&value.len()))
        .ok_or_else(|| Problem::from(IdentityError::Validation))
}

pub(crate) fn expected_revision(headers: &HeaderMap) -> Result<i64, Problem> {
    headers
        .get(header::IF_MATCH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix('"')?.strip_suffix('"'))
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value >= 0)
        .ok_or_else(|| Problem::from(IdentityError::Validation))
}

fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find_map(|(candidate, value)| (candidate == name).then_some(value))
}

fn session_cookie(token: &str, max_age_seconds: u64) -> String {
    format!(
        "{SESSION_COOKIE}={token}; Path=/; Max-Age={max_age_seconds}; Secure; HttpOnly; SameSite=Lax"
    )
}

fn csrf_cookie(token: &str, max_age_seconds: u64) -> String {
    format!("{CSRF_COOKIE}={token}; Path=/; Max-Age={max_age_seconds}; Secure; SameSite=Strict")
}

fn login_cookie(token: &str) -> String {
    format!(
        "{LOGIN_COOKIE}={token}; Path=/api/v1/auth/google; Max-Age=600; Secure; HttpOnly; SameSite=Lax"
    )
}

fn clear_cookie(name: &str, path: &str) -> String {
    format!("{name}=; Path={path}; Max-Age=0; Secure; HttpOnly; SameSite=Lax")
}

fn append_set_cookie(response: &mut Response, value: String) -> Result<(), Problem> {
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&value).map_err(|_| Problem::internal())?,
    );
    Ok(())
}

pub(crate) struct Problem {
    pub(crate) status: StatusCode,
    pub(crate) code: &'static str,
    pub(crate) retryable: bool,
    pub(crate) current_revision: Option<i64>,
    pub(crate) reference_count: Option<i64>,
    pub(crate) publish_conflict: Option<PublishConflict>,
}

#[derive(Clone, Copy)]
pub(crate) struct PublishConflict {
    pub(crate) base_version_id: Option<Uuid>,
    pub(crate) current_version_id: Option<Uuid>,
    pub(crate) draft_id: Uuid,
}

impl Problem {
    pub(crate) fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "INTERNAL_ERROR",
            retryable: false,
            current_revision: None,
            reference_count: None,
            publish_conflict: None,
        }
    }
}

impl From<IdentityError> for Problem {
    fn from(error: IdentityError) -> Self {
        match error {
            IdentityError::Validation => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "VALIDATION_FAILED",
                retryable: false,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            IdentityError::AuthenticationRequired => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "AUTH_REQUIRED",
                retryable: false,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            IdentityError::InvalidCallback => Self {
                status: StatusCode::BAD_REQUEST,
                code: "AUTH_CALLBACK_INVALID",
                retryable: false,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            IdentityError::CsrfInvalid => Self {
                status: StatusCode::FORBIDDEN,
                code: "CSRF_INVALID",
                retryable: false,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            IdentityError::RevisionConflict { current_revision } => Self {
                status: StatusCode::CONFLICT,
                code: "REVISION_CONFLICT",
                retryable: false,
                current_revision: Some(current_revision),
                reference_count: None,
                publish_conflict: None,
            },
            IdentityError::IdempotencyKeyReused => Self {
                status: StatusCode::CONFLICT,
                code: "IDEMPOTENCY_KEY_REUSED",
                retryable: false,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            IdentityError::ProviderUnavailable => Self {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "AUTH_PROVIDER_UNAVAILABLE",
                retryable: true,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            IdentityError::StorageUnavailable => Self {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "DEPENDENCY_UNAVAILABLE",
                retryable: true,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            IdentityError::RateLimited => Self {
                status: StatusCode::TOO_MANY_REQUESTS,
                code: "RATE_LIMITED",
                retryable: true,
                current_revision: None,
                reference_count: None,
                publish_conflict: None,
            },
            IdentityError::Internal => Self::internal(),
        }
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let title = self.status.canonical_reason().unwrap_or("Request failed");
        let mut body = json!({
            "type": format!("urn:adoc:error:{}", self.code),
            "title": title,
            "status": self.status.as_u16(),
            "code": self.code,
            "retryable": self.retryable,
            "correlationId": Uuid::now_v7(),
            "fieldErrors": [],
        });
        if let Some(revision) = self.current_revision {
            body["currentRevision"] = json!(revision);
        }
        if let Some(count) = self.reference_count {
            body["referenceCount"] = json!(count);
        }
        if let Some(conflict) = self.publish_conflict {
            body["baseVersionId"] = json!(conflict.base_version_id);
            body["currentVersionId"] = json!(conflict.current_version_id);
            body["draftId"] = json!(conflict.draft_id);
        }
        let rate_limited = self.code == "RATE_LIMITED";
        let mut response = (
            self.status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            Json(body),
        )
            .into_response();
        if rate_limited {
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from_static("600"));
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_contract_never_exposes_session_to_javascript() {
        let cookie = session_cookie("opaque", 43_200);
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
        let csrf = csrf_cookie("csrf", 43_200);
        assert!(!csrf.contains("HttpOnly"));
        assert!(csrf.contains("Path=/;"));
    }

    #[test]
    fn cookie_parser_does_not_match_prefixes() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "other_adoc_session=x; adoc_session=right".parse().unwrap(),
        );
        assert_eq!(cookie(&headers, SESSION_COOKIE), Some("right"));
    }
}

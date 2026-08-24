use std::{path::PathBuf, sync::Arc};

use adoc_adapters::{
    identity::SystemClock,
    object_storage::{EicarMalwareScanner, LocalObjectStorage},
    postgres::{PostgresFileRepository, PostgresStore},
};
use adoc_application::operations::{
    ByteRange, ByteStream, CreateFileUploadInput, FileService, StorageError,
};
use adoc_configuration::{AppConfig, ObjectStorageDriver};
use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use futures_util::TryStreamExt;
use uuid::Uuid;

use crate::{
    HealthState,
    identity_http::{
        Authenticated, Problem, expected_revision, idempotency_key, key_ring, validate_command,
    },
};

#[derive(Clone)]
pub(crate) struct FileRuntime {
    pub(crate) service: Arc<FileService>,
}

impl FileRuntime {
    pub(crate) fn new(
        config: &AppConfig,
        store: &PostgresStore,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if config.storage.driver != ObjectStorageDriver::Local {
            return Err("S3 object storage adapter is not configured in this release".into());
        }
        let root = config
            .storage
            .local_root
            .clone()
            .ok_or("local object storage root is missing")?;
        let public_origin = config
            .common
            .public_origin
            .as_ref()
            .ok_or("public origin is missing")?
            .as_str()
            .trim_end_matches('/');
        Ok(Self {
            service: Arc::new(FileService::new(
                Arc::new(PostgresFileRepository::new(store)),
                Arc::new(
                    LocalObjectStorage::new(absolute(root))
                        .map_err(|_| "invalid local object storage root")?,
                ),
                Arc::new(EicarMalwareScanner),
                Arc::new(SystemClock),
                Arc::new(key_ring(&config.auth.token_hash_pepper)?),
                config.storage.upload_max_bytes,
                Arc::from(format!("{public_origin}/api/v1")),
            )),
        })
    }
}

fn absolute(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    }
}

pub(crate) fn file_routes() -> Router<HealthState> {
    Router::new()
        .route(
            "/workspaces/{workspace_id}/files/uploads",
            post(create_upload),
        )
        .route(
            "/workspaces/{workspace_id}/files/{asset_id}/complete",
            post(complete_upload),
        )
        .route(
            "/workspaces/{workspace_id}/files/{asset_id}",
            axum::routing::get(metadata).delete(delete_file),
        )
        .route(
            "/workspaces/{workspace_id}/files/{asset_id}/content",
            axum::routing::get(download).put(upload),
        )
}

pub(crate) fn public_file_routes() -> Router<HealthState> {
    Router::new().route(
        "/public/v1/documents/{token}/files/{asset_id}",
        axum::routing::get(public_download),
    )
}

async fn create_upload(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path(workspace): Path<Uuid>,
    Json(input): Json<CreateFileUploadInput>,
) -> Result<Response, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    let result = state
        .files
        .service
        .create_upload(
            auth.principal.user.id,
            workspace,
            idempotency_key(&headers)?,
            input,
        )
        .await
        .map_err(Problem::from)?;
    Ok((StatusCode::CREATED, Json(result)).into_response())
}

async fn upload(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, asset)): Path<(Uuid, Uuid)>,
    body: Body,
) -> Result<StatusCode, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    let token = headers
        .get("x-upload-token")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty() && value.len() <= 1024)
        .ok_or_else(|| {
            Problem::from(adoc_application::governance::GovernanceError::UploadTokenInvalid)
        })?;
    let content_length = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| Problem::from(adoc_application::governance::GovernanceError::Validation))?;
    let stream: ByteStream = Box::pin(
        body.into_data_stream()
            .map_err(|_| StorageError::Unavailable),
    );
    state
        .files
        .service
        .upload(
            auth.principal.user.id,
            workspace,
            asset,
            token,
            content_length,
            stream,
        )
        .await
        .map_err(Problem::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn complete_upload(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, asset)): Path<(Uuid, Uuid)>,
    Json(input): Json<adoc_application::operations::CompleteFileUploadInput>,
) -> Result<Json<serde_json::Value>, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    json(
        state
            .files
            .service
            .complete(
                auth.principal.user.id,
                workspace,
                asset,
                expected_revision(&headers)?,
                idempotency_key(&headers)?,
                input,
            )
            .await
            .map_err(Problem::from)?,
    )
}

async fn metadata(
    State(state): State<HealthState>,
    auth: Authenticated,
    Path((workspace, asset)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, Problem> {
    json(
        state
            .files
            .service
            .metadata(auth.principal.user.id, workspace, asset)
            .await
            .map_err(Problem::from)?,
    )
}

async fn download(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, asset)): Path<(Uuid, Uuid)>,
) -> Result<Response, Problem> {
    let meta = state
        .files
        .service
        .metadata(auth.principal.user.id, workspace, asset)
        .await
        .map_err(Problem::from)?;
    let size = u64::try_from(meta.size_bytes).map_err(|_| Problem::internal())?;
    let range = headers
        .get(header::RANGE)
        .map(|value| {
            value
                .to_str()
                .ok()
                .and_then(|value| ByteRange::parse(value, size))
                .ok_or_else(range_problem)
        })
        .transpose()?;
    let (_, stream) = state
        .files
        .service
        .download(auth.principal.user.id, workspace, asset, range)
        .await
        .map_err(Problem::from)?;
    file_response(meta, stream, range, "private, no-store")
}

async fn public_download(
    State(state): State<HealthState>,
    headers: HeaderMap,
    Path((token, asset)): Path<(String, Uuid)>,
) -> Result<Response, Problem> {
    let range_value = headers
        .get(header::RANGE)
        .map(|value| value.to_str().map_err(|_| range_problem()))
        .transpose()?;
    let (meta, stream, range) = state
        .files
        .service
        .public_download(&token, asset, range_value)
        .await
        .map_err(Problem::from)?;
    file_response(meta, stream, range, "public, max-age=60")
}

fn file_response(
    meta: adoc_application::operations::FileAsset,
    stream: ByteStream,
    range: Option<ByteRange>,
    cache_control: &'static str,
) -> Result<Response, Problem> {
    let size = u64::try_from(meta.size_bytes).map_err(|_| Problem::internal())?;
    let mut response = Body::from_stream(stream).into_response();
    *response.status_mut() = if range.is_some() {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&meta.mime_type).map_err(|_| Problem::internal())?,
    );
    response.headers_mut().insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&range.map_or(size, ByteRange::len).to_string())
            .map_err(|_| Problem::internal())?,
    );
    response
        .headers_mut()
        .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment"),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(cache_control),
    );
    if let Some(range) = range {
        response.headers_mut().insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(&format!(
                "bytes {}-{}/{}",
                range.start, range.end_inclusive, size
            ))
            .map_err(|_| Problem::internal())?,
        );
    }
    Ok(response)
}

async fn delete_file(
    State(state): State<HealthState>,
    headers: HeaderMap,
    auth: Authenticated,
    Path((workspace, asset)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Problem> {
    validate_command(&state.identity, &headers, &auth)?;
    state
        .files
        .service
        .delete(
            auth.principal.user.id,
            workspace,
            asset,
            expected_revision(&headers)?,
            idempotency_key(&headers)?,
        )
        .await
        .map_err(Problem::from)?;
    Ok(StatusCode::NO_CONTENT)
}

fn range_problem() -> Problem {
    Problem::from(adoc_application::governance::GovernanceError::Validation)
}

fn json<T: serde::Serialize>(value: T) -> Result<Json<serde_json::Value>, Problem> {
    serde_json::to_value(value)
        .map(Json)
        .map_err(|_| Problem::internal())
}

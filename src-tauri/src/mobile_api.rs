use std::{net::SocketAddr, path::PathBuf};

use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    auth,
    db::DbPool,
    documents::StorageRoot,
    error::AppError,
    master_data, mobile_devices,
    mobile_submissions::{self, MobileSubmissionAttachmentUpload, MobileSubmissionInput},
};

#[derive(Clone)]
pub struct MobileApiState {
    pub pool: DbPool,
    pub storage: StorageRoot,
    pub config: MobileApiConfig,
}

#[derive(Clone, Debug, Default)]
pub struct MobileApiConfig {}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct LookupsResponse {
    pub categories: Vec<master_data::CategoryItem>,
    pub folders: Vec<master_data::FolderItem>,
    pub offices: Vec<master_data::OfficeItem>,
}

pub fn router(pool: DbPool, storage: StorageRoot) -> Router {
    router_with_config(pool, storage, MobileApiConfig::from_env())
}

pub fn router_with_config(pool: DbPool, storage: StorageRoot, config: MobileApiConfig) -> Router {
    Router::new()
        .route("/api/mobile/health", get(health))
        .route("/api/mobile/login", post(login))
        .route("/api/mobile/logout", post(logout))
        .route("/api/mobile/lookups", get(lookups))
        .route(
            "/api/mobile/submissions",
            get(list_submissions).post(create_submission),
        )
        .route("/api/mobile/submissions/{id}", get(get_submission))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(MobileApiState {
            pool,
            storage,
            config,
        })
}

pub async fn serve(pool: DbPool, storage: StorageRoot, addr: &str) -> Result<(), String> {
    let addr: SocketAddr = addr.parse::<SocketAddr>().map_err(|err| err.to_string())?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|err| err.to_string())?;
    println!("Mobile API started on {addr}");
    axum::serve(listener, router(pool, storage))
        .await
        .map_err(|err| err.to_string())
}

impl MobileApiConfig {
    pub fn from_env() -> Self {
        Self {}
    }
}

#[derive(Debug, Serialize)]
pub struct MobileApiSetup {
    pub enabled: bool,
    pub bind_addr: String,
    pub local_ip: String,
    pub setup_url: String,
    pub device_token_required: bool,
}

pub fn setup_info() -> MobileApiSetup {
    let bind_addr =
        std::env::var("OVPSPEE_MOBILE_API_ADDR").unwrap_or_else(|_| "0.0.0.0:1421".to_owned());
    let local_ip = local_ipv4().unwrap_or_else(|| "127.0.0.1".to_owned());
    let port = bind_addr.rsplit(':').next().unwrap_or("1421");
    let hub_url = format!("http://{local_ip}:{port}");
    MobileApiSetup {
        enabled: std::env::var("OVPSPEE_MOBILE_API_ENABLED").as_deref() == Ok("1"),
        bind_addr,
        local_ip,
        setup_url: format!("ovpspee://setup?hub={}", encode_setup_value(&hub_url)),
        device_token_required: true,
    }
}

fn encode_setup_value(value: &str) -> String {
    value.replace(':', "%3A").replace('/', "%2F")
}

fn local_ipv4() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip().to_string())
}

async fn health(
    State(state): State<MobileApiState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    require_mobile_device(&headers, &state).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn login(
    State(state): State<MobileApiState>,
    headers: HeaderMap,
    Json(input): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let device_id = require_mobile_device(&headers, &state).await?;
    if input
        .device_id
        .as_deref()
        .is_some_and(|body_device_id| body_device_id != device_id)
    {
        return Err(ApiError(AppError::Unauthorized));
    }
    let session = auth::authenticate_user(&state.pool, &input.username, &input.password).await?;
    if session.role != "Secretary" {
        return Err(ApiError(AppError::Unauthorized));
    }
    Ok(Json(session))
}

async fn logout(
    State(state): State<MobileApiState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = auth_required(&headers, &state).await?;
    auth::logout_session(&state.pool, &session_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn lookups(
    State(state): State<MobileApiState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = auth_required(&headers, &state).await?;
    let categories = master_data::list_categories(&state.pool, &session_id, Some(false)).await?;
    let folders = master_data::list_folders(&state.pool, &session_id, None, Some(false)).await?;
    let offices = master_data::list_offices(&state.pool, &session_id, Some(false)).await?;
    Ok(Json(LookupsResponse {
        categories,
        folders,
        offices,
    }))
}

async fn list_submissions(
    State(state): State<MobileApiState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = auth_required(&headers, &state).await?;
    let rows = mobile_submissions::list_mobile_submissions(
        &state.pool,
        &session_id,
        Some("Pending".into()),
        None,
        None,
        None,
    )
    .await?;
    Ok(Json(rows))
}

async fn get_submission(
    State(state): State<MobileApiState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = auth_required(&headers, &state).await?;
    let detail = mobile_submissions::get_mobile_submission(&state.pool, &session_id, id).await?;
    Ok(Json(detail))
}

async fn create_submission(
    State(state): State<MobileApiState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = auth_required(&headers, &state).await?;
    let mut input: Option<MobileSubmissionInput> = None;
    let mut uploads = Vec::new();
    let temp_dir = state.storage.resolve_checked("mobile-submissions/tmp")?;

    while let Some(field) = multipart.next_field().await.map_err(|_| {
        ApiError(AppError::Validation(
            "Invalid mobile upload payload.".into(),
        ))
    })? {
        let name = field.name().unwrap_or_default().to_owned();
        if name == "metadata" {
            let text = field
                .text()
                .await
                .map_err(|_| ApiError(AppError::Validation("Invalid mobile metadata.".into())))?;
            input = Some(serde_json::from_str::<MobileSubmissionInput>(&text)?);
            continue;
        }
        if name == "files" {
            let file_name = field.file_name().unwrap_or("mobile-upload.bin").to_owned();
            let safe_file_name = sanitize_file_name(&file_name);
            let bytes = field.bytes().await.map_err(|_| {
                ApiError(AppError::Validation("Invalid mobile upload file.".into()))
            })?;
            let temp_path = temp_dir.join(format!("{}-{safe_file_name}", uuid::Uuid::new_v4()));
            tokio::fs::write(&temp_path, bytes).await?;
            uploads.push(MobileSubmissionAttachmentUpload {
                source_path: temp_path.to_string_lossy().into_owned(),
                original_file_name: file_name,
            });
        }
    }

    let id = mobile_submissions::create_mobile_submission(
        &state.pool,
        &state.storage,
        &session_id,
        input.ok_or_else(|| ApiError(AppError::Validation("Metadata is required.".into())))?,
        uploads,
    )
    .await?;
    Ok(Json(serde_json::json!({ "mobile_submission_id": id })))
}

async fn auth_required(headers: &HeaderMap, state: &MobileApiState) -> Result<String, ApiError> {
    require_mobile_device(headers, state).await?;
    let session_id = bearer_session(headers)?;
    let session = auth::validate_session(&state.pool, &session_id).await?;
    if session.role != "Secretary" {
        return Err(ApiError(AppError::Unauthorized));
    }
    Ok(session_id)
}

async fn require_mobile_device(
    headers: &HeaderMap,
    state: &MobileApiState,
) -> Result<String, ApiError> {
    let device_id = headers
        .get("x-ovpspee-device-id")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError(AppError::Unauthorized))?;
    let token = headers
        .get("x-ovpspee-device-token")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError(AppError::Unauthorized))?;
    mobile_devices::validate_mobile_device(&state.pool, device_id, token).await?;
    Ok(device_id.to_owned())
}

fn bearer_session(headers: &HeaderMap) -> Result<String, ApiError> {
    let value = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError(AppError::Unauthorized))?;
    value
        .strip_prefix("Bearer ")
        .map(str::to_owned)
        .ok_or(ApiError(AppError::Unauthorized))
}

fn sanitize_file_name(file_name: &str) -> String {
    PathBuf::from(file_name)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("mobile-upload.bin")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

pub struct ApiError(AppError);

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(value: sqlx::Error) -> Self {
        Self(AppError::Database(value))
    }
}

impl From<std::io::Error> for ApiError {
    fn from(value: std::io::Error) -> Self {
        Self(AppError::Io(value))
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(_: serde_json::Error) -> Self {
        Self(AppError::Validation("Invalid mobile metadata.".into()))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.0 {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) | AppError::Conflict(_) | AppError::Duplicate(_) => {
                StatusCode::BAD_REQUEST
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let safe = if status == StatusCode::INTERNAL_SERVER_ERROR {
            "Something went wrong. Please try again.".to_owned()
        } else {
            self.0.to_string()
        };
        (status, Json(ApiErrorBody { error: safe })).into_response()
    }
}

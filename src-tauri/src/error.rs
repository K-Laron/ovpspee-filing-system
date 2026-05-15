use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("ERR_UNAUTHORIZED")]
    Unauthorized,
    #[error("ERR_VALIDATION: {0}")]
    Validation(String),
    #[error("ERR_DUPLICATE: {0}")]
    Duplicate(String),
    #[error("ERR_NOT_FOUND: {0}")]
    NotFound(String),
    #[error("ERR_SYSTEM_RECORD")]
    SystemRecord,
    #[error("ERR_CONFLICT: {0}")]
    Conflict(String),
    #[error("ERR_DB: {0}")]
    Database(#[from] sqlx::Error),
    #[error("ERR_DB: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("ERR_IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("ERR_VALIDATION: {0}")]
    Json(#[from] serde_json::Error),
}

pub type AppResult<T> = Result<T, AppError>;

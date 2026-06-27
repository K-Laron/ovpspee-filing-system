use tauri::State;

use crate::auth;
use crate::auth::SessionPayload;
use crate::db::DbState;
use crate::error::AppError;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub async fn first_run_check(db: State<'_, DbState>) -> CmdResult<bool> {
    auth::first_run_required(&db.pool).await
}

#[tauri::command]
pub async fn first_run_setup(
    db: State<'_, DbState>,
    first_name: String,
    last_name: String,
    username: String,
    password: String,
) -> CmdResult<()> {
    auth::create_first_admin(&db.pool, &first_name, &last_name, &username, &password).await
}

#[tauri::command]
pub async fn login(
    db: State<'_, DbState>,
    username: String,
    password: String,
) -> CmdResult<SessionPayload> {
    auth::authenticate_user(&db.pool, &username, &password).await
}

#[tauri::command]
pub async fn logout(db: State<'_, DbState>, session_id: String) -> CmdResult<()> {
    auth::logout_session(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn validate_session(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<SessionPayload> {
    auth::validate_session(&db.pool, &session_id).await
}

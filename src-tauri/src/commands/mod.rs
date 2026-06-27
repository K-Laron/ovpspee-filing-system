pub mod admin_cmds;
pub mod auth_cmds;
pub mod device_cmds;
pub mod document_cmds;
pub mod mobile_cmds;
pub mod public_cmds;
pub mod scan_cmds;


use tauri::{AppHandle, Manager};

use crate::backup::BackupRuntime;
use crate::documents::StorageRoot;
use crate::error::AppError;

type CmdResult<T> = Result<T, AppError>;

fn storage_root(app: &AppHandle) -> CmdResult<StorageRoot> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Validation(e.to_string()))?
        .join("storage");
    StorageRoot::new(root)
}

fn backup_runtime(app: &AppHandle) -> CmdResult<BackupRuntime> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let db_path = app_data_dir.join("filing_system.db");
    let storage = storage_root(app)?;
    Ok(BackupRuntime::new(app_data_dir, db_path, storage))
}

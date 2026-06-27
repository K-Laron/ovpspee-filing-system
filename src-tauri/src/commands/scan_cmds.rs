use tauri::{AppHandle, State};

use crate::db::DbState;
use crate::documents::DocumentInput;
use crate::scan_intake::{self, ScanIntakeItem, ScanIntakePreviewPage};

use super::{storage_root, CmdResult};

#[tauri::command]
pub async fn import_scan_files(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    source_paths: Vec<String>,
) -> CmdResult<Vec<i64>> {
    let storage = storage_root(&app)?;
    scan_intake::import_scan_files(&db.pool, &storage, &session_id, source_paths).await
}

#[tauri::command]
pub async fn list_scan_intake(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<ScanIntakeItem>> {
    scan_intake::list_scan_intake(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn get_scan_intake(
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_id: i64,
) -> CmdResult<ScanIntakeItem> {
    scan_intake::get_scan_intake(&db.pool, &session_id, scan_intake_id).await
}

#[tauri::command]
pub async fn get_scan_intake_preview_page(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_id: i64,
    page_number: Option<i64>,
) -> CmdResult<ScanIntakePreviewPage> {
    let storage = storage_root(&app)?;
    scan_intake::get_scan_intake_preview_page(
        &db.pool, &storage, &session_id, scan_intake_id, page_number,
    ).await
}

#[tauri::command]
pub async fn update_scan_intake_notes(
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_id: i64,
    notes: Option<String>,
) -> CmdResult<()> {
    scan_intake::update_scan_intake_notes(&db.pool, &session_id, scan_intake_id, notes).await
}

#[tauri::command]
pub async fn remove_scan_intake(
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_id: i64,
) -> CmdResult<()> {
    scan_intake::remove_scan_intake(&db.pool, &session_id, scan_intake_id).await
}

#[tauri::command]
pub async fn file_scan_as_document(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_ids: Vec<i64>,
    document_name: String,
    category_id: i64,
    folder_id: Option<i64>,
    office_id: Option<i64>,
    date_received: String,
    remarks: Option<String>,
    status: String,
) -> CmdResult<i64> {
    let storage = storage_root(&app)?;
    scan_intake::file_scan_as_document(
        &db.pool, &storage, &session_id, scan_intake_ids,
        DocumentInput { document_name, category_id, folder_id, office_id, date_received, remarks, status },
    ).await
}

#[tauri::command]
pub async fn attach_scan_to_document(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_ids: Vec<i64>,
    document_id: i64,
) -> CmdResult<Vec<i64>> {
    let storage = storage_root(&app)?;
    scan_intake::attach_scan_to_document(
        &db.pool, &storage, &session_id, scan_intake_ids, document_id,
    ).await
}

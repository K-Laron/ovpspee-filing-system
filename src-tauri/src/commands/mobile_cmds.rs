use tauri::{AppHandle, State};

use crate::db::DbState;
use crate::mobile_api::{self, MobileApiSetup};
use crate::mobile_devices::{self, CreatedMobileDevice, MobileDeviceItem};
use crate::mobile_submissions::{
    self, MobileSubmissionAttachmentPreviewPage, MobileSubmissionDetail, MobileSubmissionItem,
};

use super::{storage_root, CmdResult};

#[tauri::command]
pub async fn list_mobile_submissions(
    db: State<'_, DbState>,
    session_id: String,
    review_status: Option<String>,
    search: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> CmdResult<Vec<MobileSubmissionItem>> {
    mobile_submissions::list_mobile_submissions(
        &db.pool, &session_id, review_status, search, date_from, date_to,
    ).await
}

#[tauri::command]
pub async fn get_mobile_api_setup() -> CmdResult<MobileApiSetup> {
    Ok(mobile_api::setup_info())
}

#[tauri::command]
pub async fn create_mobile_device(
    db: State<'_, DbState>,
    session_id: String,
    device_name: String,
) -> CmdResult<CreatedMobileDevice> {
    mobile_devices::create_mobile_device(&db.pool, &session_id, &device_name).await
}

#[tauri::command]
pub async fn list_mobile_devices(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<MobileDeviceItem>> {
    mobile_devices::list_mobile_devices(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn revoke_mobile_device(
    db: State<'_, DbState>,
    session_id: String,
    device_id: String,
) -> CmdResult<()> {
    mobile_devices::revoke_mobile_device(&db.pool, &session_id, &device_id).await
}

#[tauri::command]
pub async fn get_mobile_submission(
    db: State<'_, DbState>,
    session_id: String,
    mobile_submission_id: i64,
) -> CmdResult<MobileSubmissionDetail> {
    mobile_submissions::get_mobile_submission(&db.pool, &session_id, mobile_submission_id).await
}

#[tauri::command]
pub async fn get_mobile_submission_attachment_preview_page(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    mobile_submission_attachment_id: i64,
    page_number: Option<i64>,
) -> CmdResult<MobileSubmissionAttachmentPreviewPage> {
    let storage = storage_root(&app)?;
    mobile_submissions::get_mobile_submission_attachment_preview_page(
        &db.pool, &storage, &session_id, mobile_submission_attachment_id, page_number,
    ).await
}

#[tauri::command]
pub async fn approve_mobile_submission(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    mobile_submission_id: i64,
    review_notes: Option<String>,
) -> CmdResult<i64> {
    let storage = storage_root(&app)?;
    mobile_submissions::approve_mobile_submission(
        &db.pool, &storage, &session_id, mobile_submission_id, review_notes,
    ).await
}

#[tauri::command]
pub async fn reject_mobile_submission(
    db: State<'_, DbState>,
    session_id: String,
    mobile_submission_id: i64,
    rejection_reason: String,
) -> CmdResult<()> {
    mobile_submissions::reject_mobile_submission(
        &db.pool, &session_id, mobile_submission_id, rejection_reason,
    ).await
}

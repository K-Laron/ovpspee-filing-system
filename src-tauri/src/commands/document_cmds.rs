use tauri::{AppHandle, State};

use crate::db::DbState;
use crate::documents::{
    self, AttachmentInput, AttachmentPreviewInfo, AttachmentPreviewPage, DocumentDetail,
    DocumentInput, DocumentItem, DocumentListFilter, DocumentListPage,
};

use super::{storage_root, CmdResult};

#[tauri::command]
pub async fn create_document(
    db: State<'_, DbState>,
    session_id: String,
    document_name: String,
    category_id: i64,
    folder_id: Option<i64>,
    office_id: Option<i64>,
    date_received: String,
    remarks: Option<String>,
    status: String,
) -> CmdResult<i64> {
    documents::create_document(
        &db.pool, &session_id,
        DocumentInput { document_name, category_id, folder_id, office_id, date_received, remarks, status },
    ).await
}

#[tauri::command]
pub async fn update_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    document_name: String,
    category_id: i64,
    folder_id: Option<i64>,
    office_id: Option<i64>,
    date_received: String,
    remarks: Option<String>,
    status: String,
) -> CmdResult<()> {
    documents::update_document(
        &db.pool, &session_id, document_id,
        DocumentInput { document_name, category_id, folder_id, office_id, date_received, remarks, status },
    ).await
}

#[tauri::command]
pub async fn move_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    category_id: i64,
    folder_id: Option<i64>,
) -> CmdResult<()> {
    documents::move_document(&db.pool, &session_id, document_id, category_id, folder_id).await
}

#[tauri::command]
pub async fn set_document_status(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    status: String,
) -> CmdResult<()> {
    documents::set_document_status(&db.pool, &session_id, document_id, status).await
}

#[tauri::command]
pub async fn set_document_hidden(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    is_hidden: bool,
) -> CmdResult<()> {
    documents::set_document_hidden(&db.pool, &session_id, document_id, is_hidden).await
}

#[tauri::command]
pub async fn trash_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> CmdResult<()> {
    documents::trash_document(&db.pool, &session_id, document_id).await
}

#[tauri::command]
pub async fn restore_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> CmdResult<()> {
    documents::restore_document(&db.pool, &session_id, document_id).await
}

#[tauri::command]
pub async fn list_trash_documents(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<DocumentItem>> {
    documents::list_trash_documents(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn purge_document(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> CmdResult<()> {
    let storage = storage_root(&app)?;
    documents::purge_document(&db.pool, &storage, &session_id, document_id).await
}

#[tauri::command]
pub async fn empty_trash(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<i64> {
    let storage = storage_root(&app)?;
    documents::empty_trash(&db.pool, &storage, &session_id).await
}

#[tauri::command]
pub async fn list_documents(
    db: State<'_, DbState>,
    session_id: String,
    search: Option<String>,
    category_id: Option<i64>,
    folder_id: Option<i64>,
    office_id: Option<i64>,
    date_from: Option<String>,
    date_to: Option<String>,
    status: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> CmdResult<DocumentListPage> {
    documents::list_documents(
        &db.pool, &session_id,
        DocumentListFilter { search, category_id, folder_id, office_id, date_from, date_to, status, limit, offset },
    ).await
}

#[tauri::command]
pub async fn get_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> CmdResult<DocumentDetail> {
    documents::get_document(&db.pool, &session_id, document_id).await
}

#[tauri::command]
pub async fn add_attachment(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    source_path: String,
    sort_order: Option<i64>,
) -> CmdResult<i64> {
    let storage = storage_root(&app)?;
    documents::add_attachment(
        &db.pool, &storage, &session_id, document_id,
        AttachmentInput { source_path, sort_order },
    ).await
}

#[tauri::command]
pub async fn remove_attachment(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    attachment_id: i64,
) -> CmdResult<()> {
    let storage = storage_root(&app)?;
    documents::remove_attachment(&db.pool, &storage, &session_id, attachment_id).await
}

#[tauri::command]
pub async fn reorder_attachments(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    attachment_ids: Vec<i64>,
) -> CmdResult<()> {
    documents::reorder_attachments(&db.pool, &session_id, document_id, attachment_ids).await
}

#[tauri::command]
pub async fn get_attachment_file_path(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: Option<String>,
    attachment_id: i64,
) -> CmdResult<String> {
    let storage = storage_root(&app)?;
    documents::get_attachment_file_path(&db.pool, &storage, session_id.as_deref(), attachment_id).await
}

#[tauri::command]
pub async fn get_attachment_preview_info(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: Option<String>,
    attachment_id: i64,
) -> CmdResult<AttachmentPreviewInfo> {
    let storage = storage_root(&app)?;
    documents::get_attachment_preview_info(&db.pool, &storage, session_id.as_deref(), attachment_id).await
}

#[tauri::command]
pub async fn get_attachment_preview_page(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: Option<String>,
    attachment_id: i64,
    page_number: Option<i64>,
) -> CmdResult<AttachmentPreviewPage> {
    let storage = storage_root(&app)?;
    documents::get_attachment_preview_page(
        &db.pool, &storage, session_id.as_deref(), attachment_id, page_number,
    ).await
}

#[tauri::command]
pub async fn export_document_pdf(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: Option<String>,
    document_id: i64,
    output_path: String,
) -> CmdResult<String> {
    let storage = storage_root(&app)?;
    documents::export_document_pdf(&db.pool, &storage, session_id.as_deref(), document_id, &output_path).await
}

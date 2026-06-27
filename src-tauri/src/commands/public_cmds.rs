use tauri::{AppHandle, State};

use crate::db::DbState;
use crate::devices::PrinterDevice;
use crate::documents::{self, DocumentDetail, DocumentListFilter, DocumentListPage};
use crate::master_data::{CategoryItem, FolderItem};
use crate::printing::{self, PrintOptions, PrintResult};

use super::{storage_root, CmdResult};

#[tauri::command]
pub async fn list_public_categories(db: State<'_, DbState>) -> CmdResult<Vec<CategoryItem>> {
    documents::list_public_categories(&db.pool).await
}

#[tauri::command]
pub async fn list_public_folders(
    db: State<'_, DbState>,
    category_id: i64,
) -> CmdResult<Vec<FolderItem>> {
    documents::list_public_folders(&db.pool, category_id).await
}

#[tauri::command]
pub async fn list_public_documents(
    db: State<'_, DbState>,
    search: Option<String>,
    category_id: Option<i64>,
    folder_id: Option<i64>,
    office_id: Option<i64>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> CmdResult<DocumentListPage> {
    documents::list_public_documents(
        &db.pool,
        DocumentListFilter { search, category_id, folder_id, office_id, date_from, date_to, status: None, limit: None, offset: None },
    ).await
}

#[tauri::command]
pub async fn get_public_document(
    db: State<'_, DbState>,
    document_id: i64,
) -> CmdResult<DocumentDetail> {
    documents::get_public_document(&db.pool, document_id).await
}

#[tauri::command]
pub async fn list_document_offices(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<crate::master_data::OfficeItem>> {
    documents::list_document_offices(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn list_print_printers(
    db: State<'_, DbState>,
    session_id: Option<String>,
) -> CmdResult<Vec<PrinterDevice>> {
    printing::list_print_printers(&db.pool, session_id.as_deref()).await
}

#[tauri::command]
pub async fn print_document_pdf(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: Option<String>,
    document_id: i64,
    printer_id: String,
    copies: i64,
) -> CmdResult<PrintResult> {
    let storage = storage_root(&app)?;
    printing::print_document_pdf(&db.pool, &storage, session_id.as_deref(), document_id, &printer_id, PrintOptions { copies }).await
}

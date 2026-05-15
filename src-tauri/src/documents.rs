use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

use chrono::{NaiveDate, SecondsFormat, Utc};
use serde::Serialize;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::{require_admin_role, require_session, write_audit_log},
    db::DbPool,
    error::{AppError, AppResult},
    master_data::OfficeItem,
    master_data::{CategoryItem, FolderItem},
};

pub(crate) const MAX_ATTACHMENT_BYTES: u64 = 1_073_741_824;

const ALLOWED_EXTENSIONS: &[&str] = &[
    "pdf", "doc", "docx", "xls", "xlsx", "jpg", "jpeg", "png", "tif", "tiff", "txt",
];

#[derive(Debug, Clone)]
pub struct StorageRoot {
    base: PathBuf,
}

impl StorageRoot {
    pub fn new(base: PathBuf) -> AppResult<Self> {
        fs::create_dir_all(base.join("documents"))?;
        fs::create_dir_all(base.join("intake"))?;
        let base = base.canonicalize()?;
        Ok(Self { base })
    }

    pub fn documents_dir(&self) -> &Path {
        &self.base
    }

    pub fn resolve_relative(&self, relative: &str) -> PathBuf {
        self.base
            .join(relative.replace('/', std::path::MAIN_SEPARATOR_STR))
    }

    pub fn resolve_checked(&self, relative: &str) -> AppResult<PathBuf> {
        let path = Path::new(relative);
        if path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, Component::ParentDir | Component::Prefix(_)))
        {
            return Err(AppError::Validation("Invalid attachment path.".into()));
        }
        let full = self.resolve_relative(relative);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        let parent = full
            .parent()
            .ok_or_else(|| AppError::Validation("Invalid attachment path.".into()))?
            .canonicalize()?;
        if !parent.starts_with(&self.base) {
            return Err(AppError::Validation("Invalid attachment path.".into()));
        }
        Ok(full)
    }
}

#[derive(Debug, Clone)]
pub struct DocumentInput {
    pub document_name: String,
    pub category_id: i64,
    pub folder_id: Option<i64>,
    pub office_id: Option<i64>,
    pub date_received: String,
    pub remarks: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentListFilter {
    pub search: Option<String>,
    pub category_id: Option<i64>,
    pub folder_id: Option<i64>,
    pub office_id: Option<i64>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttachmentInput {
    pub source_path: String,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentItem {
    pub document_id: i64,
    pub document_name: String,
    pub category_id: i64,
    pub category_name: String,
    pub folder_id: Option<i64>,
    pub folder_name: Option<String>,
    pub office_id: Option<i64>,
    pub office_name: Option<String>,
    pub date_received: String,
    pub date_added: String,
    pub remarks: Option<String>,
    pub status: String,
    pub is_hidden: bool,
    pub is_trashed: bool,
    pub attachment_count: i64,
    pub created_by: i64,
    pub created_by_name: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachmentItem {
    pub attachment_id: i64,
    pub document_id: i64,
    pub original_file_name: String,
    pub stored_relative_path: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentDetail {
    pub document: DocumentItem,
    pub attachments: Vec<AttachmentItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachmentPreviewInfo {
    pub attachment_id: i64,
    pub document_id: i64,
    pub original_file_name: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub preview_kind: String,
    pub page_count: Option<i64>,
    pub file_exists: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachmentPreviewPage {
    pub info: AttachmentPreviewInfo,
    pub page_number: i64,
    pub file_path: Option<String>,
}

pub async fn create_document(
    pool: &DbPool,
    session_id: &str,
    input: DocumentInput,
) -> AppResult<i64> {
    let session = require_document_editor(pool, session_id).await?;
    let input = validate_document_input(pool, input).await?;
    let is_hidden = if input.status == "Confidential" { 1 } else { 0 };
    let now = now_text();

    let result = sqlx::query!(
        "INSERT INTO document
         (document_name, category_id, folder_id, office_id, date_received, date_added, remarks, status, is_hidden, created_by, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        input.document_name,
        input.category_id,
        input.folder_id,
        input.office_id,
        input.date_received,
        now,
        input.remarks,
        input.status,
        is_hidden,
        session.user_id,
        now
    )
    .execute(pool)
    .await?;
    let id = result.last_insert_rowid();
    refresh_document_fts(pool, id).await?;
    write_audit_log(
        pool,
        "INSERT",
        Some("document"),
        Some(id),
        "Created document",
        Some(session.user_id),
    )
    .await?;
    Ok(id)
}

pub async fn set_document_hidden(
    pool: &DbPool,
    session_id: &str,
    document_id: i64,
    is_hidden: bool,
) -> AppResult<()> {
    let session = require_document_editor(pool, session_id).await?;
    ensure_document_exists(pool, document_id).await?;
    let hidden = if is_hidden { 1_i64 } else { 0_i64 };
    let now = now_text();
    sqlx::query!(
        "UPDATE document SET is_hidden = ?, updated_at = ? WHERE document_id = ?",
        hidden,
        now,
        document_id
    )
    .execute(pool)
    .await?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("document"),
        Some(document_id),
        if is_hidden {
            "Hid document"
        } else {
            "Unhid document"
        },
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn trash_document(pool: &DbPool, session_id: &str, document_id: i64) -> AppResult<()> {
    let session = require_document_editor(pool, session_id).await?;
    let current = sqlx::query!(
        "SELECT category_id AS \"category_id!: i64\", folder_id, is_trashed AS \"is_trashed!: i64\"
         FROM document WHERE document_id = ?",
        document_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found.".into()))?;
    if current.is_trashed == 1 {
        return Ok(());
    }
    let trash_category_id = trash_category_id(pool).await?;
    let now = now_text();
    sqlx::query!(
        "UPDATE document
         SET category_id = ?, folder_id = NULL, is_trashed = 1, trashed_at = ?,
             original_category_id = ?, original_folder_id = ?, updated_at = ?
         WHERE document_id = ?",
        trash_category_id,
        now,
        current.category_id,
        current.folder_id,
        now,
        document_id
    )
    .execute(pool)
    .await?;
    refresh_document_fts(pool, document_id).await?;
    write_audit_log(
        pool,
        "TRASH",
        Some("document"),
        Some(document_id),
        "Moved document to trash",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn restore_document(pool: &DbPool, session_id: &str, document_id: i64) -> AppResult<()> {
    let session = require_document_editor(pool, session_id).await?;
    let current = sqlx::query!(
        "SELECT is_trashed AS \"is_trashed!: i64\", original_category_id, original_folder_id
         FROM document WHERE document_id = ?",
        document_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found.".into()))?;
    if current.is_trashed != 1 {
        return Err(AppError::Validation("Document is not in trash.".into()));
    }
    let category_id = current
        .original_category_id
        .ok_or_else(|| AppError::Conflict("Original category is missing.".into()))?;
    let category = sqlx::query!(
        "SELECT is_active AS \"is_active!: i64\", is_system AS \"is_system!: i64\"
         FROM category WHERE category_id = ?",
        category_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Conflict("Original category no longer exists.".into()))?;
    if category.is_active != 1 || category.is_system == 1 {
        return Err(AppError::Conflict(
            "Original category is inactive or unavailable.".into(),
        ));
    }
    let folder_id = if let Some(folder_id) = current.original_folder_id {
        let folder = sqlx::query!(
            "SELECT category_id AS \"category_id!: i64\", is_active AS \"is_active!: i64\"
             FROM folder WHERE folder_id = ?",
            folder_id
        )
        .fetch_optional(pool)
        .await?;
        match folder {
            Some(folder) if folder.category_id == category_id && folder.is_active == 1 => {
                Some(folder_id)
            }
            _ => None,
        }
    } else {
        None
    };
    let now = now_text();
    sqlx::query!(
        "UPDATE document
         SET category_id = ?, folder_id = ?, is_trashed = 0, trashed_at = NULL,
             original_category_id = NULL, original_folder_id = NULL, updated_at = ?
         WHERE document_id = ?",
        category_id,
        folder_id,
        now,
        document_id
    )
    .execute(pool)
    .await?;
    refresh_document_fts(pool, document_id).await?;
    write_audit_log(
        pool,
        "RESTORE",
        Some("document"),
        Some(document_id),
        "Restored document",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn list_trash_documents(pool: &DbPool, session_id: &str) -> AppResult<Vec<DocumentItem>> {
    require_trash_viewer(pool, session_id).await?;
    fetch_documents(pool, DocumentListFilter::default(), false, true).await
}

pub async fn purge_document(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    document_id: i64,
) -> AppResult<()> {
    let session = require_admin(pool, session_id).await?;
    purge_document_internal(pool, storage, document_id, session.user_id).await
}

pub async fn empty_trash(pool: &DbPool, storage: &StorageRoot, session_id: &str) -> AppResult<i64> {
    let session = require_admin(pool, session_id).await?;
    let rows = sqlx::query!(
        "SELECT document_id AS \"document_id!: i64\" FROM document WHERE is_trashed = 1"
    )
    .fetch_all(pool)
    .await?;
    let count = rows.len() as i64;
    for row in rows {
        purge_document_internal(pool, storage, row.document_id, session.user_id).await?;
    }
    write_audit_log(
        pool,
        "PURGE",
        Some("document"),
        None,
        "Emptied trash",
        Some(session.user_id),
    )
    .await?;
    Ok(count)
}

pub async fn update_document(
    pool: &DbPool,
    session_id: &str,
    document_id: i64,
    input: DocumentInput,
) -> AppResult<()> {
    let session = require_document_editor(pool, session_id).await?;
    ensure_document_exists(pool, document_id).await?;
    let input = validate_document_input(pool, input).await?;
    let now = now_text();
    sqlx::query!(
        "UPDATE document
         SET document_name = ?, category_id = ?, folder_id = ?, office_id = ?, date_received = ?,
             remarks = ?, status = ?, is_hidden = CASE WHEN ? = 'Confidential' THEN 1 ELSE is_hidden END, updated_at = ?
         WHERE document_id = ?",
        input.document_name,
        input.category_id,
        input.folder_id,
        input.office_id,
        input.date_received,
        input.remarks,
        input.status,
        input.status,
        now,
        document_id
    )
    .execute(pool)
    .await?;
    refresh_document_fts(pool, document_id).await?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("document"),
        Some(document_id),
        "Updated document",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn move_document(
    pool: &DbPool,
    session_id: &str,
    document_id: i64,
    category_id: i64,
    folder_id: Option<i64>,
) -> AppResult<()> {
    let session = require_document_editor(pool, session_id).await?;
    let current = sqlx::query!(
        "SELECT category_id AS \"category_id!: i64\", folder_id, is_trashed AS \"is_trashed!: i64\"
         FROM document WHERE document_id = ?",
        document_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found.".into()))?;
    if current.is_trashed == 1 {
        return Err(AppError::Validation(
            "Restore document before moving.".into(),
        ));
    }
    validate_document_location(pool, category_id, folder_id).await?;
    let now = now_text();
    sqlx::query!(
        "UPDATE document SET category_id = ?, folder_id = ?, updated_at = ? WHERE document_id = ?",
        category_id,
        folder_id,
        now,
        document_id
    )
    .execute(pool)
    .await?;
    refresh_document_fts(pool, document_id).await?;
    write_audit_log(
        pool,
        "MOVE",
        Some("document"),
        Some(document_id),
        &format!(
            "Moved document from category {} folder {:?} to category {} folder {:?}",
            current.category_id, current.folder_id, category_id, folder_id
        ),
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn set_document_status(
    pool: &DbPool,
    session_id: &str,
    document_id: i64,
    status: String,
) -> AppResult<()> {
    let session = require_document_editor(pool, session_id).await?;
    validate_status(&status)?;
    let current = sqlx::query!(
        "SELECT status, is_trashed AS \"is_trashed!: i64\" FROM document WHERE document_id = ?",
        document_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found.".into()))?;
    if current.is_trashed == 1 {
        return Err(AppError::Validation(
            "Restore document before changing status.".into(),
        ));
    }
    let now = now_text();
    sqlx::query!(
        "UPDATE document
         SET status = ?, is_hidden = CASE WHEN ? = 'Confidential' THEN 1 ELSE is_hidden END, updated_at = ?
         WHERE document_id = ?",
        status,
        status,
        now,
        document_id
    )
    .execute(pool)
    .await?;
    refresh_document_fts(pool, document_id).await?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("document"),
        Some(document_id),
        &format!(
            "Changed document status from {} to {}",
            current.status, status
        ),
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn list_documents(
    pool: &DbPool,
    session_id: &str,
    filter: DocumentListFilter,
) -> AppResult<Vec<DocumentItem>> {
    require_document_editor(pool, session_id).await?;
    fetch_documents(pool, filter, false, false).await
}

pub async fn list_public_documents(
    pool: &DbPool,
    filter: DocumentListFilter,
) -> AppResult<Vec<DocumentItem>> {
    fetch_documents(pool, filter, true, false).await
}

pub async fn get_document(
    pool: &DbPool,
    session_id: &str,
    document_id: i64,
) -> AppResult<DocumentDetail> {
    require_document_editor(pool, session_id).await?;
    get_document_internal(pool, document_id, false).await
}

pub async fn get_public_document(pool: &DbPool, document_id: i64) -> AppResult<DocumentDetail> {
    get_document_internal(pool, document_id, true).await
}

pub async fn add_attachment(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    document_id: i64,
    input: AttachmentInput,
) -> AppResult<i64> {
    let session = require_document_editor(pool, session_id).await?;
    ensure_document_exists(pool, document_id).await?;
    let source = validate_source_file(&input.source_path)?;
    let original_file_name = source
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| AppError::Validation("File name is required.".into()))?
        .to_owned();
    let ext = source
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let file_size = fs::metadata(&source)?.len();
    if file_size > MAX_ATTACHMENT_BYTES {
        return Err(AppError::Validation(
            "Attachment exceeds 1 GB maximum.".into(),
        ));
    }
    validate_extension(&ext)?;
    validate_magic(&source, &ext)?;
    let relative = format!("documents/{document_id}/{}.{}", Uuid::new_v4(), ext);
    let destination = storage.resolve_checked(&relative)?;
    fs::copy(&source, &destination)?;
    let mime_type = mime_for_extension(&ext).to_owned();
    let sort_order = input.sort_order.unwrap_or_else(|| 1);
    let file_size_i64 = file_size as i64;

    let result = sqlx::query!(
        "INSERT INTO attachment
         (document_id, original_file_name, stored_relative_path, mime_type, file_size_bytes, sort_order)
         VALUES (?, ?, ?, ?, ?, ?)",
        document_id,
        original_file_name,
        relative,
        mime_type,
        file_size_i64,
        sort_order
    )
    .execute(pool)
    .await?;
    let id = result.last_insert_rowid();
    write_audit_log(
        pool,
        "INSERT",
        Some("attachment"),
        Some(id),
        "Added document attachment",
        Some(session.user_id),
    )
    .await?;
    Ok(id)
}

pub async fn remove_attachment(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    attachment_id: i64,
) -> AppResult<()> {
    let session = require_document_editor(pool, session_id).await?;
    let row = sqlx::query!(
        "SELECT attachment_id AS \"attachment_id!: i64\", stored_relative_path
         FROM attachment WHERE attachment_id = ?",
        attachment_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Attachment not found.".into()))?;
    sqlx::query!(
        "DELETE FROM attachment WHERE attachment_id = ?",
        attachment_id
    )
    .execute(pool)
    .await?;
    let path = storage.resolve_checked(&row.stored_relative_path)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    write_audit_log(
        pool,
        "DELETE",
        Some("attachment"),
        Some(attachment_id),
        "Removed document attachment",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn reorder_attachments(
    pool: &DbPool,
    session_id: &str,
    document_id: i64,
    attachment_ids: Vec<i64>,
) -> AppResult<()> {
    let session = require_document_editor(pool, session_id).await?;
    ensure_document_exists(pool, document_id).await?;
    let mut tx = pool.begin().await?;
    for (index, attachment_id) in attachment_ids.iter().enumerate() {
        let sort_order = (index + 1) as i64;
        let result = sqlx::query!(
            "UPDATE attachment SET sort_order = ? WHERE attachment_id = ? AND document_id = ?",
            sort_order,
            attachment_id,
            document_id
        )
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Attachment not found.".into()));
        }
    }
    tx.commit().await?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("attachment"),
        Some(document_id),
        "Reordered document attachments",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn get_attachment_file_path(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: Option<&str>,
    attachment_id: i64,
) -> AppResult<String> {
    let row = sqlx::query!(
        "SELECT a.stored_relative_path, d.document_id AS \"document_id!: i64\",
            d.is_hidden AS \"is_hidden!: i64\", d.is_trashed AS \"is_trashed!: i64\"
         FROM attachment a
         JOIN document d ON d.document_id = a.document_id
         WHERE a.attachment_id = ?",
        attachment_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Attachment not found.".into()))?;
    if let Some(session_id) = session_id {
        require_document_editor(pool, session_id).await?;
    } else if row.is_hidden == 1 || row.is_trashed == 1 {
        return Err(AppError::Unauthorized);
    }
    Ok(storage
        .resolve_checked(&row.stored_relative_path)?
        .to_string_lossy()
        .into_owned())
}

pub async fn get_attachment_preview_info(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: Option<&str>,
    attachment_id: i64,
) -> AppResult<AttachmentPreviewInfo> {
    let row = attachment_access_row(pool, session_id, attachment_id).await?;
    let path = storage.resolve_checked(&row.stored_relative_path)?;
    Ok(preview_info_from_row(
        row.attachment_id,
        row.document_id,
        row.original_file_name,
        row.mime_type,
        row.file_size_bytes,
        &path,
    ))
}

pub async fn get_attachment_preview_page(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: Option<&str>,
    attachment_id: i64,
    page_number: Option<i64>,
) -> AppResult<AttachmentPreviewPage> {
    let row = attachment_access_row(pool, session_id, attachment_id).await?;
    let path = storage.resolve_checked(&row.stored_relative_path)?;
    let info = preview_info_from_row(
        row.attachment_id,
        row.document_id,
        row.original_file_name,
        row.mime_type,
        row.file_size_bytes,
        &path,
    );
    let requested = page_number.unwrap_or(1).max(1);
    let max_page = info.page_count.unwrap_or(1).max(1);
    if requested > max_page {
        return Err(AppError::Validation("Preview page is out of range.".into()));
    }
    let file_path = if info.file_exists && info.preview_kind != "Unsupported" {
        Some(path.to_string_lossy().into_owned())
    } else {
        None
    };
    Ok(AttachmentPreviewPage {
        info,
        page_number: requested,
        file_path,
    })
}

pub async fn export_document_pdf(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: Option<&str>,
    document_id: i64,
    output_path: &str,
) -> AppResult<String> {
    let actor = if let Some(session_id) = session_id {
        let session = require_session(pool, session_id).await?;
        if session.role != "Secretary" && session.role != "Admin" {
            return Err(AppError::Unauthorized);
        }
        ExportActor::User {
            user_id: session.user_id,
            role: session.role,
        }
    } else {
        ExportActor::PublicViewer
    };
    let detail = get_document_internal(pool, document_id, false).await?;
    match &actor {
        ExportActor::PublicViewer => {
            if detail.document.is_hidden
                || detail.document.is_trashed
                || detail.document.status == "Confidential"
            {
                return Err(AppError::Unauthorized);
            }
        }
        ExportActor::User { role, .. } if role == "Secretary" || role == "Admin" => {
            if detail.document.is_trashed {
                return Err(AppError::Validation(
                    "Trashed documents cannot be exported from normal document detail.".into(),
                ));
            }
        }
        _ => return Err(AppError::Unauthorized),
    }
    let output = validate_pdf_output_path(output_path)?;
    let generated_at = now_text();
    let exported_by = match &actor {
        ExportActor::PublicViewer => "Staff/Head Viewer".to_owned(),
        ExportActor::User { role, user_id } => format!("{role} user #{user_id}"),
    };
    let mut pages = export_pages(&detail, storage, &generated_at, &exported_by)?;
    if pages.is_empty() {
        pages.push(vec!["No export content.".to_owned()]);
    }
    let pdf = build_simple_pdf(&pages);
    fs::write(&output, pdf)?;
    if let ExportActor::User { user_id, .. } = actor {
        write_audit_log(
            pool,
            "EXPORT",
            Some("document"),
            Some(document_id),
            "Exported document PDF",
            Some(user_id),
        )
        .await?;
    } else {
        write_audit_log(
            pool,
            "EXPORT",
            Some("document"),
            Some(document_id),
            "Exported public document PDF",
            None,
        )
        .await?;
    }
    Ok(output.to_string_lossy().into_owned())
}

pub async fn list_public_categories(pool: &DbPool) -> AppResult<Vec<CategoryItem>> {
    let rows = sqlx::query!(
        "SELECT c.category_id, c.category_name, c.description, c.color_code, c.icon, c.is_system, c.is_active,
            COUNT(d.document_id) AS \"document_count!: i64\"
         FROM category c
         LEFT JOIN document d ON d.category_id = c.category_id AND d.is_hidden = 0 AND d.is_trashed = 0
         WHERE c.is_active = 1 AND c.is_system = 0
         GROUP BY c.category_id
         ORDER BY c.category_name COLLATE NOCASE ASC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| CategoryItem {
            category_id: row.category_id,
            category_name: row.category_name,
            description: row.description,
            color_code: row.color_code,
            icon: row.icon,
            is_system: row.is_system == 1,
            is_active: row.is_active == 1,
            document_count: row.document_count,
        })
        .collect())
}

pub async fn list_public_folders(pool: &DbPool, category_id: i64) -> AppResult<Vec<FolderItem>> {
    let rows = sqlx::query!(
        "SELECT f.folder_id AS \"folder_id!: i64\", f.category_id AS \"category_id!: i64\", c.category_name, f.folder_name, f.description, f.folder_color,
            f.is_active, COUNT(d.document_id) AS \"document_count!: i64\"
         FROM folder f
         JOIN category c ON c.category_id = f.category_id
         LEFT JOIN document d ON d.folder_id = f.folder_id AND d.is_hidden = 0 AND d.is_trashed = 0
         WHERE f.category_id = ? AND f.is_active = 1
         GROUP BY f.folder_id
         ORDER BY f.folder_name COLLATE NOCASE ASC",
        category_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| FolderItem {
            folder_id: row.folder_id,
            category_id: row.category_id,
            category_name: row.category_name,
            folder_name: row.folder_name,
            description: row.description,
            folder_color: row.folder_color,
            is_active: row.is_active == 1,
            document_count: row.document_count,
        })
        .collect())
}

pub async fn list_document_offices(pool: &DbPool, session_id: &str) -> AppResult<Vec<OfficeItem>> {
    require_document_editor(pool, session_id).await?;
    let rows = sqlx::query!(
        "SELECT office_id AS \"office_id!: i64\", office_name, description, is_active
         FROM office
         WHERE is_active = 1
         ORDER BY office_name COLLATE NOCASE ASC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| OfficeItem {
            office_id: row.office_id,
            office_name: row.office_name,
            description: row.description,
            is_active: row.is_active == 1,
        })
        .collect())
}

async fn require_document_editor(
    pool: &DbPool,
    session_id: &str,
) -> AppResult<crate::auth::ValidSession> {
    let session = require_session(pool, session_id).await?;
    if session.role == "Secretary" {
        Ok(session)
    } else {
        Err(AppError::Unauthorized)
    }
}

async fn require_admin(pool: &DbPool, session_id: &str) -> AppResult<crate::auth::ValidSession> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    Ok(session)
}

async fn require_trash_viewer(
    pool: &DbPool,
    session_id: &str,
) -> AppResult<crate::auth::ValidSession> {
    let session = require_session(pool, session_id).await?;
    if session.role == "Secretary" || session.role == "Admin" {
        Ok(session)
    } else {
        Err(AppError::Unauthorized)
    }
}

async fn validate_document_input(pool: &DbPool, input: DocumentInput) -> AppResult<DocumentInput> {
    let name = require_len(&input.document_name, "Document name", 255)?;
    validate_status(&input.status)?;
    validate_date(&input.date_received)?;
    validate_document_location(pool, input.category_id, input.folder_id).await?;
    if let Some(office_id) = input.office_id {
        let office = sqlx::query!(
            "SELECT is_active AS \"is_active!: i64\" FROM office WHERE office_id = ?",
            office_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Office not found.".into()))?;
        if office.is_active != 1 {
            return Err(AppError::Validation("Office is inactive.".into()));
        }
    }
    Ok(DocumentInput {
        document_name: name,
        remarks: trim_optional(input.remarks, 2000)?,
        ..input
    })
}

async fn validate_document_location(
    pool: &DbPool,
    category_id: i64,
    folder_id: Option<i64>,
) -> AppResult<()> {
    let category = sqlx::query!(
        "SELECT is_active AS \"is_active!: i64\", is_system AS \"is_system!: i64\" FROM category WHERE category_id = ?",
        category_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Category not found.".into()))?;
    if category.is_active != 1 || category.is_system == 1 {
        return Err(AppError::Validation(
            "Category cannot accept documents.".into(),
        ));
    }
    if let Some(folder_id) = folder_id {
        let folder = sqlx::query!(
            "SELECT category_id AS \"category_id!: i64\", is_active AS \"is_active!: i64\" FROM folder WHERE folder_id = ?",
            folder_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Folder not found.".into()))?;
        if folder.category_id != category_id || folder.is_active != 1 {
            return Err(AppError::Validation(
                "Folder must belong to selected category.".into(),
            ));
        }
    }
    Ok(())
}

async fn fetch_documents(
    pool: &DbPool,
    filter: DocumentListFilter,
    public_only: bool,
    trash_only: bool,
) -> AppResult<Vec<DocumentItem>> {
    let search = like_filter(filter.search.as_deref());
    let date_from = normalize_optional_date(filter.date_from.as_deref())?;
    let date_to = normalize_optional_date(filter.date_to.as_deref())?;
    let public_only = if public_only { 1_i64 } else { 0_i64 };
    let trash_mode = if trash_only { 1_i64 } else { 0_i64 };
    let rows = sqlx::query!(
        "SELECT d.document_id AS \"document_id!: i64\", d.document_name, d.category_id AS \"category_id!: i64\",
            c.category_name, d.folder_id, f.folder_name, d.office_id, o.office_name, d.date_received,
            d.date_added, d.remarks, d.status, d.is_hidden AS \"is_hidden!: i64\",
            d.is_trashed AS \"is_trashed!: i64\", d.created_by AS \"created_by!: i64\", d.updated_at,
            u.first_name || ' ' || u.last_name AS \"created_by_name!: String\",
            COUNT(a.attachment_id) AS \"attachment_count!: i64\"
         FROM document d
         JOIN category c ON c.category_id = d.category_id
         JOIN user u ON u.user_id = d.created_by
         LEFT JOIN folder f ON f.folder_id = d.folder_id
         LEFT JOIN office o ON o.office_id = d.office_id
         LEFT JOIN attachment a ON a.document_id = d.document_id
         WHERE (? = 0 OR (d.is_hidden = 0 AND d.is_trashed = 0))
           AND ((? = 1 AND d.is_trashed = 1) OR (? = 0 AND d.is_trashed = 0))
           AND (? IS NULL OR d.category_id = ?)
           AND (? IS NULL OR d.folder_id = ?)
           AND (? IS NULL OR d.office_id = ?)
           AND (? IS NULL OR d.date_received >= ?)
           AND (? IS NULL OR d.date_received <= ?)
           AND (? IS NULL OR d.document_name LIKE ? OR d.remarks LIKE ? OR o.office_name LIKE ?)
         GROUP BY d.document_id
         ORDER BY d.date_received DESC, d.document_name COLLATE NOCASE ASC",
        public_only,
        trash_mode,
        trash_mode,
        filter.category_id,
        filter.category_id,
        filter.folder_id,
        filter.folder_id,
        filter.office_id,
        filter.office_id,
        date_from,
        date_from,
        date_to,
        date_to,
        search,
        search,
        search,
        search
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| DocumentItem {
            document_id: row.document_id,
            document_name: row.document_name,
            category_id: row.category_id,
            category_name: row.category_name,
            folder_id: row.folder_id,
            folder_name: row.folder_name,
            office_id: row.office_id,
            office_name: row.office_name,
            date_received: row.date_received,
            date_added: row.date_added,
            remarks: row.remarks,
            status: row.status,
            is_hidden: row.is_hidden == 1,
            is_trashed: row.is_trashed == 1,
            attachment_count: row.attachment_count,
            created_by: row.created_by,
            created_by_name: row.created_by_name,
            updated_at: row.updated_at,
        })
        .collect())
}

async fn get_document_internal(
    pool: &DbPool,
    document_id: i64,
    public_only: bool,
) -> AppResult<DocumentDetail> {
    let public_only = if public_only { 1_i64 } else { 0_i64 };
    let row = sqlx::query!(
        "SELECT d.document_id AS \"document_id!: i64\", d.document_name, d.category_id AS \"category_id!: i64\",
            c.category_name, d.folder_id, f.folder_name, d.office_id, o.office_name, d.date_received,
            d.date_added, d.remarks, d.status, d.is_hidden AS \"is_hidden!: i64\",
            d.is_trashed AS \"is_trashed!: i64\", d.created_by AS \"created_by!: i64\", d.updated_at,
            u.first_name || ' ' || u.last_name AS \"created_by_name!: String\",
            COUNT(a.attachment_id) AS \"attachment_count!: i64\"
         FROM document d
         JOIN category c ON c.category_id = d.category_id
         JOIN user u ON u.user_id = d.created_by
         LEFT JOIN folder f ON f.folder_id = d.folder_id
         LEFT JOIN office o ON o.office_id = d.office_id
         LEFT JOIN attachment a ON a.document_id = d.document_id
         WHERE d.document_id = ? AND (? = 0 OR (d.is_hidden = 0 AND d.is_trashed = 0))
         GROUP BY d.document_id",
        document_id,
        public_only
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found.".into()))?;
    let attachments = sqlx::query!(
        "SELECT attachment_id AS \"attachment_id!: i64\", document_id AS \"document_id!: i64\",
            original_file_name, stored_relative_path, mime_type,
            file_size_bytes AS \"file_size_bytes!: i64\", sort_order AS \"sort_order!: i64\", created_at
         FROM attachment
         WHERE document_id = ?
         ORDER BY sort_order ASC, attachment_id ASC",
        document_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| AttachmentItem {
        attachment_id: row.attachment_id,
        document_id: row.document_id,
        original_file_name: row.original_file_name,
        stored_relative_path: row.stored_relative_path,
        mime_type: row.mime_type,
        file_size_bytes: row.file_size_bytes,
        sort_order: row.sort_order,
        created_at: row.created_at,
    })
    .collect();
    Ok(DocumentDetail {
        document: DocumentItem {
            document_id: row.document_id,
            document_name: row.document_name,
            category_id: row.category_id,
            category_name: row.category_name,
            folder_id: row.folder_id,
            folder_name: row.folder_name,
            office_id: row.office_id,
            office_name: row.office_name,
            date_received: row.date_received,
            date_added: row.date_added,
            remarks: row.remarks,
            status: row.status,
            is_hidden: row.is_hidden == 1,
            is_trashed: row.is_trashed == 1,
            attachment_count: row.attachment_count,
            created_by: row.created_by,
            created_by_name: row.created_by_name,
            updated_at: row.updated_at,
        },
        attachments,
    })
}

struct AttachmentAccessRow {
    attachment_id: i64,
    document_id: i64,
    original_file_name: String,
    stored_relative_path: String,
    mime_type: String,
    file_size_bytes: i64,
}

enum ExportActor {
    PublicViewer,
    User { user_id: i64, role: String },
}

async fn attachment_access_row(
    pool: &DbPool,
    session_id: Option<&str>,
    attachment_id: i64,
) -> AppResult<AttachmentAccessRow> {
    let row = sqlx::query(
        "SELECT a.attachment_id, a.document_id,
            a.original_file_name, a.stored_relative_path, a.mime_type, a.file_size_bytes,
            d.is_hidden, d.is_trashed, d.status
         FROM attachment a
         JOIN document d ON d.document_id = a.document_id
         WHERE a.attachment_id = ?",
    )
    .bind(attachment_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Attachment not found.".into()))?;
    if let Some(session_id) = session_id {
        let session = require_session(pool, session_id).await?;
        if session.role != "Secretary" && session.role != "Admin" {
            return Err(AppError::Unauthorized);
        }
    } else if row.get::<i64, _>("is_hidden") == 1
        || row.get::<i64, _>("is_trashed") == 1
        || row.get::<String, _>("status") == "Confidential"
    {
        return Err(AppError::Unauthorized);
    }
    Ok(AttachmentAccessRow {
        attachment_id: row.get("attachment_id"),
        document_id: row.get("document_id"),
        original_file_name: row.get("original_file_name"),
        stored_relative_path: row.get("stored_relative_path"),
        mime_type: row.get("mime_type"),
        file_size_bytes: row.get("file_size_bytes"),
    })
}

fn preview_info_from_row(
    attachment_id: i64,
    document_id: i64,
    original_file_name: String,
    mime_type: String,
    file_size_bytes: i64,
    path: &Path,
) -> AttachmentPreviewInfo {
    let file_exists = path.is_file();
    let preview_kind = preview_kind(&mime_type).to_owned();
    let page_count = if file_exists && preview_kind == "Pdf" {
        estimate_pdf_page_count(path)
    } else if file_exists && preview_kind == "Image" {
        Some(1)
    } else {
        None
    };
    let message = if !file_exists {
        "Attachment file is unavailable.".to_owned()
    } else if preview_kind == "Unsupported" {
        "Preview is not supported for this file type.".to_owned()
    } else {
        "Preview available.".to_owned()
    };
    AttachmentPreviewInfo {
        attachment_id,
        document_id,
        original_file_name,
        mime_type,
        file_size_bytes,
        preview_kind,
        page_count,
        file_exists,
        message,
    }
}

fn preview_kind(mime_type: &str) -> &'static str {
    if mime_type == "application/pdf" {
        "Pdf"
    } else if mime_type.starts_with("image/") {
        "Image"
    } else {
        "Unsupported"
    }
}

fn estimate_pdf_page_count(path: &Path) -> Option<i64> {
    let bytes = fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let count =
        text.matches("/Type /Page").count() as i64 - text.matches("/Type /Pages").count() as i64;
    Some(count.max(1))
}

fn validate_pdf_output_path(output_path: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(output_path);
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(AppError::Validation("Invalid export path.".into()));
    }
    if path
        .extension()
        .and_then(OsStr::to_str)
        .map(|ext| ext.eq_ignore_ascii_case("pdf"))
        != Some(true)
    {
        return Err(AppError::Validation("Export path must end in .pdf.".into()));
    }
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Validation("Invalid export path.".into()))?;
    if !parent.is_dir() {
        return Err(AppError::Validation("Export folder does not exist.".into()));
    }
    Ok(path)
}

fn export_pages(
    detail: &DocumentDetail,
    storage: &StorageRoot,
    generated_at: &str,
    exported_by: &str,
) -> AppResult<Vec<Vec<String>>> {
    let doc = &detail.document;
    let mut pages = vec![vec![
        "UNIVERSITY OF EASTERN PHILIPPINES (UEP)".to_owned(),
        "Office of the Vice President for Student and Public External Engagement".to_owned(),
        "OVPSPEE Filing & Tracking System".to_owned(),
        String::new(),
        format!("Document: {}", doc.document_name),
        format!("Category: {}", doc.category_name),
        format!(
            "Folder: {}",
            doc.folder_name.as_deref().unwrap_or("Category root")
        ),
        format!(
            "Sender Office: {}",
            doc.office_name.as_deref().unwrap_or("Not specified")
        ),
        format!("Date Received: {}", doc.date_received),
        format!("Date Filed: {}", doc.date_added),
        format!("Status: {}", doc.status),
        format!("Generated At: {generated_at}"),
        format!("Exported By: {exported_by}"),
        String::new(),
        "Remarks".to_owned(),
    ]];
    pages[0].extend(wrap_text(
        doc.remarks.as_deref().unwrap_or("No remarks."),
        86,
    ));
    pages[0].extend([
        String::new(),
        "Certification".to_owned(),
        "This PDF was generated by the OVPSPEE Filing & Tracking System as a system copy of the stored record.".to_owned(),
        "Final certification wording and letterhead assets are pending client confirmation.".to_owned(),
    ]);

    let mut attachment_page = vec![
        "Attachment Manifest".to_owned(),
        "Attachments are listed in stored sort order. Rendered pages are included where supported by the bundled exporter.".to_owned(),
        String::new(),
    ];
    for attachment in &detail.attachments {
        let path = storage.resolve_checked(&attachment.stored_relative_path)?;
        let file_state = if path.is_file() {
            "available"
        } else {
            "file unavailable"
        };
        let preview = preview_info_from_row(
            attachment.attachment_id,
            attachment.document_id,
            attachment.original_file_name.clone(),
            attachment.mime_type.clone(),
            attachment.file_size_bytes,
            &path,
        );
        attachment_page.push(format!(
            "{}. {} ({}, {} bytes) - {}",
            attachment.sort_order,
            attachment.original_file_name,
            attachment.mime_type,
            attachment.file_size_bytes,
            file_state
        ));
        match preview.preview_kind.as_str() {
            "Pdf" => attachment_page.push(format!(
                "   PDF attachment detected; {} page(s) estimated. Full PDF page rasterization deferred.",
                preview.page_count.unwrap_or(1)
            )),
            "Image" => attachment_page.push(
                "   Image attachment detected; image preview available in the app. Inline image rendering deferred."
                    .to_owned(),
            ),
            _ => attachment_page.push(
                "   Unsupported for inline PDF rendering; file remains stored in the system.".to_owned(),
            ),
        }
    }
    if detail.attachments.is_empty() {
        attachment_page.push("No attachments.".to_owned());
    }
    pages.push(attachment_page);
    Ok(pages)
}

fn build_simple_pdf(pages: &[Vec<String>]) -> Vec<u8> {
    let total_pages = pages.len();
    let mut objects: Vec<Vec<u8>> = Vec::new();
    let catalog_id = 1;
    let pages_id = 2;
    let font_id = 3;
    let first_page_id = 4;
    let kids = (0..total_pages)
        .map(|index| format!("{} 0 R", first_page_id + (index * 2)))
        .collect::<Vec<_>>()
        .join(" ");
    objects.push(format!("<< /Type /Catalog /Pages {pages_id} 0 R >>").into_bytes());
    objects.push(format!("<< /Type /Pages /Kids [{kids}] /Count {total_pages} >>").into_bytes());
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());
    for (index, lines) in pages.iter().enumerate() {
        let page_id = first_page_id + (index * 2);
        let content_id = page_id + 1;
        objects.push(
            format!(
                "<< /Type /Page /Parent {pages_id} 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 {font_id} 0 R >> >> /Contents {content_id} 0 R >>"
            )
            .into_bytes(),
        );
        let stream = page_stream(lines, index + 1, total_pages);
        objects.push(
            format!(
                "<< /Length {} >>\nstream\n{}endstream",
                stream.as_bytes().len(),
                stream
            )
            .into_bytes(),
        );
    }

    let mut output = Vec::new();
    output.extend_from_slice(b"%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(output.len());
        let _ = writeln!(output, "{} 0 obj", index + 1);
        output.extend_from_slice(object);
        output.extend_from_slice(b"\nendobj\n");
    }
    let xref_offset = output.len();
    let _ = writeln!(output, "xref\n0 {}", objects.len() + 1);
    output.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        let _ = writeln!(output, "{offset:010} 00000 n ");
    }
    let _ = write!(
        output,
        "trailer\n<< /Size {} /Root {catalog_id} 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
        objects.len() + 1
    );
    output
}

fn page_stream(lines: &[String], page_number: usize, total_pages: usize) -> String {
    let mut stream = String::new();
    stream.push_str("BT\n/F1 11 Tf\n14 TL\n72 730 Td\n");
    for (index, line) in lines
        .iter()
        .flat_map(|line| wrap_text(line, 90))
        .take(43)
        .enumerate()
    {
        if index == 0 {
            stream.push_str(&format!("({}) Tj\n", pdf_escape(&line)));
        } else {
            stream.push_str(&format!("T* ({}) Tj\n", pdf_escape(&line)));
        }
    }
    stream.push_str("ET\n");
    stream.push_str("BT\n/F1 9 Tf\n72 42 Td\n");
    stream.push_str(&format!(
        "({}) Tj\n",
        pdf_escape(
            "System-generated copy. Verify against OVPSPEE Filing & Tracking System records."
        )
    ));
    stream.push_str(&format!(
        "430 0 Td (PAGE {} of {}) Tj\n",
        page_number, total_pages
    ));
    stream.push_str("ET\n");
    stream
}

fn wrap_text(value: &str, width: usize) -> Vec<String> {
    if value.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    for raw in value.lines() {
        let mut current = String::new();
        for word in raw.split_whitespace() {
            if !current.is_empty() && current.len() + word.len() + 1 > width {
                lines.push(current);
                current = String::new();
            }
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
        lines.push(current);
    }
    lines
}

fn pdf_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

async fn ensure_document_exists(pool: &DbPool, document_id: i64) -> AppResult<()> {
    let row = sqlx::query!(
        "SELECT document_id FROM document WHERE document_id = ?",
        document_id
    )
    .fetch_optional(pool)
    .await?;
    if row.is_some() {
        Ok(())
    } else {
        Err(AppError::NotFound("Document not found.".into()))
    }
}

async fn trash_category_id(pool: &DbPool) -> AppResult<i64> {
    sqlx::query!(
        "SELECT category_id AS \"category_id!: i64\" FROM category WHERE category_name = 'TRASH' AND is_system = 1"
    )
    .fetch_optional(pool)
    .await?
    .map(|row| row.category_id)
    .ok_or_else(|| AppError::NotFound("TRASH category not found.".into()))
}

async fn purge_document_internal(
    pool: &DbPool,
    storage: &StorageRoot,
    document_id: i64,
    user_id: i64,
) -> AppResult<()> {
    let document = sqlx::query!(
        "SELECT document_id AS \"document_id!: i64\", is_trashed AS \"is_trashed!: i64\"
         FROM document WHERE document_id = ?",
        document_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found.".into()))?;
    if document.is_trashed != 1 {
        return Err(AppError::Validation(
            "Only trashed documents can be purged.".into(),
        ));
    }
    let attachments = sqlx::query!(
        "SELECT attachment_id AS \"attachment_id!: i64\", stored_relative_path
         FROM attachment WHERE document_id = ?",
        document_id
    )
    .fetch_all(pool)
    .await?;
    for attachment in &attachments {
        let path = storage.resolve_checked(&attachment.stored_relative_path)?;
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM attachment WHERE document_id = ?", document_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM document_fts WHERE rowid = ?")
        .bind(document_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM document WHERE document_id = ?", document_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    write_audit_log(
        pool,
        "PURGE",
        Some("document"),
        Some(document_id),
        "Purged document",
        Some(user_id),
    )
    .await?;
    Ok(())
}

async fn refresh_document_fts(pool: &DbPool, document_id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM document_fts WHERE rowid = ?")
        .bind(document_id)
        .execute(pool)
        .await?;
    sqlx::query(
        "INSERT INTO document_fts(rowid, document_name, remarks, status, category_name, folder_name, office_name)
         SELECT d.document_id, d.document_name, COALESCE(d.remarks, ''), d.status, c.category_name,
                COALESCE(f.folder_name, ''), COALESCE(o.office_name, '')
         FROM document d
         JOIN category c ON c.category_id = d.category_id
         LEFT JOIN folder f ON f.folder_id = d.folder_id
         LEFT JOIN office o ON o.office_id = d.office_id
         WHERE d.document_id = ?",
    )
    .bind(document_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) fn validate_source_file(source_path: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(source_path);
    if !path.is_absolute() || !path.is_file() {
        return Err(AppError::Validation(
            "Attachment source file is invalid.".into(),
        ));
    }
    Ok(path.canonicalize()?)
}

fn validate_extension(ext: &str) -> AppResult<()> {
    if ALLOWED_EXTENSIONS.contains(&ext) {
        Ok(())
    } else {
        Err(AppError::Validation(
            "Attachment file type is not allowed.".into(),
        ))
    }
}

pub(crate) fn validate_magic(path: &Path, ext: &str) -> AppResult<()> {
    let bytes = fs::read(path)?;
    let ok = match ext {
        "pdf" => bytes.starts_with(b"%PDF"),
        "png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "jpg" | "jpeg" => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
        "docx" | "xlsx" => bytes.starts_with(b"PK"),
        "txt" => true,
        "doc" | "xls" | "tif" | "tiff" => true,
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(AppError::Validation(
            "Attachment file signature is invalid.".into(),
        ))
    }
}

pub(crate) fn mime_for_extension(ext: &str) -> &'static str {
    match ext {
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "tif" | "tiff" => "image/tiff",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
}

pub(crate) fn require_len(value: &str, label: &str, max: usize) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > max {
        return Err(AppError::Validation(format!(
            "{label} must be 1 to {max} characters."
        )));
    }
    Ok(trimmed.to_owned())
}

pub(crate) fn trim_optional(value: Option<String>, max: usize) -> AppResult<Option<String>> {
    match value {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else if trimmed.chars().count() > max {
                Err(AppError::Validation(format!(
                    "Text must be at most {max} characters."
                )))
            } else {
                Ok(Some(trimmed.to_owned()))
            }
        }
        None => Ok(None),
    }
}

fn validate_date(value: &str) -> AppResult<()> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Date must be YYYY-MM-DD.".into()))?;
    let today = Utc::now().date_naive();
    if date > today {
        return Err(AppError::Validation(
            "Date received cannot be in the future.".into(),
        ));
    }
    Ok(())
}

fn normalize_optional_date(value: Option<&str>) -> AppResult<Option<String>> {
    match value {
        Some(value) if !value.trim().is_empty() => {
            validate_date(value.trim())?;
            Ok(Some(value.trim().to_owned()))
        }
        _ => Ok(None),
    }
}

fn validate_status(value: &str) -> AppResult<()> {
    match value {
        "Filed" | "Archived" | "Confidential" | "Other" => Ok(()),
        _ => Err(AppError::Validation("Invalid document status.".into())),
    }
}

fn like_filter(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{value}%"))
}

pub(crate) fn now_text() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

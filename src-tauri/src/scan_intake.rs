use std::{ffi::OsStr, fs, path::PathBuf};

use serde::Serialize;
use uuid::Uuid;

use crate::{
    auth::{require_role, write_audit_log},
    db::DbPool,
    documents::{
        self, mime_for_extension, trim_optional, validate_magic, validate_source_file,
        DocumentInput, StorageRoot, MAX_ATTACHMENT_BYTES,
    },
    preview,
    error::{AppError, AppResult},
    util::now_text,
};

const LARGE_SCAN_WARNING_BYTES: i64 = 262_144_000;
const MAX_SCAN_PREVIEW_BYTES: i64 = 2_097_152;
// ponytail: using documents::MAX_TEXT_PREVIEW_BYTES and MAX_TEXT_PREVIEW_CHARS
const SCAN_EXTENSIONS: &[&str] = &["pdf", "jpg", "jpeg", "png", "tif", "tiff"];

#[derive(Debug, Clone, Serialize)]
pub struct ScanIntakeItem {
    pub scan_intake_id: i64,
    pub original_file_name: String,
    pub stored_relative_path: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub status: String,
    pub notes: Option<String>,
    pub is_deleted: bool,
    pub is_large: bool,
    pub created_by: i64,
    pub created_at: String,
    pub updated_at: String,
    pub filed_document_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanIntakePreviewInfo {
    pub scan_intake_id: i64,
    pub original_file_name: String,
    pub extension: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub preview_kind: String,
    pub page_count: Option<i64>,
    pub file_exists: bool,
    pub supported: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanIntakePreviewPage {
    pub info: ScanIntakePreviewInfo,
    pub page_number: i64,
    pub preview_data_url: Option<String>,
    pub text_content: Option<String>,
    pub text_truncated: bool,
}

pub async fn import_scan_files(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    source_paths: Vec<String>,
) -> AppResult<Vec<i64>> {
    let session = require_role(pool, session_id, &["Secretary"]).await?;
    if source_paths.is_empty() {
        return Err(AppError::Validation(
            "Select at least one scan file.".into(),
        ));
    }
    let mut ids = Vec::with_capacity(source_paths.len());
    for source_path in source_paths {
        let source = validate_scan_source(&source_path)?;
        let original_file_name = source
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| AppError::Validation("File name is required.".into()))?
            .to_owned();
        let ext = scan_extension(&source)?;
        let file_size = fs::metadata(&source)?.len();
        if file_size > MAX_ATTACHMENT_BYTES {
            return Err(AppError::Validation(
                "Scan file exceeds 1 GB maximum.".into(),
            ));
        }
        validate_magic(&source, &ext)?;
        let relative = format!("intake/{}.{}", Uuid::new_v4(), ext);
        let destination = storage.resolve_checked(&relative)?;
        fs::copy(&source, &destination)?;
        let mime_type = mime_for_extension(&ext).to_owned();
        let file_size_i64 = file_size as i64;
        let item = create_scan_intake_from_stored_file(
            pool,
            session.user_id,
            original_file_name,
            relative,
            mime_type,
            file_size_i64,
            "Imported scan intake file",
        )
        .await?;
        ids.push(item.scan_intake_id);
    }
    Ok(ids)
}

pub async fn create_scan_intake_from_stored_file(
    pool: &DbPool,
    user_id: i64,
    original_file_name: String,
    stored_relative_path: String,
    mime_type: String,
    file_size_bytes: i64,
    audit_summary: &str,
) -> AppResult<ScanIntakeItem> {
    if PathBuf::from(&stored_relative_path).is_absolute()
        || stored_relative_path.contains("..")
        || stored_relative_path.contains('\\')
    {
        return Err(AppError::Validation("Invalid scan storage path.".into()));
    }
    let now = now_text();
    let result = sqlx::query!(
        "INSERT INTO scan_intake
         (original_file_name, stored_relative_path, mime_type, file_size_bytes, status, created_by, created_at, updated_at)
         VALUES (?, ?, ?, ?, 'Pending', ?, ?, ?)",
        original_file_name,
        stored_relative_path,
        mime_type,
        file_size_bytes,
        user_id,
        now,
        now
    )
    .execute(pool)
    .await?;
    let id = result.last_insert_rowid();
    write_audit_log(
        pool,
        "SCAN",
        Some("scan_intake"),
        Some(id),
        audit_summary,
        Some(user_id),
    )
    .await?;
    fetch_scan(pool, id).await
}

pub async fn list_scan_intake(pool: &DbPool, session_id: &str) -> AppResult<Vec<ScanIntakeItem>> {
    require_role(pool, session_id, &["Secretary"]).await?;
    let rows = sqlx::query!(
        "SELECT scan_intake_id AS \"scan_intake_id!: i64\", original_file_name, stored_relative_path,
            mime_type, file_size_bytes AS \"file_size_bytes!: i64\", status, notes,
            is_deleted AS \"is_deleted!: i64\", created_by AS \"created_by!: i64\", created_at, updated_at,
            filed_document_id
         FROM scan_intake
         WHERE status = 'Pending' AND is_deleted = 0
         ORDER BY created_at DESC, scan_intake_id DESC"
    )
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            scan_item(
                row.scan_intake_id,
                row.original_file_name,
                row.stored_relative_path,
                row.mime_type,
                row.file_size_bytes,
                row.status,
                row.notes,
                row.is_deleted,
                row.created_by,
                row.created_at,
                row.updated_at,
                row.filed_document_id,
            )
        })
        .collect())
}

pub async fn get_scan_intake(
    pool: &DbPool,
    session_id: &str,
    scan_intake_id: i64,
) -> AppResult<ScanIntakeItem> {
    require_role(pool, session_id, &["Secretary"]).await?;
    fetch_scan(pool, scan_intake_id).await
}

pub async fn get_scan_intake_preview_page(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    scan_intake_id: i64,
    page_number: Option<i64>,
) -> AppResult<ScanIntakePreviewPage> {
    require_role(pool, session_id, &["Secretary"]).await?;
    let scan = fetch_scan(pool, scan_intake_id).await?;
    if scan.status != "Pending" || scan.is_deleted {
        return Err(AppError::NotFound("Pending scan not found.".into()));
    }
    let path = storage.resolve_checked(&scan.stored_relative_path)?;
    let info = scan_preview_info(&scan, &path);
    let requested = page_number.unwrap_or(1).max(1);
    let max_page = info.page_count.unwrap_or(1).max(1);
    if requested > max_page {
        return Err(AppError::Validation("Preview page is out of range.".into()));
    }
    let preview_data_url =
        if info.file_exists && matches!(info.preview_kind.as_str(), "Pdf" | "Image") {
            read_preview_data_url(&path, &info.mime_type, info.file_size_bytes)?
        } else {
            None
        };
    let (text_content, text_truncated) = if info.file_exists && info.preview_kind == "Text" {
        preview::read_text_preview(&path, info.file_size_bytes)?
    } else {
        (None, false)
    };
    Ok(ScanIntakePreviewPage {
        info,
        page_number: requested,
        preview_data_url,
        text_content,
        text_truncated,
    })
}

pub async fn update_scan_intake_notes(
    pool: &DbPool,
    session_id: &str,
    scan_intake_id: i64,
    notes: Option<String>,
) -> AppResult<()> {
    let session = require_role(pool, session_id, &["Secretary"]).await?;
    let notes = trim_optional(notes, 1000)?;
    let now = now_text();
    let result = sqlx::query!(
        "UPDATE scan_intake SET notes = ?, updated_at = ? WHERE scan_intake_id = ? AND status = 'Pending' AND is_deleted = 0",
        notes,
        now,
        scan_intake_id
    )
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Pending scan not found.".into()));
    }
    write_audit_log(
        pool,
        "UPDATE",
        Some("scan_intake"),
        Some(scan_intake_id),
        "Updated scan intake notes",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn remove_scan_intake(
    pool: &DbPool,
    session_id: &str,
    scan_intake_id: i64,
) -> AppResult<()> {
    let session = require_role(pool, session_id, &["Secretary"]).await?;
    let now = now_text();
    let result = sqlx::query!(
        "UPDATE scan_intake
         SET status = 'Removed', is_deleted = 1, deleted_at = ?, updated_at = ?
         WHERE scan_intake_id = ? AND status = 'Pending' AND is_deleted = 0",
        now,
        now,
        scan_intake_id
    )
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Pending scan not found.".into()));
    }
    write_audit_log(
        pool,
        "DELETE",
        Some("scan_intake"),
        Some(scan_intake_id),
        "Removed scan intake file",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn file_scan_as_document(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    scan_intake_ids: Vec<i64>,
    input: DocumentInput,
) -> AppResult<i64> {
    let session = require_role(pool, session_id, &["Secretary"]).await?;
    let scans = validate_pending_scans(pool, &scan_intake_ids).await?;
    let document_id = documents::create_document(pool, session_id, input).await?;
    claim_scans(pool, storage, session.user_id, document_id, scans).await?;
    write_audit_log(
        pool,
        "SCAN",
        Some("document"),
        Some(document_id),
        "Filed scan intake as new document",
        Some(session.user_id),
    )
    .await?;
    Ok(document_id)
}

pub async fn attach_scan_to_document(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    scan_intake_ids: Vec<i64>,
    document_id: i64,
) -> AppResult<Vec<i64>> {
    let session = require_role(pool, session_id, &["Secretary"]).await?;
    let document = sqlx::query!(
        "SELECT is_trashed AS \"is_trashed!: i64\" FROM document WHERE document_id = ?",
        document_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found.".into()))?;
    if document.is_trashed == 1 {
        return Err(AppError::Validation(
            "Restore document before attaching scans.".into(),
        ));
    }
    let scans = validate_pending_scans(pool, &scan_intake_ids).await?;
    let ids = claim_scans(pool, storage, session.user_id, document_id, scans).await?;
    write_audit_log(
        pool,
        "SCAN",
        Some("document"),
        Some(document_id),
        "Attached scan intake to existing document",
        Some(session.user_id),
    )
    .await?;
    Ok(ids)
}

async fn claim_scans(
    pool: &DbPool,
    storage: &StorageRoot,
    user_id: i64,
    document_id: i64,
    scans: Vec<ScanIntakeItem>,
) -> AppResult<Vec<i64>> {
    let mut attachment_ids = Vec::with_capacity(scans.len());
    let max_order = sqlx::query!(
        "SELECT COALESCE(MAX(sort_order), 0) AS \"max_order!: i64\" FROM attachment WHERE document_id = ?",
        document_id
    )
    .fetch_one(pool)
    .await?
    .max_order;
    for (index, scan) in scans.into_iter().enumerate() {
        let ext = PathBuf::from(&scan.stored_relative_path)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or("scan")
            .to_ascii_lowercase();
        let new_relative = format!("documents/{document_id}/scans/{}.{}", Uuid::new_v4(), ext);
        let old_path = storage.resolve_checked(&scan.stored_relative_path)?;
        let new_path = storage.resolve_checked(&new_relative)?;
        if let Err(rename_err) = fs::rename(&old_path, &new_path) {
            fs::copy(&old_path, &new_path).map_err(|_| rename_err)?;
            fs::remove_file(&old_path)?;
        }
        let sort_order = max_order + index as i64 + 1;
        let result = sqlx::query!(
            "INSERT INTO attachment
             (document_id, original_file_name, stored_relative_path, mime_type, file_size_bytes, sort_order)
             VALUES (?, ?, ?, ?, ?, ?)",
            document_id,
            scan.original_file_name,
            new_relative,
            scan.mime_type,
            scan.file_size_bytes,
            sort_order
        )
        .execute(pool)
        .await?;
        let attachment_id = result.last_insert_rowid();
        let now = now_text();
        sqlx::query!(
            "UPDATE scan_intake
             SET status = 'Filed', stored_relative_path = ?, filed_document_id = ?, updated_at = ?
             WHERE scan_intake_id = ?",
            new_relative,
            document_id,
            now,
            scan.scan_intake_id
        )
        .execute(pool)
        .await?;
        write_audit_log(
            pool,
            "SCAN",
            Some("scan_intake"),
            Some(scan.scan_intake_id),
            "Claimed scan intake as document attachment",
            Some(user_id),
        )
        .await?;
        attachment_ids.push(attachment_id);
    }
    Ok(attachment_ids)
}

async fn validate_pending_scans(
    pool: &DbPool,
    scan_intake_ids: &[i64],
) -> AppResult<Vec<ScanIntakeItem>> {
    if scan_intake_ids.is_empty() {
        return Err(AppError::Validation("Select at least one scan.".into()));
    }
    let placeholders = scan_intake_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT scan_intake_id, original_file_name, stored_relative_path,
            mime_type, file_size_bytes, status, notes,
            is_deleted, created_by, created_at, updated_at,
            filed_document_id
         FROM scan_intake WHERE scan_intake_id IN ({placeholders})"
    );
    let mut q = sqlx::query(&sql);
    for id in scan_intake_ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(pool).await?;
    if rows.len() != scan_intake_ids.len() {
        return Err(AppError::NotFound("One or more scan intake items not found.".into()));
    }
    let mut scans = Vec::with_capacity(rows.len());
    use sqlx::Row;
    for row in rows {
        let scan_intake_id: i64 = row.get("scan_intake_id");
        let status: String = row.get("status");
        let is_deleted: i64 = row.get("is_deleted");
        if status != "Pending" || is_deleted != 0 {
            return Err(AppError::Validation("Only pending scans can be filed.".into()));
        }
        scans.push(scan_item(
            scan_intake_id,
            row.get("original_file_name"),
            row.get("stored_relative_path"),
            row.get("mime_type"),
            row.get("file_size_bytes"),
            status,
            row.get("notes"),
            is_deleted,
            row.get("created_by"),
            row.get("created_at"),
            row.get("updated_at"),
            row.get("filed_document_id"),
        ));
    }
    Ok(scans)
}

async fn fetch_scan(pool: &DbPool, scan_intake_id: i64) -> AppResult<ScanIntakeItem> {
    let row = sqlx::query!(
        "SELECT scan_intake_id AS \"scan_intake_id!: i64\", original_file_name, stored_relative_path,
            mime_type, file_size_bytes AS \"file_size_bytes!: i64\", status, notes,
            is_deleted AS \"is_deleted!: i64\", created_by AS \"created_by!: i64\", created_at, updated_at,
            filed_document_id
         FROM scan_intake WHERE scan_intake_id = ?",
        scan_intake_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Scan intake item not found.".into()))?;
    Ok(scan_item(
        row.scan_intake_id,
        row.original_file_name,
        row.stored_relative_path,
        row.mime_type,
        row.file_size_bytes,
        row.status,
        row.notes,
        row.is_deleted,
        row.created_by,
        row.created_at,
        row.updated_at,
        row.filed_document_id,
    ))
}

#[allow(clippy::too_many_arguments)]
fn scan_item(
    scan_intake_id: i64,
    original_file_name: String,
    stored_relative_path: String,
    mime_type: String,
    file_size_bytes: i64,
    status: String,
    notes: Option<String>,
    is_deleted: i64,
    created_by: i64,
    created_at: String,
    updated_at: String,
    filed_document_id: Option<i64>,
) -> ScanIntakeItem {
    ScanIntakeItem {
        scan_intake_id,
        original_file_name,
        stored_relative_path,
        mime_type,
        file_size_bytes,
        status,
        notes,
        is_deleted: is_deleted == 1,
        is_large: file_size_bytes > LARGE_SCAN_WARNING_BYTES,
        created_by,
        created_at,
        updated_at,
        filed_document_id,
    }
}

fn validate_scan_source(source_path: &str) -> AppResult<PathBuf> {
    let source = validate_source_file(source_path)?;
    let ext = scan_extension(&source)?;
    if !SCAN_EXTENSIONS.contains(&ext.as_str()) {
        return Err(AppError::Validation(
            "Scan file type is not allowed.".into(),
        ));
    }
    Ok(source)
}

fn scan_extension(path: &PathBuf) -> AppResult<String> {
    let ext = path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if SCAN_EXTENSIONS.contains(&ext.as_str()) {
        Ok(ext)
    } else {
        Err(AppError::Validation(
            "Scan file type is not allowed.".into(),
        ))
    }
}

fn scan_preview_info(scan: &ScanIntakeItem, path: &std::path::Path) -> ScanIntakePreviewInfo {
    let file_exists = path.is_file();
    // ponytail: TIFF treated as Image for scan preview, not via shared preview_kind (breaks attachment preview)
    let preview_kind = if scan.mime_type == "image/tiff" { "Image" } else { preview::preview_kind(&scan.mime_type) }.to_owned();
    let supported = preview_kind != "Unsupported";
    let page_count = if file_exists && preview_kind == "Pdf" {
        preview::estimate_pdf_page_count(path)
    } else if file_exists && preview_kind == "Image" {
        Some(1)
    } else {
        None
    };
    let message = if !file_exists {
        "Pending scan file is unavailable.".to_owned()
    } else if matches!(preview_kind.as_str(), "Pdf" | "Image")
        && scan.file_size_bytes > MAX_SCAN_PREVIEW_BYTES
    {
        "Preview is unavailable because this pending file is too large.".to_owned()
    } else if preview_kind == "Text" && scan.file_size_bytes > preview::MAX_TEXT_PREVIEW_BYTES {
        "Text preview is unavailable because this pending file is too large.".to_owned()
    } else if preview_kind == "Unsupported" {
        "Preview not available for this scan file type.".to_owned()
    } else {
        "Preview available.".to_owned()
    };
    ScanIntakePreviewInfo {
        scan_intake_id: scan.scan_intake_id,
        original_file_name: scan.original_file_name.clone(),
        extension: path
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or("unknown")
            .to_ascii_lowercase(),
        mime_type: scan.mime_type.clone(),
        file_size_bytes: scan.file_size_bytes,
        preview_kind,
        page_count,
        file_exists,
        supported,
        message,
    }
}

fn read_preview_data_url(
    path: &std::path::Path,
    mime_type: &str,
    file_size: i64,
) -> AppResult<Option<String>> {
    if file_size > MAX_SCAN_PREVIEW_BYTES {
        return Ok(None);
    }
    let (bytes, response_mime) = if mime_type == "image/tiff" {
        (convert_tiff_to_png(path)?, "image/png")
    } else {
        (fs::read(path)?, mime_type)
    };
    Ok(Some(format!(
        "data:{response_mime};base64,{}",
        encode_base64(&bytes)
    )))
}

fn convert_tiff_to_png(path: &std::path::Path) -> AppResult<Vec<u8>> {
    let img = image::open(path)
        .map_err(|e| AppError::Validation(format!("Failed to decode TIFF for preview: {e}")))?;
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| AppError::Validation(format!("Failed to encode TIFF as PNG for preview: {e}")))?;
    Ok(buf)
}

fn encode_base64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}



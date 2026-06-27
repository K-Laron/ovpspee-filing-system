use std::{ffi::OsStr, fs, path::Path};

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::{require_role, write_audit_log},
    db::DbPool,
    documents::{
        self, mime_for_extension, require_len, trim_optional, validate_magic, validate_source_file,
        AttachmentInput, DocumentInput, StorageRoot, MAX_ATTACHMENT_BYTES,
    },
    error::{AppError, AppResult},
    preview,
    util::{now_text, validate_date, validate_status},
};

#[derive(Debug, Clone, Deserialize)]
pub struct MobileSubmissionInput {
    pub client_submission_id: Option<String>,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub document_name: String,
    pub category_id: i64,
    pub folder_id: Option<i64>,
    pub office_id: Option<i64>,
    pub date_received: String,
    pub remarks: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct MobileSubmissionAttachmentUpload {
    pub source_path: String,
    pub original_file_name: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MobileSubmissionItem {
    pub mobile_submission_id: i64,
    pub submitted_by: i64,
    pub submitter_name: String,
    pub document_name: String,
    pub category_id: i64,
    pub category_name: String,
    pub folder_id: Option<i64>,
    pub folder_name: Option<String>,
    pub office_id: Option<i64>,
    pub office_name: Option<String>,
    pub date_received: String,
    pub remarks: Option<String>,
    pub status: String,
    pub review_status: String,
    pub rejection_reason: Option<String>,
    pub review_notes: Option<String>,
    pub reviewed_by: Option<i64>,
    pub reviewer_name: Option<String>,
    pub reviewed_at: Option<String>,
    pub resulting_document_id: Option<i64>,
    pub client_submission_id: Option<String>,
    pub submitted_device_id: Option<String>,
    pub submitted_device_name: Option<String>,
    pub attachment_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MobileSubmissionAttachmentItem {
    pub mobile_submission_attachment_id: i64,
    pub mobile_submission_id: i64,
    pub original_file_name: String,
    #[serde(skip_serializing)]
    pub stored_relative_path: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MobileSubmissionAttachmentPreviewInfo {
    pub mobile_submission_attachment_id: i64,
    pub mobile_submission_id: i64,
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
pub struct MobileSubmissionAttachmentPreviewPage {
    pub info: MobileSubmissionAttachmentPreviewInfo,
    pub page_number: i64,
    pub file_path: Option<String>,
    pub text_content: Option<String>,
    pub text_truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MobileSubmissionDetail {
    pub submission: MobileSubmissionItem,
    pub attachments: Vec<MobileSubmissionAttachmentItem>,
}

pub async fn create_mobile_submission(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    input: MobileSubmissionInput,
    uploads: Vec<MobileSubmissionAttachmentUpload>,
) -> AppResult<i64> {
    let session = require_role(pool, session_id, &["Secretary"]).await?;
    let client_submission_id = trim_optional(input.client_submission_id.clone(), 120)?;
    let submitted_device_id = trim_optional(input.device_id.clone(), 120)?;
    let submitted_device_name = trim_optional(input.device_name.clone(), 120)?;
    if let Some(client_submission_id) = client_submission_id.as_deref() {
        if let Some(existing_id) =
            find_existing_client_submission(pool, session.user_id, client_submission_id).await?
        {
            return Ok(existing_id);
        }
    }
    if uploads.is_empty() {
        return Err(AppError::Validation(
            "At least one attachment is required.".into(),
        ));
    }
    let input = validate_mobile_input(pool, input).await?;
    let now = now_text();
    let result = sqlx::query(
        "INSERT INTO mobile_submission
         (submitted_by, document_name, category_id, folder_id, office_id, date_received, remarks,
          status, review_status, client_submission_id, submitted_device_id, submitted_device_name,
          created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'Pending', ?, ?, ?, ?, ?)",
    )
    .bind(session.user_id)
    .bind(&input.document_name)
    .bind(input.category_id)
    .bind(input.folder_id)
    .bind(input.office_id)
    .bind(&input.date_received)
    .bind(&input.remarks)
    .bind(&input.status)
    .bind(&client_submission_id)
    .bind(&submitted_device_id)
    .bind(&submitted_device_name)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    let submission_id = result.last_insert_rowid();

    for (index, upload) in uploads.into_iter().enumerate() {
        store_mobile_attachment(pool, storage, submission_id, upload, index as i64 + 1).await?;
    }

    write_audit_log(
        pool,
        "INSERT",
        Some("mobile_submission"),
        Some(submission_id),
        "Created mobile submission",
        Some(session.user_id),
    )
    .await?;
    Ok(submission_id)
}

pub async fn list_mobile_submissions(
    pool: &DbPool,
    session_id: &str,
    review_status: Option<String>,
    search: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> AppResult<Vec<MobileSubmissionItem>> {
    require_role(pool, session_id, &["Secretary"]).await?;
    let status = review_status
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    if let Some(status) = status.as_deref() {
        validate_review_status(status)?;
    }
    fetch_submission_rows(pool, status, search, date_from, date_to).await
}

pub async fn get_mobile_submission(
    pool: &DbPool,
    session_id: &str,
    mobile_submission_id: i64,
) -> AppResult<MobileSubmissionDetail> {
    require_role(pool, session_id, &["Secretary"]).await?;
    let submission = fetch_submission(pool, mobile_submission_id).await?;
    let attachments = fetch_attachments(pool, mobile_submission_id).await?;
    Ok(MobileSubmissionDetail {
        submission,
        attachments,
    })
}

pub async fn get_mobile_submission_attachment_preview_page(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    mobile_submission_attachment_id: i64,
    page_number: Option<i64>,
) -> AppResult<MobileSubmissionAttachmentPreviewPage> {
    require_role(pool, session_id, &["Secretary"]).await?;
    let attachment = fetch_attachment(pool, mobile_submission_attachment_id).await?;
    let path = storage.resolve_checked(&attachment.stored_relative_path)?;
    let info = mobile_preview_info(&attachment, &path);
    let requested = page_number.unwrap_or(1).max(1);
    let max_page = info.page_count.unwrap_or(1).max(1);
    if requested > max_page {
        return Err(AppError::Validation("Preview page is out of range.".into()));
    }
    let (text_content, text_truncated) = if info.file_exists && info.preview_kind == "Text" {
        preview::read_text_preview(&path, info.file_size_bytes)?
    } else {
        (None, false)
    };
    let file_path = if info.file_exists && matches!(info.preview_kind.as_str(), "Pdf" | "Image") {
        Some(path.to_string_lossy().into_owned())
    } else {
        None
    };
    Ok(MobileSubmissionAttachmentPreviewPage {
        info,
        page_number: requested,
        file_path,
        text_content,
        text_truncated,
    })
}

pub async fn approve_mobile_submission(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    mobile_submission_id: i64,
    review_notes: Option<String>,
) -> AppResult<i64> {
    let session = require_role(pool, session_id, &["Secretary"]).await?;
    let detail = get_pending_submission(pool, mobile_submission_id).await?;
    let document_id = documents::create_document(
        pool,
        session_id,
        DocumentInput {
            document_name: detail.submission.document_name.clone(),
            category_id: detail.submission.category_id,
            folder_id: detail.submission.folder_id,
            office_id: detail.submission.office_id,
            date_received: detail.submission.date_received.clone(),
            remarks: detail.submission.remarks.clone(),
            status: detail.submission.status.clone(),
        },
    )
    .await?;

    for (index, attachment) in detail.attachments.iter().enumerate() {
        let source = storage.resolve_checked(&attachment.stored_relative_path)?;
        let attachment_id = documents::add_attachment(
            pool,
            storage,
            session_id,
            document_id,
            AttachmentInput {
                source_path: source.to_string_lossy().into_owned(),
                sort_order: Some(index as i64 + 1),
            },
        )
        .await?;
        sqlx::query("UPDATE attachment SET original_file_name = ? WHERE attachment_id = ?")
            .bind(&attachment.original_file_name)
            .bind(attachment_id)
            .execute(pool)
            .await?;
    }

    let notes = trim_optional(review_notes, 2000)?;
    let now = now_text();
    sqlx::query(
        "UPDATE mobile_submission
         SET review_status = 'Approved', review_notes = ?, reviewed_by = ?, reviewed_at = ?,
             resulting_document_id = ?, updated_at = ?
         WHERE mobile_submission_id = ? AND review_status = 'Pending'",
    )
    .bind(&notes)
    .bind(session.user_id)
    .bind(&now)
    .bind(document_id)
    .bind(&now)
    .bind(mobile_submission_id)
    .execute(pool)
    .await?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("mobile_submission"),
        Some(mobile_submission_id),
        "Approved mobile submission",
        Some(session.user_id),
    )
    .await?;
    Ok(document_id)
}

pub async fn reject_mobile_submission(
    pool: &DbPool,
    session_id: &str,
    mobile_submission_id: i64,
    rejection_reason: String,
) -> AppResult<()> {
    let session = require_role(pool, session_id, &["Secretary"]).await?;
    get_pending_submission(pool, mobile_submission_id).await?;
    let reason = require_len(&rejection_reason, "Rejection reason", 1000)?;
    let now = now_text();
    sqlx::query(
        "UPDATE mobile_submission
         SET review_status = 'Rejected', rejection_reason = ?, reviewed_by = ?, reviewed_at = ?, updated_at = ?
         WHERE mobile_submission_id = ? AND review_status = 'Pending'",
    )
    .bind(&reason)
    .bind(session.user_id)
    .bind(&now)
    .bind(&now)
    .bind(mobile_submission_id)
    .execute(pool)
    .await?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("mobile_submission"),
        Some(mobile_submission_id),
        "Rejected mobile submission",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}



async fn validate_mobile_input(
    pool: &DbPool,
    input: MobileSubmissionInput,
) -> AppResult<DocumentInput> {
    let normalized = DocumentInput {
        document_name: require_len(&input.document_name, "Document title", 200)?,
        category_id: input.category_id,
        folder_id: input.folder_id,
        office_id: input.office_id,
        date_received: input.date_received,
        remarks: trim_optional(input.remarks, 2000)?,
        status: input.status,
    };
    validate_status(&normalized.status)?;
    validate_date(&normalized.date_received)?;
    crate::documents::validate_document_location(pool, normalized.category_id, normalized.folder_id).await?;
    if let Some(office_id) = normalized.office_id {
        let office = sqlx::query("SELECT is_active FROM office WHERE office_id = ?")
            .bind(office_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Office not found.".into()))?;
        let is_active: i64 = sqlx::Row::try_get(&office, "is_active")?;
        if is_active != 1 {
            return Err(AppError::Validation("Office is inactive.".into()));
        }
    }
    Ok(normalized)
}

async fn store_mobile_attachment(
    pool: &DbPool,
    storage: &StorageRoot,
    submission_id: i64,
    upload: MobileSubmissionAttachmentUpload,
    sort_order: i64,
) -> AppResult<i64> {
    let source = validate_source_file(&upload.source_path)?;
    let ext = source
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    validate_magic(&source, &ext)?;
    let file_size = fs::metadata(&source)?.len();
    if file_size > MAX_ATTACHMENT_BYTES {
        return Err(AppError::Validation(
            "Attachment exceeds 1 GB maximum.".into(),
        ));
    }
    let relative = format!(
        "mobile-submissions/{submission_id}/{}.{}",
        Uuid::new_v4(),
        ext
    );
    let destination = storage.resolve_checked(&relative)?;
    fs::copy(&source, &destination)?;
    let mime_type = mime_for_extension(&ext).to_owned();
    let original = require_len(&upload.original_file_name, "File name", 255)?;
    let file_size_i64 = file_size as i64;

    let result = sqlx::query(
        "INSERT INTO mobile_submission_attachment
         (mobile_submission_id, original_file_name, stored_relative_path, mime_type, file_size_bytes, sort_order)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(submission_id)
    .bind(&original)
    .bind(&relative)
    .bind(&mime_type)
    .bind(file_size_i64)
    .bind(sort_order)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}



async fn get_pending_submission(
    pool: &DbPool,
    mobile_submission_id: i64,
) -> AppResult<MobileSubmissionDetail> {
    let submission = fetch_submission(pool, mobile_submission_id).await?;
    if submission.review_status != "Pending" {
        return Err(AppError::Conflict(
            "Only pending mobile submissions can be reviewed.".into(),
        ));
    }
    let attachments = fetch_attachments(pool, mobile_submission_id).await?;
    if attachments.is_empty() {
        return Err(AppError::Validation(
            "At least one attachment is required.".into(),
        ));
    }
    Ok(MobileSubmissionDetail {
        submission,
        attachments,
    })
}

async fn fetch_submission(
    pool: &DbPool,
    mobile_submission_id: i64,
) -> AppResult<MobileSubmissionItem> {
    fetch_submission_row_by_id(pool, mobile_submission_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Mobile submission not found.".into()))
}

async fn find_existing_client_submission(
    pool: &DbPool,
    submitted_by: i64,
    client_submission_id: &str,
) -> AppResult<Option<i64>> {
    let id = sqlx::query_scalar::<_, i64>(
        "SELECT mobile_submission_id
         FROM mobile_submission
         WHERE submitted_by = ? AND client_submission_id = ?",
    )
    .bind(submitted_by)
    .bind(client_submission_id)
    .fetch_optional(pool)
    .await?;
    Ok(id)
}

async fn fetch_submission_rows(
    pool: &DbPool,
    status: Option<String>,
    search: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> AppResult<Vec<MobileSubmissionItem>> {
    let mut where_parts = Vec::new();
    if status.is_some() {
        where_parts.push("ms.review_status = ?");
    }
    let search = search
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    if search.is_some() {
        where_parts.push("(ms.document_name LIKE ? OR u.first_name || ' ' || u.last_name LIKE ? OR ms.submitted_device_name LIKE ?)");
    }
    if date_from
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        validate_date(date_from.as_deref().unwrap_or_default())?;
        where_parts.push("ms.date_received >= ?");
    }
    if date_to
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        validate_date(date_to.as_deref().unwrap_or_default())?;
        where_parts.push("ms.date_received <= ?");
    }
    let where_clause = if where_parts.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_parts.join(" AND "))
    };
    let sql = submission_select(&where_clause);
    let mut query = sqlx::query_as::<_, MobileSubmissionItem>(&sql);
    if let Some(status) = status {
        query = query.bind(status);
    }
    if let Some(search) = search {
        let pattern = format!("%{search}%");
        query = query
            .bind(pattern.clone())
            .bind(pattern.clone())
            .bind(pattern);
    }
    if let Some(date_from) = date_from.filter(|value| !value.trim().is_empty()) {
        query = query.bind(date_from);
    }
    if let Some(date_to) = date_to.filter(|value| !value.trim().is_empty()) {
        query = query.bind(date_to);
    }
    Ok(query.fetch_all(pool).await?)
}

async fn fetch_submission_row_by_id(
    pool: &DbPool,
    mobile_submission_id: i64,
) -> AppResult<Option<MobileSubmissionItem>> {
    let sql = submission_select("WHERE ms.mobile_submission_id = ?");
    Ok(sqlx::query_as::<_, MobileSubmissionItem>(&sql)
        .bind(mobile_submission_id)
        .fetch_optional(pool)
        .await?)
}

fn submission_select(where_clause: &str) -> String {
    format!(
        "SELECT ms.mobile_submission_id,
                ms.submitted_by,
                (u.first_name || ' ' || u.last_name) AS submitter_name,
                ms.document_name,
                ms.category_id,
                c.category_name,
                ms.folder_id,
                f.folder_name,
                ms.office_id,
                o.office_name,
                ms.date_received,
                ms.remarks,
                ms.status,
                ms.review_status,
                ms.rejection_reason,
                ms.review_notes,
                ms.reviewed_by,
                CASE WHEN reviewer.user_id IS NULL THEN NULL ELSE reviewer.first_name || ' ' || reviewer.last_name END AS reviewer_name,
                ms.reviewed_at,
                ms.resulting_document_id,
                ms.client_submission_id,
                ms.submitted_device_id,
                ms.submitted_device_name,
                COUNT(a.mobile_submission_attachment_id) AS attachment_count,
                ms.created_at,
                ms.updated_at
         FROM mobile_submission ms
         JOIN user u ON u.user_id = ms.submitted_by
         JOIN category c ON c.category_id = ms.category_id
         LEFT JOIN folder f ON f.folder_id = ms.folder_id
         LEFT JOIN office o ON o.office_id = ms.office_id
         LEFT JOIN user reviewer ON reviewer.user_id = ms.reviewed_by
         LEFT JOIN mobile_submission_attachment a ON a.mobile_submission_id = ms.mobile_submission_id
         {where_clause}
         GROUP BY ms.mobile_submission_id
         ORDER BY ms.created_at DESC, ms.mobile_submission_id DESC"
    )
}

async fn fetch_attachments(
    pool: &DbPool,
    mobile_submission_id: i64,
) -> AppResult<Vec<MobileSubmissionAttachmentItem>> {
    let rows = sqlx::query_as::<_, MobileSubmissionAttachmentItem>(
        "SELECT mobile_submission_attachment_id,
                mobile_submission_id,
                original_file_name,
                stored_relative_path,
                mime_type,
                file_size_bytes,
                sort_order,
                created_at
         FROM mobile_submission_attachment
         WHERE mobile_submission_id = ?
         ORDER BY sort_order, mobile_submission_attachment_id",
    )
    .bind(mobile_submission_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn fetch_attachment(
    pool: &DbPool,
    mobile_submission_attachment_id: i64,
) -> AppResult<MobileSubmissionAttachmentItem> {
    sqlx::query_as::<_, MobileSubmissionAttachmentItem>(
        "SELECT mobile_submission_attachment_id,
                mobile_submission_id,
                original_file_name,
                stored_relative_path,
                mime_type,
                file_size_bytes,
                sort_order,
                created_at
         FROM mobile_submission_attachment
         WHERE mobile_submission_attachment_id = ?",
    )
    .bind(mobile_submission_attachment_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Mobile attachment not found.".into()))
}

fn mobile_preview_info(
    attachment: &MobileSubmissionAttachmentItem,
    path: &Path,
) -> MobileSubmissionAttachmentPreviewInfo {
    let file_exists = path.is_file();
    let preview_kind = preview::preview_kind(&attachment.mime_type).to_owned();
    let supported = preview_kind != "Unsupported";
    let page_count = if file_exists && preview_kind == "Pdf" {
        preview::estimate_pdf_page_count(path)
    } else if file_exists && preview_kind == "Image" {
        Some(1)
    } else {
        None
    };
    let message = if !file_exists {
        "Attachment file is unavailable.".to_owned()
    } else if preview_kind == "Text" && attachment.file_size_bytes > preview::MAX_TEXT_PREVIEW_BYTES {
        "Text preview is unavailable because this file is too large.".to_owned()
    } else if preview_kind == "Unsupported" {
        "Preview not available for this file type.".to_owned()
    } else {
        "Preview available.".to_owned()
    };
    MobileSubmissionAttachmentPreviewInfo {
        mobile_submission_attachment_id: attachment.mobile_submission_attachment_id,
        mobile_submission_id: attachment.mobile_submission_id,
        original_file_name: attachment.original_file_name.clone(),
        extension: preview::extension_from_name(Path::new(&attachment.original_file_name), &attachment.mime_type),
        mime_type: attachment.mime_type.clone(),
        file_size_bytes: attachment.file_size_bytes,
        preview_kind,
        page_count,
        file_exists,
        supported,
        message,
    }
}

fn validate_review_status(value: &str) -> AppResult<()> {
    match value {
        "Pending" | "Approved" | "Rejected" | "Removed" => Ok(()),
        _ => Err(AppError::Validation("Invalid review status.".into())),
    }
}

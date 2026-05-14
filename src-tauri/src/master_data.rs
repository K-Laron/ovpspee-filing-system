use serde::Serialize;

use crate::{
    auth::{require_admin_role, require_session, write_audit_log},
    db::DbPool,
    error::{AppError, AppResult},
};

#[derive(Debug, Clone)]
pub struct CategoryInput {
    pub category_name: String,
    pub description: Option<String>,
    pub color_code: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryItem {
    pub category_id: i64,
    pub category_name: String,
    pub description: Option<String>,
    pub color_code: String,
    pub icon: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub document_count: i64,
}

#[derive(Debug, Clone)]
pub struct FolderInput {
    pub category_id: i64,
    pub folder_name: String,
    pub description: Option<String>,
    pub folder_color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FolderItem {
    pub folder_id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub folder_name: String,
    pub description: Option<String>,
    pub folder_color: String,
    pub is_active: bool,
    pub document_count: i64,
}

#[derive(Debug, Clone)]
pub struct OfficeInput {
    pub office_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OfficeItem {
    pub office_id: i64,
    pub office_name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

pub async fn list_categories(
    pool: &DbPool,
    session_id: &str,
    include_inactive: Option<bool>,
) -> AppResult<Vec<CategoryItem>> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let include_inactive = if include_inactive.unwrap_or(false) { 1_i64 } else { 0_i64 };

    let rows = sqlx::query!(
        "SELECT category_id, category_name, description, color_code, icon, is_system, is_active,
            0 AS \"document_count!: i64\"
         FROM category
         WHERE (? = 1 OR is_active = 1)
         ORDER BY CASE WHEN category_name = 'TRASH' THEN 1 ELSE 0 END, category_name COLLATE NOCASE ASC",
        include_inactive
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

pub async fn create_category(
    pool: &DbPool,
    session_id: &str,
    input: CategoryInput,
) -> AppResult<i64> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let name = validate_name(&input.category_name, "Category name")?;
    validate_hex_color(&input.color_code)?;

    let result = sqlx::query!(
        "INSERT INTO category (category_name, description, color_code, icon) VALUES (?, ?, ?, ?)",
        name,
        input.description,
        input.color_code,
        input.icon
    )
    .execute(pool)
    .await;

    let result = map_unique(result, "Category already exists.")?;
    let id = result.last_insert_rowid();
    write_audit_log(
        pool,
        "INSERT",
        Some("category"),
        Some(id),
        "Created category",
        Some(session.user_id),
    )
    .await?;
    Ok(id)
}

pub async fn update_category(
    pool: &DbPool,
    session_id: &str,
    category_id: i64,
    input: CategoryInput,
    is_active: bool,
) -> AppResult<()> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let existing = sqlx::query!(
        "SELECT is_system AS \"is_system!: i64\" FROM category WHERE category_id = ?",
        category_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Category not found.".into()))?;
    if existing.is_system == 1 {
        return Err(AppError::SystemRecord);
    }

    let name = validate_name(&input.category_name, "Category name")?;
    validate_hex_color(&input.color_code)?;
    let is_active_int = if is_active { 1 } else { 0 };
    let now = now_text();
    let result = sqlx::query!(
        "UPDATE category
         SET category_name = ?, description = ?, color_code = ?, icon = ?, is_active = ?, updated_at = ?
         WHERE category_id = ?",
        name,
        input.description,
        input.color_code,
        input.icon,
        is_active_int,
        now,
        category_id
    )
    .execute(pool)
    .await;
    map_unique(result, "Category already exists.")?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("category"),
        Some(category_id),
        "Updated category",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn list_folders(
    pool: &DbPool,
    session_id: &str,
    category_id: Option<i64>,
    include_inactive: Option<bool>,
) -> AppResult<Vec<FolderItem>> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let include_inactive = if include_inactive.unwrap_or(false) { 1_i64 } else { 0_i64 };

    let rows = sqlx::query!(
        "SELECT f.folder_id, f.category_id AS \"category_id!: i64\", c.category_name, f.folder_name, f.description, f.folder_color,
            f.is_active, 0 AS \"document_count!: i64\"
         FROM folder f
         JOIN category c ON c.category_id = f.category_id
         WHERE (? IS NULL OR f.category_id = ?) AND (? = 1 OR f.is_active = 1)
         ORDER BY c.category_name COLLATE NOCASE ASC, f.folder_name COLLATE NOCASE ASC",
        category_id,
        category_id,
        include_inactive
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

pub async fn create_folder(pool: &DbPool, session_id: &str, input: FolderInput) -> AppResult<i64> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let name = validate_name(&input.folder_name, "Folder name")?;
    validate_hex_color(&input.folder_color)?;
    ensure_category_allows_folders(pool, input.category_id).await?;

    let result = sqlx::query!(
        "INSERT INTO folder (category_id, folder_name, description, folder_color) VALUES (?, ?, ?, ?)",
        input.category_id,
        name,
        input.description,
        input.folder_color
    )
    .execute(pool)
    .await;
    let result = map_unique(result, "Folder already exists in this category.")?;
    let id = result.last_insert_rowid();
    write_audit_log(
        pool,
        "INSERT",
        Some("folder"),
        Some(id),
        "Created folder",
        Some(session.user_id),
    )
    .await?;
    Ok(id)
}

pub async fn update_folder(
    pool: &DbPool,
    session_id: &str,
    folder_id: i64,
    input: FolderInput,
    is_active: bool,
) -> AppResult<()> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let name = validate_name(&input.folder_name, "Folder name")?;
    validate_hex_color(&input.folder_color)?;
    ensure_category_allows_folders(pool, input.category_id).await?;
    let exists = sqlx::query!(
        "SELECT folder_id FROM folder WHERE folder_id = ?",
        folder_id
    )
    .fetch_optional(pool)
    .await?;
    if exists.is_none() {
        return Err(AppError::NotFound("Folder not found.".into()));
    }

    let is_active_int = if is_active { 1 } else { 0 };
    let now = now_text();
    let result = sqlx::query!(
        "UPDATE folder
         SET category_id = ?, folder_name = ?, description = ?, folder_color = ?, is_active = ?, updated_at = ?
         WHERE folder_id = ?",
        input.category_id,
        name,
        input.description,
        input.folder_color,
        is_active_int,
        now,
        folder_id
    )
    .execute(pool)
    .await;
    map_unique(result, "Folder already exists in this category.")?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("folder"),
        Some(folder_id),
        "Updated folder",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn list_offices(
    pool: &DbPool,
    session_id: &str,
    include_inactive: Option<bool>,
) -> AppResult<Vec<OfficeItem>> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let include_inactive = if include_inactive.unwrap_or(false) { 1_i64 } else { 0_i64 };
    let rows = sqlx::query!(
        "SELECT office_id AS \"office_id!: i64\", office_name, description, is_active
         FROM office
         WHERE (? = 1 OR is_active = 1)
         ORDER BY office_name COLLATE NOCASE ASC",
        include_inactive
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

pub async fn create_office(pool: &DbPool, session_id: &str, input: OfficeInput) -> AppResult<i64> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let name = validate_name(&input.office_name, "Office name")?;
    let result = sqlx::query!(
        "INSERT INTO office (office_name, description) VALUES (?, ?)",
        name,
        input.description
    )
    .execute(pool)
    .await;
    let result = map_unique(result, "Office already exists.")?;
    let id = result.last_insert_rowid();
    write_audit_log(
        pool,
        "INSERT",
        Some("office"),
        Some(id),
        "Created office",
        Some(session.user_id),
    )
    .await?;
    Ok(id)
}

pub async fn update_office(
    pool: &DbPool,
    session_id: &str,
    office_id: i64,
    input: OfficeInput,
    is_active: bool,
) -> AppResult<()> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let name = validate_name(&input.office_name, "Office name")?;
    let is_active_int = if is_active { 1 } else { 0 };
    let now = now_text();
    let result = sqlx::query!(
        "UPDATE office SET office_name = ?, description = ?, is_active = ?, updated_at = ? WHERE office_id = ?",
        name,
        input.description,
        is_active_int,
        now,
        office_id
    )
    .execute(pool)
    .await;
    let result = map_unique(result, "Office already exists.")?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Office not found.".into()));
    }
    write_audit_log(
        pool,
        "UPDATE",
        Some("office"),
        Some(office_id),
        "Updated office",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

async fn ensure_category_allows_folders(pool: &DbPool, category_id: i64) -> AppResult<()> {
    let category = sqlx::query!(
        "SELECT is_system AS \"is_system!: i64\", is_active AS \"is_active!: i64\" FROM category WHERE category_id = ?",
        category_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Category not found.".into()))?;
    if category.is_system == 1 {
        return Err(AppError::SystemRecord);
    }
    if category.is_active != 1 {
        return Err(AppError::Validation("Category is inactive.".into()));
    }
    Ok(())
}

fn validate_name(value: &str, label: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 100 {
        return Err(AppError::Validation(format!("{label} must be 1 to 100 characters.")));
    }
    Ok(trimmed.to_owned())
}

fn validate_hex_color(value: &str) -> AppResult<()> {
    let bytes = value.as_bytes();
    let valid = bytes.len() == 7
        && bytes[0] == b'#'
        && bytes[1..].iter().all(|byte| byte.is_ascii_hexdigit());
    if valid {
        Ok(())
    } else {
        Err(AppError::Validation("Color must be #RRGGBB.".into()))
    }
}

fn map_unique(
    result: Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>,
    message: &str,
) -> AppResult<sqlx::sqlite::SqliteQueryResult> {
    match result {
        Ok(result) => Ok(result),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(AppError::Duplicate(message.into()))
        }
        Err(err) => Err(AppError::Database(err)),
    }
}

fn now_text() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

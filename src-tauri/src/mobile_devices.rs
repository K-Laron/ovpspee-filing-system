use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::{require_admin_role, require_session, write_audit_log},
    db::DbPool,
    error::{AppError, AppResult},
    util::{map_unique, now_text},
};

#[derive(Debug, Clone, Serialize)]
pub struct CreatedMobileDevice {
    pub device_id: String,
    pub device_name: String,
    pub device_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MobileDeviceItem {
    pub mobile_device_id: i64,
    pub device_id: String,
    pub device_name: String,
    pub is_active: bool,
    pub last_seen_at: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn create_mobile_device(
    pool: &DbPool,
    session_id: &str,
    device_name: &str,
) -> AppResult<CreatedMobileDevice> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let device_name = require_device_name(device_name)?;
    let device_id = format!("device-{}", Uuid::new_v4());
    let device_token = format!("ovpspee-{}-{}", Uuid::new_v4(), Uuid::new_v4());
    let token_hash = hash_token(&device_token);
    let now = now_text();

    let result = sqlx::query(
        "INSERT INTO mobile_device
         (device_id, device_name, token_hash, created_by, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&device_id)
    .bind(&device_name)
    .bind(&token_hash)
    .bind(session.user_id)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await;

    let result = map_unique(result, "Mobile device already exists.")?;
    write_audit_log(
        pool,
        "INSERT",
        Some("mobile_device"),
        Some(result.last_insert_rowid()),
        "Created mobile device token",
        Some(session.user_id),
    )
    .await?;

    Ok(CreatedMobileDevice {
        device_id,
        device_name,
        device_token,
    })
}

pub async fn list_mobile_devices(
    pool: &DbPool,
    session_id: &str,
) -> AppResult<Vec<MobileDeviceItem>> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let rows = sqlx::query(
        "SELECT mobile_device_id, device_id, device_name, is_active, last_seen_at, created_by, created_at, updated_at
         FROM mobile_device
         ORDER BY is_active DESC, device_name COLLATE NOCASE ASC, mobile_device_id DESC",
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_item).collect()
}

pub async fn revoke_mobile_device(
    pool: &DbPool,
    session_id: &str,
    device_id: &str,
) -> AppResult<()> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let device_id = require_device_id(device_id)?;
    let now = now_text();
    let result = sqlx::query(
        "UPDATE mobile_device
         SET is_active = 0, updated_at = ?
         WHERE device_id = ?",
    )
    .bind(&now)
    .bind(&device_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Mobile device not found.".into()));
    }

    write_audit_log(
        pool,
        "UPDATE",
        Some("mobile_device"),
        None,
        "Revoked mobile device token",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn validate_mobile_device(pool: &DbPool, device_id: &str, token: &str) -> AppResult<()> {
    let device_id = require_device_id(device_id)?;
    let token = token.trim();
    if token.is_empty() {
        return Err(AppError::Unauthorized);
    }
    let row = sqlx::query("SELECT token_hash, is_active FROM mobile_device WHERE device_id = ?")
        .bind(&device_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::Unauthorized)?;
    let token_hash: String = row.try_get("token_hash")?;
    let is_active: i64 = row.try_get("is_active")?;
    if is_active != 1 || !constant_time_eq(&token_hash, &hash_token(token)) {
        return Err(AppError::Unauthorized);
    }

    let now = now_text();
    sqlx::query("UPDATE mobile_device SET last_seen_at = ?, updated_at = ? WHERE device_id = ?")
        .bind(&now)
        .bind(&now)
        .bind(&device_id)
        .execute(pool)
        .await?;
    Ok(())
}

fn row_to_item(row: sqlx::sqlite::SqliteRow) -> AppResult<MobileDeviceItem> {
    Ok(MobileDeviceItem {
        mobile_device_id: row.try_get("mobile_device_id")?,
        device_id: row.try_get("device_id")?,
        device_name: row.try_get("device_name")?,
        is_active: row.try_get::<i64, _>("is_active")? == 1,
        last_seen_at: row.try_get("last_seen_at")?,
        created_by: row.try_get("created_by")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn require_device_name(value: &str) -> AppResult<String> {
    let value = value.trim();
    let len = value.chars().count();
    if !(2..=80).contains(&len) {
        return Err(AppError::Validation(
            "Device name must be 2 to 80 characters.".into(),
        ));
    }
    Ok(value.to_owned())
}

fn require_device_id(value: &str) -> AppResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 120 {
        return Err(AppError::Unauthorized);
    }
    Ok(value.to_owned())
}

fn hash_token(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0_u8, |diff, (left, right)| diff | (left ^ right))
        == 0
}



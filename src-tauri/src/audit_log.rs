use serde::Serialize;
use sqlx::{QueryBuilder, Row};

use crate::{
    auth::{require_admin_role, require_session, write_audit_log},
    db::DbPool,
    error::{AppError, AppResult},
};

const DEFAULT_RETENTION_MONTHS: i64 = 36;
const MIN_RETENTION_MONTHS: i64 = 24;
const MAX_RETENTION_MONTHS: i64 = 36;
const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;

#[derive(Debug, Clone, Default)]
pub struct AuditLogFilter {
    pub search: Option<String>,
    pub actor_user_id: Option<i64>,
    pub actor_search: Option<String>,
    pub action: Option<String>,
    pub entity_type: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub action: String,
    pub actor_user_id: Option<i64>,
    pub actor_username: Option<String>,
    pub actor_display_name: Option<String>,
    pub actor_role: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    pub summary: String,
    pub created_at: String,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditLogPage {
    pub entries: Vec<AuditLogEntry>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditRetentionSettings {
    pub retention_months: i64,
    pub min_months: i64,
    pub max_months: i64,
    pub cleanup_deferred: bool,
}

pub async fn list_audit_logs(
    pool: &DbPool,
    session_id: &str,
    filter: AuditLogFilter,
) -> AppResult<AuditLogPage> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    fetch_audit_logs(pool, filter, None).await
}

pub async fn list_my_activity(
    pool: &DbPool,
    session_id: &str,
    filter: AuditLogFilter,
) -> AppResult<AuditLogPage> {
    let session = require_session(pool, session_id).await?;
    if session.role != "Secretary" {
        return Err(AppError::Unauthorized);
    }
    fetch_audit_logs(pool, filter, Some(session.user_id)).await
}

pub async fn list_audit_event_types(pool: &DbPool, session_id: &str) -> AppResult<Vec<String>> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    list_event_types_for_user(pool, None).await
}

pub async fn list_my_activity_event_types(
    pool: &DbPool,
    session_id: &str,
) -> AppResult<Vec<String>> {
    let session = require_session(pool, session_id).await?;
    if session.role != "Secretary" {
        return Err(AppError::Unauthorized);
    }
    list_event_types_for_user(pool, Some(session.user_id)).await
}

pub async fn get_audit_retention_settings(
    pool: &DbPool,
    session_id: &str,
) -> AppResult<AuditRetentionSettings> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let retention_months = read_retention_months(pool).await?;
    Ok(settings_payload(retention_months))
}

pub async fn update_audit_retention_settings(
    pool: &DbPool,
    session_id: &str,
    retention_months: i64,
) -> AppResult<AuditRetentionSettings> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    validate_retention(retention_months)?;
    let value = retention_months.to_string();
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at)
         VALUES ('audit_log_retention_months', ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(value)
    .bind(now)
    .execute(pool)
    .await?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("settings"),
        None,
        "Updated audit retention setting",
        Some(session.user_id),
    )
    .await?;
    Ok(settings_payload(retention_months))
}

async fn fetch_audit_logs(
    pool: &DbPool,
    filter: AuditLogFilter,
    current_user_only: Option<i64>,
) -> AppResult<AuditLogPage> {
    let limit = normalize_limit(filter.limit);
    let offset = filter.offset.unwrap_or(0).max(0);
    let mut query = QueryBuilder::new(
        "SELECT
            a.log_id,
            a.log_action,
            a.table_affected,
            a.record_id,
            a.description,
            a.user_id,
            a.ip_address,
            a.timestamp,
            u.username,
            u.first_name,
            u.last_name,
            r.role_name
         FROM audit_log a
         LEFT JOIN user u ON u.user_id = a.user_id
         LEFT JOIN role r ON r.role_id = u.role_id
         WHERE 1 = 1",
    );

    if let Some(user_id) = current_user_only {
        query.push(" AND a.user_id = ").push_bind(user_id);
    }
    if let Some(user_id) = filter.actor_user_id {
        query.push(" AND a.user_id = ").push_bind(user_id);
    }
    if let Some(value) = trimmed(filter.action) {
        query.push(" AND a.log_action = ").push_bind(value);
    }
    if let Some(value) = trimmed(filter.entity_type) {
        query.push(" AND a.table_affected = ").push_bind(value);
    }
    if let Some(value) = trimmed(filter.date_from) {
        query.push(" AND a.timestamp >= ").push_bind(value);
    }
    if let Some(value) = trimmed(filter.date_to) {
        query.push(" AND a.timestamp <= ").push_bind(value);
    }
    if let Some(value) = trimmed(filter.search) {
        let like = format!("%{value}%");
        query
            .push(" AND (a.description LIKE ")
            .push_bind(like.clone())
            .push(" OR a.log_action LIKE ")
            .push_bind(like.clone())
            .push(" OR a.table_affected LIKE ")
            .push_bind(like.clone())
            .push(" OR u.username LIKE ")
            .push_bind(like.clone())
            .push(" OR u.first_name LIKE ")
            .push_bind(like.clone())
            .push(" OR u.last_name LIKE ")
            .push_bind(like)
            .push(")");
    }
    if let Some(value) = trimmed(filter.actor_search) {
        let like = format!("%{value}%");
        query
            .push(" AND (u.username LIKE ")
            .push_bind(like.clone())
            .push(" OR u.first_name LIKE ")
            .push_bind(like.clone())
            .push(" OR u.last_name LIKE ")
            .push_bind(like)
            .push(")");
    }

    query
        .push(" ORDER BY a.timestamp DESC, a.log_id DESC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let rows = query.build().fetch_all(pool).await?;
    let entries = rows
        .into_iter()
        .map(|row| AuditLogEntry {
            id: row.get::<i64, _>("log_id"),
            action: row.get::<String, _>("log_action"),
            actor_user_id: row.get::<Option<i64>, _>("user_id"),
            actor_username: row.get::<Option<String>, _>("username"),
            actor_display_name: display_name(
                row.get::<Option<String>, _>("first_name"),
                row.get::<Option<String>, _>("last_name"),
            ),
            actor_role: row.get::<Option<String>, _>("role_name"),
            entity_type: row.get::<Option<String>, _>("table_affected"),
            entity_id: row.get::<Option<i64>, _>("record_id"),
            summary: sanitize_summary(&row.get::<String, _>("description")),
            created_at: row.get::<String, _>("timestamp"),
            ip_address: row.get::<Option<String>, _>("ip_address"),
        })
        .collect();

    Ok(AuditLogPage {
        entries,
        limit,
        offset,
    })
}

async fn list_event_types_for_user(pool: &DbPool, user_id: Option<i64>) -> AppResult<Vec<String>> {
    let rows = if let Some(user_id) = user_id {
        sqlx::query(
            "SELECT DISTINCT log_action AS \"log_action!: String\"
             FROM audit_log
             WHERE user_id = ?
             ORDER BY log_action",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            "SELECT DISTINCT log_action AS \"log_action!: String\"
             FROM audit_log
             ORDER BY log_action",
        )
        .fetch_all(pool)
        .await?
    };
    Ok(rows
        .into_iter()
        .map(|row| row.get::<String, _>("log_action"))
        .collect())
}

async fn read_retention_months(pool: &DbPool) -> AppResult<i64> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = 'audit_log_retention_months'")
        .fetch_optional(pool)
        .await?;
    let Some(row) = row else {
        seed_default_retention(pool).await?;
        return Ok(DEFAULT_RETENTION_MONTHS);
    };
    let parsed = row
        .get::<String, _>("value")
        .parse::<i64>()
        .unwrap_or(DEFAULT_RETENTION_MONTHS);
    if (MIN_RETENTION_MONTHS..=MAX_RETENTION_MONTHS).contains(&parsed) {
        Ok(parsed)
    } else {
        Ok(DEFAULT_RETENTION_MONTHS)
    }
}

async fn seed_default_retention(pool: &DbPool) -> AppResult<()> {
    let value = DEFAULT_RETENTION_MONTHS.to_string();
    sqlx::query(
        "INSERT OR IGNORE INTO settings (key, value)
         VALUES ('audit_log_retention_months', ?)",
    )
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

fn validate_retention(retention_months: i64) -> AppResult<()> {
    if !(MIN_RETENTION_MONTHS..=MAX_RETENTION_MONTHS).contains(&retention_months) {
        return Err(AppError::Validation(
            "Audit retention must be between 24 and 36 months.".into(),
        ));
    }
    Ok(())
}

fn settings_payload(retention_months: i64) -> AuditRetentionSettings {
    AuditRetentionSettings {
        retention_months,
        min_months: MIN_RETENTION_MONTHS,
        max_months: MAX_RETENTION_MONTHS,
        cleanup_deferred: true,
    }
}

fn normalize_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

fn trimmed(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn display_name(first_name: Option<String>, last_name: Option<String>) -> Option<String> {
    let name = [first_name, last_name]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    if name.trim().is_empty() {
        None
    } else {
        Some(name)
    }
}

fn sanitize_summary(description: &str) -> String {
    let lower = description.to_ascii_lowercase();
    if lower.contains("password_hash")
        || lower.contains("$argon2")
        || lower.contains("password=")
        || lower.contains("password:")
        || lower.contains("new_password")
        || lower.contains("current_password")
        || lower.contains("secret_key")
        || lower.contains("token=")
        || lower.contains("token:")
    {
        "Sensitive audit details redacted.".to_owned()
    } else {
        description.to_owned()
    }
}

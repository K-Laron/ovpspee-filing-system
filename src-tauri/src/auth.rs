use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use chrono::{Duration, SecondsFormat, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    db::DbPool,
    error::{AppError, AppResult},
    util::require_non_empty,
};

const PASSWORD_SPECIALS: &str = "!@#$%^&*()-_=+[]{}|;:,.<>?";
const SESSION_HOURS: i64 = 8;

#[derive(Debug, Clone, Serialize)]
pub struct SessionPayload {
    pub session_id: String,
    pub user_id: i64,
    pub role: String,
    pub display_name: String,
    pub profile_pic_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidSession {
    pub user_id: i64,
    pub role: String,
}

pub fn validate_password(password: &str) -> AppResult<()> {
    if password.chars().count() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters.".into(),
        ));
    }
    if !password.chars().any(|ch| ch.is_ascii_digit()) {
        return Err(AppError::Validation(
            "Password must include at least one number.".into(),
        ));
    }
    if !password.chars().any(|ch| PASSWORD_SPECIALS.contains(ch)) {
        return Err(AppError::Validation(
            "Password must include at least one special character.".into(),
        ));
    }
    Ok(())
}

pub fn validate_username(username: &str) -> AppResult<()> {
    let length = username.chars().count();
    if !(3..=50).contains(&length) {
        return Err(AppError::Validation(
            "Username must be 3 to 50 characters.".into(),
        ));
    }
    if !username
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(AppError::Validation(
            "Username may contain only letters, numbers, underscore, and hyphen.".into(),
        ));
    }
    Ok(())
}

pub fn hash_password(password: &str) -> AppResult<String> {
    validate_password(password)?;
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(65_536, 3, 4, None)
        .map_err(|_| AppError::Validation("Invalid Argon2id parameters.".into()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::Validation("Could not hash password.".into()))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> AppResult<()> {
    let parsed = PasswordHash::new(password_hash).map_err(|_| AppError::Unauthorized)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized)
}

pub async fn first_run_required(pool: &DbPool) -> AppResult<bool> {
    let row = sqlx::query!(
        "SELECT COUNT(*) AS \"count!: i64\" FROM user u JOIN role r ON r.role_id = u.role_id WHERE r.role_name = 'Admin'"
    )
    .fetch_one(pool)
    .await?;
    Ok(row.count == 0)
}

pub async fn create_first_admin(
    pool: &DbPool,
    first_name: &str,
    last_name: &str,
    username: &str,
    password: &str,
) -> AppResult<()> {
    if !first_run_required(pool).await? {
        return Err(AppError::Conflict("Initial Admin already exists.".into()));
    }

    let first_name = require_non_empty(first_name, "First name")?;
    let last_name = require_non_empty(last_name, "Last name")?;
    let username = require_non_empty(username, "Username")?;
    validate_username(&username)?;
    let password_hash = hash_password(password)?;

    let role =
        sqlx::query!("SELECT role_id AS \"role_id!: i64\" FROM role WHERE role_name = 'Admin'")
            .fetch_one(pool)
            .await?;

    let result = sqlx::query!(
        "INSERT INTO user (role_id, first_name, last_name, username, password_hash) VALUES (?, ?, ?, ?, ?)",
        role.role_id,
        first_name,
        last_name,
        username,
        password_hash
    )
    .execute(pool)
    .await;

    let result = match result {
        Ok(result) => result,
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            return Err(AppError::Conflict("Username already exists.".into()));
        }
        Err(err) => return Err(AppError::Database(err)),
    };

    write_audit_log(
        pool,
        "INSERT",
        Some("user"),
        Some(result.last_insert_rowid()),
        "Created initial Admin account",
        Some(result.last_insert_rowid()),
    )
    .await?;

    Ok(())
}

pub async fn authenticate_user(
    pool: &DbPool,
    username: &str,
    password: &str,
) -> AppResult<SessionPayload> {
    let username = username.trim();
    let row = sqlx::query!(
        "SELECT u.user_id AS \"user_id!: i64\", u.first_name, u.last_name, u.password_hash, u.is_active, u.profile_pic_path, r.role_name
         FROM user u
         JOIN role r ON r.role_id = u.role_id
         WHERE u.username = ?",
        username
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if row.is_active != 1 {
        return Err(AppError::Unauthorized);
    }

    verify_password(password, &row.password_hash)?;

    let now = Utc::now();
    let expires_at =
        (now + Duration::hours(SESSION_HOURS)).to_rfc3339_opts(SecondsFormat::Secs, true);
    let session_id = Uuid::new_v4().to_string();
    let session_id_arg = session_id.as_str();
    let expires_at_arg = expires_at.as_str();

    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM session WHERE user_id = ?", row.user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "INSERT INTO session (session_id, user_id, expires_at) VALUES (?, ?, ?)",
        session_id_arg,
        row.user_id,
        expires_at_arg
    )
    .execute(&mut *tx)
    .await?;
    let now_text = now.to_rfc3339_opts(SecondsFormat::Secs, true);
    sqlx::query!(
        "UPDATE user SET last_login_at = ?, updated_at = ? WHERE user_id = ?",
        now_text,
        now_text,
        row.user_id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    // TODO: Include this audit log in the transaction above once write_audit_log
    // accepts an Executor rather than requiring &DbPool.
    write_audit_log(
        pool,
        "LOGIN",
        Some("user"),
        Some(row.user_id),
        "User logged in",
        Some(row.user_id),
    )
    .await?;

    Ok(SessionPayload {
        session_id: session_id.clone(),
        user_id: row.user_id,
        role: row.role_name,
        display_name: format!("{} {}", row.first_name, row.last_name),
        profile_pic_path: row.profile_pic_path,
    })
}

pub async fn validate_session(pool: &DbPool, session_id: &str) -> AppResult<SessionPayload> {
    let row = sqlx::query!(
        "SELECT s.session_id AS \"session_id!: String\", s.user_id AS \"user_id!: i64\", u.first_name, u.last_name, u.profile_pic_path, r.role_name, s.expires_at, u.is_active
         FROM session s
         JOIN user u ON u.user_id = s.user_id
         JOIN role r ON r.role_id = u.role_id
         WHERE s.session_id = ?",
        session_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if row.is_active != 1 || row.expires_at <= Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
    {
        sqlx::query!("DELETE FROM session WHERE session_id = ?", session_id)
            .execute(pool)
            .await?;
        return Err(AppError::Unauthorized);
    }

    Ok(SessionPayload {
        session_id: row.session_id,
        user_id: row.user_id,
        role: row.role_name,
        display_name: format!("{} {}", row.first_name, row.last_name),
        profile_pic_path: row.profile_pic_path,
    })
}

pub async fn require_session(pool: &DbPool, session_id: &str) -> AppResult<ValidSession> {
    let payload = validate_session(pool, session_id).await?;
    Ok(ValidSession {
        user_id: payload.user_id,
        role: payload.role,
    })
}

pub fn require_admin_role(role: &str) -> AppResult<()> {
    if role == "Admin" {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}

pub async fn require_role(pool: &DbPool, session_id: &str, allowed_roles: &[&str]) -> AppResult<ValidSession> {
    let session = require_session(pool, session_id).await?;
    if allowed_roles.iter().any(|r| *r == session.role) {
        Ok(session)
    } else {
        Err(AppError::Unauthorized)
    }
}

pub async fn require_admin(pool: &DbPool, session_id: &str) -> AppResult<ValidSession> {
    require_role(pool, session_id, &["Admin"]).await
}

pub async fn logout_session(pool: &DbPool, session_id: &str) -> AppResult<()> {
    let session = validate_session(pool, session_id).await?;
    sqlx::query!("DELETE FROM session WHERE session_id = ?", session_id)
        .execute(pool)
        .await?;
    write_audit_log(
        pool,
        "LOGOUT",
        Some("user"),
        Some(session.user_id),
        "User logged out",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn write_audit_log(
    pool: &DbPool,
    action: &str,
    table_affected: Option<&str>,
    record_id: Option<i64>,
    description: &str,
    user_id: Option<i64>,
) -> AppResult<()> {
    sqlx::query!(
        "INSERT INTO audit_log (log_action, table_affected, record_id, description, user_id, ip_address)
         VALUES (?, ?, ?, ?, ?, ?)",
        action,
        table_affected,
        record_id,
        description,
        user_id,
        "127.0.0.1"
    )
    .execute(pool)
    .await?;
    Ok(())
}



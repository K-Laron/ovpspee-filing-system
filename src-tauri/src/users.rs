use serde::{Deserialize, Serialize};

use crate::{
    auth::{
        hash_password, require_admin_role, require_session, validate_password, validate_username,
        verify_password, write_audit_log,
    },
    db::DbPool,
    error::{AppError, AppResult},
};

#[derive(Debug, Clone, Deserialize)]
pub struct UserInput {
    pub role: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub username: String,
    pub email: Option<String>,
    pub contact_number: Option<String>,
    pub address: Option<String>,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserUpdateInput {
    pub role: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub username: String,
    pub email: Option<String>,
    pub contact_number: Option<String>,
    pub address: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileInput {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub email: Option<String>,
    pub contact_number: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserItem {
    pub user_id: i64,
    pub role: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub username: String,
    pub email: Option<String>,
    pub contact_number: Option<String>,
    pub address: Option<String>,
    pub profile_pic_path: Option<String>,
    pub is_active: bool,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileItem {
    pub user_id: i64,
    pub role: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub username: String,
    pub email: Option<String>,
    pub contact_number: Option<String>,
    pub address: Option<String>,
    pub profile_pic_path: Option<String>,
}

pub async fn list_users(
    pool: &DbPool,
    session_id: &str,
    search: Option<&str>,
) -> AppResult<Vec<UserItem>> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;

    let search = search.map(str::trim).filter(|value| !value.is_empty());
    let pattern = search.map(|value| format!("%{}%", value));
    let rows = sqlx::query!(
        "SELECT u.user_id AS \"user_id!: i64\", r.role_name, u.first_name, u.middle_name, u.last_name,
            u.username, u.email, u.contact_number, u.address, u.profile_pic_path, u.is_active,
            u.last_login_at, u.created_at, u.updated_at
         FROM user u
         JOIN role r ON r.role_id = u.role_id
         WHERE (? IS NULL OR u.username LIKE ? OR u.first_name LIKE ? OR u.last_name LIKE ? OR u.email LIKE ?)
         ORDER BY u.is_active DESC, u.last_name COLLATE NOCASE ASC, u.first_name COLLATE NOCASE ASC",
        pattern,
        pattern,
        pattern,
        pattern,
        pattern
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| UserItem {
            user_id: row.user_id,
            role: row.role_name,
            first_name: row.first_name,
            middle_name: row.middle_name,
            last_name: row.last_name,
            username: row.username,
            email: row.email,
            contact_number: row.contact_number,
            address: row.address,
            profile_pic_path: row.profile_pic_path,
            is_active: row.is_active == 1,
            last_login_at: row.last_login_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect())
}

pub async fn create_user(pool: &DbPool, session_id: &str, input: UserInput) -> AppResult<i64> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;

    let role_id = role_id(pool, &input.role).await?;
    let first_name = require_non_empty(&input.first_name, "First name")?;
    let last_name = require_non_empty(&input.last_name, "Last name")?;
    let username = normalize_username(&input.username)?;
    let email = normalize_optional(input.email);
    let password_hash = hash_password(&input.password)?;

    let result = sqlx::query!(
        "INSERT INTO user (role_id, first_name, middle_name, last_name, username, email, contact_number, address, password_hash)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        role_id,
        first_name,
        input.middle_name,
        last_name,
        username,
        email,
        input.contact_number,
        input.address,
        password_hash
    )
    .execute(pool)
    .await;

    let result = map_unique(result, "Username or email already exists.")?;
    let user_id = result.last_insert_rowid();
    write_audit_log(
        pool,
        "INSERT",
        Some("user"),
        Some(user_id),
        "Created user account",
        Some(session.user_id),
    )
    .await?;
    Ok(user_id)
}

pub async fn update_user(
    pool: &DbPool,
    session_id: &str,
    user_id: i64,
    input: UserUpdateInput,
) -> AppResult<()> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;

    let role_id = role_id(pool, &input.role).await?;
    let first_name = require_non_empty(&input.first_name, "First name")?;
    let last_name = require_non_empty(&input.last_name, "Last name")?;
    let username = normalize_username(&input.username)?;
    let email = normalize_optional(input.email);
    let is_active = if input.is_active { 1_i64 } else { 0_i64 };
    let now = now_text();

    let result = sqlx::query!(
        "UPDATE user
         SET role_id = ?, first_name = ?, middle_name = ?, last_name = ?, username = ?, email = ?,
             contact_number = ?, address = ?, is_active = ?, updated_at = ?
         WHERE user_id = ?",
        role_id,
        first_name,
        input.middle_name,
        last_name,
        username,
        email,
        input.contact_number,
        input.address,
        is_active,
        now,
        user_id
    )
    .execute(pool)
    .await;

    let result = map_unique(result, "Username or email already exists.")?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found.".into()));
    }

    if !input.is_active {
        sqlx::query!("DELETE FROM session WHERE user_id = ?", user_id)
            .execute(pool)
            .await?;
    }

    write_audit_log(
        pool,
        "UPDATE",
        Some("user"),
        Some(user_id),
        if input.is_active { "Updated user account" } else { "Updated and deactivated user account" },
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn admin_reset_password(
    pool: &DbPool,
    session_id: &str,
    user_id: i64,
    new_password: &str,
) -> AppResult<()> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    validate_password(new_password)?;
    let password_hash = hash_password(new_password)?;
    let now = now_text();

    let result = sqlx::query!(
        "UPDATE user SET password_hash = ?, updated_at = ? WHERE user_id = ?",
        password_hash,
        now,
        user_id
    )
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found.".into()));
    }
    sqlx::query!("DELETE FROM session WHERE user_id = ?", user_id)
        .execute(pool)
        .await?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("user"),
        Some(user_id),
        "Admin reset user password",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn get_my_profile(pool: &DbPool, session_id: &str) -> AppResult<ProfileItem> {
    let session = require_session(pool, session_id).await?;
    let row = sqlx::query!(
        "SELECT u.user_id AS \"user_id!: i64\", r.role_name, u.first_name, u.middle_name, u.last_name,
            u.username, u.email, u.contact_number, u.address, u.profile_pic_path
         FROM user u
         JOIN role r ON r.role_id = u.role_id
         WHERE u.user_id = ?",
        session.user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(ProfileItem {
        user_id: row.user_id,
        role: row.role_name,
        first_name: row.first_name,
        middle_name: row.middle_name,
        last_name: row.last_name,
        username: row.username,
        email: row.email,
        contact_number: row.contact_number,
        address: row.address,
        profile_pic_path: row.profile_pic_path,
    })
}

pub async fn update_my_profile(
    pool: &DbPool,
    session_id: &str,
    input: ProfileInput,
) -> AppResult<()> {
    let session = require_session(pool, session_id).await?;
    let first_name = require_non_empty(&input.first_name, "First name")?;
    let last_name = require_non_empty(&input.last_name, "Last name")?;
    let email = normalize_optional(input.email);
    let now = now_text();

    let result = sqlx::query!(
        "UPDATE user
         SET first_name = ?, middle_name = ?, last_name = ?, email = ?, contact_number = ?, address = ?, updated_at = ?
         WHERE user_id = ?",
        first_name,
        input.middle_name,
        last_name,
        email,
        input.contact_number,
        input.address,
        now,
        session.user_id
    )
    .execute(pool)
    .await;
    map_unique(result, "Email already exists.")?;

    write_audit_log(
        pool,
        "UPDATE",
        Some("user"),
        Some(session.user_id),
        "Updated own profile",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

pub async fn change_my_password(
    pool: &DbPool,
    session_id: &str,
    current_password: &str,
    new_password: &str,
) -> AppResult<()> {
    let session = require_session(pool, session_id).await?;
    let row = sqlx::query!(
        "SELECT password_hash FROM user WHERE user_id = ?",
        session.user_id
    )
    .fetch_one(pool)
    .await?;
    verify_password(current_password, &row.password_hash)?;
    validate_password(new_password)?;
    let password_hash = hash_password(new_password)?;
    let now = now_text();

    sqlx::query!(
        "UPDATE user SET password_hash = ?, updated_at = ? WHERE user_id = ?",
        password_hash,
        now,
        session.user_id
    )
    .execute(pool)
    .await?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("user"),
        Some(session.user_id),
        "Changed own password",
        Some(session.user_id),
    )
    .await?;
    Ok(())
}

// TODO(Slice 3 follow-up): Implement profile picture upload with file validation, size limits,
// safe app-data storage, and path-only audit logging. Avoid accepting arbitrary filesystem paths.

async fn role_id(pool: &DbPool, role: &str) -> AppResult<i64> {
    match role {
        "Admin" | "Secretary" => {}
        _ => return Err(AppError::Validation("Role must be Admin or Secretary.".into())),
    }
    let row = sqlx::query!(
        "SELECT role_id AS \"role_id!: i64\" FROM role WHERE role_name = ?",
        role
    )
    .fetch_one(pool)
    .await?;
    Ok(row.role_id)
}

fn normalize_username(value: &str) -> AppResult<String> {
    let username = require_non_empty(value, "Username")?;
    validate_username(&username)?;
    Ok(username)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_owned();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn require_non_empty(value: &str, label: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!("{label} is required.")));
    }
    Ok(trimmed.to_owned())
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

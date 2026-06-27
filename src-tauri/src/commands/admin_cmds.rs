use tauri::{AppHandle, State};

use crate::audit_log::{self, AuditLogFilter, AuditLogPage, AuditRetentionSettings};
use crate::backup::{self, BackupSettings, BackupSettingsInput, BackupSummary, BackupValidation, RestoreResult};
use crate::db::DbState;
use crate::master_data::{self, CategoryInput, CategoryItem, FolderInput, FolderItem, OfficeInput, OfficeItem};
use crate::users::{self, ProfileInput, ProfileItem, UserInput, UserItem, UserUpdateInput};

use super::{backup_runtime, CmdResult};

#[tauri::command]
pub async fn list_categories(
    db: State<'_, DbState>,
    session_id: String,
    include_inactive: Option<bool>,
) -> CmdResult<Vec<CategoryItem>> {
    master_data::list_categories(&db.pool, &session_id, include_inactive).await
}

#[tauri::command]
pub async fn create_category(
    db: State<'_, DbState>,
    session_id: String,
    category_name: String,
    description: Option<String>,
    color_code: String,
    icon: Option<String>,
) -> CmdResult<i64> {
    master_data::create_category(
        &db.pool, &session_id,
        CategoryInput { category_name, description, color_code, icon },
    ).await
}

#[tauri::command]
pub async fn update_category(
    db: State<'_, DbState>,
    session_id: String,
    category_id: i64,
    category_name: String,
    description: Option<String>,
    color_code: String,
    icon: Option<String>,
    is_active: bool,
) -> CmdResult<()> {
    master_data::update_category(
        &db.pool, &session_id, category_id,
        CategoryInput { category_name, description, color_code, icon },
        is_active,
    ).await
}

#[tauri::command]
pub async fn list_folders(
    db: State<'_, DbState>,
    session_id: String,
    category_id: Option<i64>,
    include_inactive: Option<bool>,
) -> CmdResult<Vec<FolderItem>> {
    master_data::list_folders(&db.pool, &session_id, category_id, include_inactive).await
}

#[tauri::command]
pub async fn create_folder(
    db: State<'_, DbState>,
    session_id: String,
    category_id: i64,
    folder_name: String,
    description: Option<String>,
    folder_color: String,
) -> CmdResult<i64> {
    master_data::create_folder(
        &db.pool, &session_id,
        FolderInput { category_id, folder_name, description, folder_color },
    ).await
}

#[tauri::command]
pub async fn update_folder(
    db: State<'_, DbState>,
    session_id: String,
    folder_id: i64,
    category_id: i64,
    folder_name: String,
    description: Option<String>,
    folder_color: String,
    is_active: bool,
) -> CmdResult<()> {
    master_data::update_folder(
        &db.pool, &session_id, folder_id,
        FolderInput { category_id, folder_name, description, folder_color },
        is_active,
    ).await
}

#[tauri::command]
pub async fn list_offices(
    db: State<'_, DbState>,
    session_id: String,
    include_inactive: Option<bool>,
) -> CmdResult<Vec<OfficeItem>> {
    master_data::list_offices(&db.pool, &session_id, include_inactive).await
}

#[tauri::command]
pub async fn create_office(
    db: State<'_, DbState>,
    session_id: String,
    office_name: String,
    description: Option<String>,
) -> CmdResult<i64> {
    master_data::create_office(
        &db.pool, &session_id,
        OfficeInput { office_name, description },
    ).await
}

#[tauri::command]
pub async fn update_office(
    db: State<'_, DbState>,
    session_id: String,
    office_id: i64,
    office_name: String,
    description: Option<String>,
    is_active: bool,
) -> CmdResult<()> {
    master_data::update_office(
        &db.pool, &session_id, office_id,
        OfficeInput { office_name, description },
        is_active,
    ).await
}

#[tauri::command]
pub async fn list_users(
    db: State<'_, DbState>,
    session_id: String,
    search: Option<String>,
) -> CmdResult<Vec<UserItem>> {
    users::list_users(&db.pool, &session_id, search.as_deref()).await
}

#[tauri::command]
pub async fn create_user(
    db: State<'_, DbState>,
    session_id: String,
    role: String,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    username: String,
    email: Option<String>,
    contact_number: Option<String>,
    address: Option<String>,
    password: String,
) -> CmdResult<i64> {
    users::create_user(
        &db.pool, &session_id,
        UserInput { role, first_name, middle_name, last_name, username, email, contact_number, address, password },
    ).await
}

#[tauri::command]
pub async fn update_user(
    db: State<'_, DbState>,
    session_id: String,
    user_id: i64,
    role: String,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    username: String,
    email: Option<String>,
    contact_number: Option<String>,
    address: Option<String>,
    is_active: bool,
) -> CmdResult<()> {
    users::update_user(
        &db.pool, &session_id, user_id,
        UserUpdateInput { role, first_name, middle_name, last_name, username, email, contact_number, address, is_active },
    ).await
}

#[tauri::command]
pub async fn admin_reset_password(
    db: State<'_, DbState>,
    session_id: String,
    user_id: i64,
    new_password: String,
) -> CmdResult<()> {
    users::admin_reset_password(&db.pool, &session_id, user_id, &new_password).await
}

#[tauri::command]
pub async fn get_my_profile(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<ProfileItem> {
    users::get_my_profile(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn update_my_profile(
    db: State<'_, DbState>,
    session_id: String,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    email: Option<String>,
    contact_number: Option<String>,
    address: Option<String>,
) -> CmdResult<()> {
    users::update_my_profile(
        &db.pool, &session_id,
        ProfileInput { first_name, middle_name, last_name, email, contact_number, address },
    ).await
}

#[tauri::command]
pub async fn change_my_password(
    db: State<'_, DbState>,
    session_id: String,
    current_password: String,
    new_password: String,
) -> CmdResult<()> {
    users::change_my_password(&db.pool, &session_id, &current_password, &new_password).await
}

#[tauri::command]
pub async fn list_audit_logs(
    db: State<'_, DbState>,
    session_id: String,
    search: Option<String>,
    actor_user_id: Option<i64>,
    actor_search: Option<String>,
    action: Option<String>,
    entity_type: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> CmdResult<AuditLogPage> {
    audit_log::list_audit_logs(
        &db.pool, &session_id,
        AuditLogFilter { search, actor_user_id, actor_search, action, entity_type, date_from, date_to, limit, offset },
    ).await
}

#[tauri::command]
pub async fn list_my_activity(
    db: State<'_, DbState>,
    session_id: String,
    search: Option<String>,
    action: Option<String>,
    entity_type: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> CmdResult<AuditLogPage> {
    audit_log::list_my_activity(
        &db.pool, &session_id,
        AuditLogFilter { search, action, entity_type, date_from, date_to, limit, offset, ..AuditLogFilter::default() },
    ).await
}

#[tauri::command]
pub async fn list_audit_event_types(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<String>> {
    audit_log::list_audit_event_types(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn list_my_activity_event_types(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<String>> {
    audit_log::list_my_activity_event_types(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn get_audit_retention_settings(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<AuditRetentionSettings> {
    audit_log::get_audit_retention_settings(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn update_audit_retention_settings(
    db: State<'_, DbState>,
    session_id: String,
    retention_months: i64,
) -> CmdResult<AuditRetentionSettings> {
    audit_log::update_audit_retention_settings(&db.pool, &session_id, retention_months).await
}

#[tauri::command]
pub async fn get_backup_settings(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<BackupSettings> {
    let runtime = backup_runtime(&app)?;
    backup::get_backup_settings(&db.pool, &runtime, &session_id).await
}

#[tauri::command]
pub async fn update_backup_settings(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    destination_path: Option<String>,
    schedule_enabled: bool,
    schedule_time: String,
    retention_count: i64,
) -> CmdResult<BackupSettings> {
    let runtime = backup_runtime(&app)?;
    backup::update_backup_settings(
        &db.pool, &runtime, &session_id,
        BackupSettingsInput { destination_path, schedule_enabled, schedule_time, retention_count },
    ).await
}

#[tauri::command]
pub async fn create_backup(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<BackupSummary> {
    let runtime = backup_runtime(&app)?;
    backup::create_backup(&db.pool, &runtime, &session_id, false).await
}

#[tauri::command]
pub async fn list_backup_history(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<BackupSummary>> {
    let runtime = backup_runtime(&app)?;
    backup::list_backup_history(&db.pool, &runtime, &session_id).await
}

#[tauri::command]
pub async fn export_backup_archive(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    backup_name: String,
    output_path: String,
) -> CmdResult<String> {
    let runtime = backup_runtime(&app)?;
    backup::export_backup_archive(&db.pool, &runtime, &session_id, backup_name, output_path).await
}

#[tauri::command]
pub async fn validate_backup_archive(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    archive_path: String,
) -> CmdResult<BackupValidation> {
    let runtime = backup_runtime(&app)?;
    backup::validate_backup_archive(&db.pool, &runtime, &session_id, archive_path).await
}

#[tauri::command]
pub async fn import_backup_archive(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    archive_path: String,
) -> CmdResult<BackupSummary> {
    let runtime = backup_runtime(&app)?;
    backup::import_backup_archive(&db.pool, &runtime, &session_id, archive_path).await
}

#[tauri::command]
pub async fn restore_from_backup(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    backup_name: String,
) -> CmdResult<RestoreResult> {
    let runtime = backup_runtime(&app)?;
    backup::restore_from_backup(&db.pool, &runtime, &session_id, backup_name).await
}

#[tauri::command]
pub async fn restore_from_backup_folder(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    folder_path: String,
) -> CmdResult<RestoreResult> {
    let runtime = backup_runtime(&app)?;
    backup::restore_from_backup_folder(&db.pool, &runtime, &session_id, folder_path).await
}

#[tauri::command]
pub async fn run_scheduled_backup_check(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Option<BackupSummary>> {
    let runtime = backup_runtime(&app)?;
    backup::run_scheduled_backup_check(&db.pool, &runtime, &session_id).await
}

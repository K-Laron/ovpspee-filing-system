use tauri::{AppHandle, Manager, State};

use crate::{
    audit_log::{self, AuditLogFilter, AuditLogPage, AuditRetentionSettings},
    error::AppError,
    auth::{self, SessionPayload},
    backup::{
        self, BackupRuntime, BackupSettings, BackupSettingsInput, BackupSummary, BackupValidation,
        RestoreResult,
    },
    db::DbState,
    devices::{
        self, DeviceSettings, DeviceSettingsInput, PrinterDevice, ScanOptions, ScannerCapabilities,
        ScannerDevice,
    },
    documents::{
        self, AttachmentInput, AttachmentPreviewInfo, AttachmentPreviewPage, DocumentDetail,
        DocumentInput, DocumentItem, DocumentListFilter, StorageRoot,
    },
    master_data::{
        self, CategoryInput, CategoryItem, FolderInput, FolderItem, OfficeInput, OfficeItem,
    },
    mobile_api::{self, MobileApiSetup},
    mobile_devices::{self, CreatedMobileDevice, MobileDeviceItem},
    mobile_submissions::{
        self, MobileSubmissionAttachmentPreviewPage, MobileSubmissionDetail, MobileSubmissionItem,
    },
    printing::{self, PrintOptions, PrintResult},
    scan_intake::{self, ScanIntakeItem, ScanIntakePreviewPage},
    users::{self, ProfileInput, ProfileItem, UserInput, UserItem, UserUpdateInput},
};

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub async fn first_run_check(db: State<'_, DbState>) -> CmdResult<bool> {
    auth::first_run_required(&db.pool).await
}

#[tauri::command]
pub async fn first_run_setup(
    db: State<'_, DbState>,
    first_name: String,
    last_name: String,
    username: String,
    password: String,
) -> CmdResult<()> {
    auth::create_first_admin(&db.pool, &first_name, &last_name, &username, &password).await
}

#[tauri::command]
pub async fn login(
    db: State<'_, DbState>,
    username: String,
    password: String,
) -> CmdResult<SessionPayload> {
    auth::authenticate_user(&db.pool, &username, &password).await
}

#[tauri::command]
pub async fn logout(db: State<'_, DbState>, session_id: String) -> CmdResult<()> {
    auth::logout_session(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn validate_session(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<SessionPayload> {
    auth::validate_session(&db.pool, &session_id).await
}

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
        &db.pool,
        &session_id,
        CategoryInput {
            category_name,
            description,
            color_code,
            icon,
        },
    )
    .await
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
        &db.pool,
        &session_id,
        category_id,
        CategoryInput {
            category_name,
            description,
            color_code,
            icon,
        },
        is_active,
    )
    .await
}

#[tauri::command]
pub async fn list_folders(
    db: State<'_, DbState>,
    session_id: String,
    category_id: Option<i64>,
    include_inactive: Option<bool>,
) -> CmdResult<Vec<FolderItem>> {
    master_data::list_folders(&db.pool, &session_id, category_id, include_inactive)
        .await
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
        &db.pool,
        &session_id,
        FolderInput {
            category_id,
            folder_name,
            description,
            folder_color,
        },
    )
    .await
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
        &db.pool,
        &session_id,
        folder_id,
        FolderInput {
            category_id,
            folder_name,
            description,
            folder_color,
        },
        is_active,
    )
    .await
}

#[tauri::command]
pub async fn list_offices(
    db: State<'_, DbState>,
    session_id: String,
    include_inactive: Option<bool>,
) -> CmdResult<Vec<OfficeItem>> {
    master_data::list_offices(&db.pool, &session_id, include_inactive)
        .await
}

#[tauri::command]
pub async fn create_office(
    db: State<'_, DbState>,
    session_id: String,
    office_name: String,
    description: Option<String>,
) -> CmdResult<i64> {
    master_data::create_office(
        &db.pool,
        &session_id,
        OfficeInput {
            office_name,
            description,
        },
    )
    .await
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
        &db.pool,
        &session_id,
        office_id,
        OfficeInput {
            office_name,
            description,
        },
        is_active,
    )
    .await
}

#[tauri::command]
pub async fn list_users(
    db: State<'_, DbState>,
    session_id: String,
    search: Option<String>,
) -> CmdResult<Vec<UserItem>> {
    users::list_users(&db.pool, &session_id, search.as_deref())
        .await
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
        &db.pool,
        &session_id,
        UserInput {
            role,
            first_name,
            middle_name,
            last_name,
            username,
            email,
            contact_number,
            address,
            password,
        },
    )
    .await
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
        &db.pool,
        &session_id,
        user_id,
        UserUpdateInput {
            role,
            first_name,
            middle_name,
            last_name,
            username,
            email,
            contact_number,
            address,
            is_active,
        },
    )
    .await
}

#[tauri::command]
pub async fn admin_reset_password(
    db: State<'_, DbState>,
    session_id: String,
    user_id: i64,
    new_password: String,
) -> CmdResult<()> {
    users::admin_reset_password(&db.pool, &session_id, user_id, &new_password)
        .await
}

#[tauri::command]
pub async fn get_my_profile(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<ProfileItem> {
    users::get_my_profile(&db.pool, &session_id)
        .await
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
        &db.pool,
        &session_id,
        ProfileInput {
            first_name,
            middle_name,
            last_name,
            email,
            contact_number,
            address,
        },
    )
    .await
}

#[tauri::command]
pub async fn change_my_password(
    db: State<'_, DbState>,
    session_id: String,
    current_password: String,
    new_password: String,
) -> CmdResult<()> {
    users::change_my_password(&db.pool, &session_id, &current_password, &new_password)
        .await
}

#[tauri::command]
pub async fn create_document(
    db: State<'_, DbState>,
    session_id: String,
    document_name: String,
    category_id: i64,
    folder_id: Option<i64>,
    office_id: Option<i64>,
    date_received: String,
    remarks: Option<String>,
    status: String,
) -> CmdResult<i64> {
    documents::create_document(
        &db.pool,
        &session_id,
        DocumentInput {
            document_name,
            category_id,
            folder_id,
            office_id,
            date_received,
            remarks,
            status,
        },
    )
    .await
}

#[tauri::command]
pub async fn update_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    document_name: String,
    category_id: i64,
    folder_id: Option<i64>,
    office_id: Option<i64>,
    date_received: String,
    remarks: Option<String>,
    status: String,
) -> CmdResult<()> {
    documents::update_document(
        &db.pool,
        &session_id,
        document_id,
        DocumentInput {
            document_name,
            category_id,
            folder_id,
            office_id,
            date_received,
            remarks,
            status,
        },
    )
    .await
}

#[tauri::command]
pub async fn move_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    category_id: i64,
    folder_id: Option<i64>,
) -> CmdResult<()> {
    documents::move_document(&db.pool, &session_id, document_id, category_id, folder_id)
        .await
}

#[tauri::command]
pub async fn set_document_status(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    status: String,
) -> CmdResult<()> {
    documents::set_document_status(&db.pool, &session_id, document_id, status)
        .await
}

#[tauri::command]
pub async fn set_document_hidden(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    is_hidden: bool,
) -> CmdResult<()> {
    documents::set_document_hidden(&db.pool, &session_id, document_id, is_hidden)
        .await
}

#[tauri::command]
pub async fn trash_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> CmdResult<()> {
    documents::trash_document(&db.pool, &session_id, document_id)
        .await
}

#[tauri::command]
pub async fn restore_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> CmdResult<()> {
    documents::restore_document(&db.pool, &session_id, document_id)
        .await
}

#[tauri::command]
pub async fn list_trash_documents(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<DocumentItem>> {
    documents::list_trash_documents(&db.pool, &session_id)
        .await
}

#[tauri::command]
pub async fn purge_document(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> CmdResult<()> {
    let storage = storage_root(&app)?;
    documents::purge_document(&db.pool, &storage, &session_id, document_id)
        .await
}

#[tauri::command]
pub async fn empty_trash(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<i64> {
    let storage = storage_root(&app)?;
    documents::empty_trash(&db.pool, &storage, &session_id)
        .await
}

#[tauri::command]
pub async fn list_documents(
    db: State<'_, DbState>,
    session_id: String,
    search: Option<String>,
    category_id: Option<i64>,
    folder_id: Option<i64>,
    office_id: Option<i64>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> CmdResult<Vec<DocumentItem>> {
    documents::list_documents(
        &db.pool,
        &session_id,
        DocumentListFilter {
            search,
            category_id,
            folder_id,
            office_id,
            date_from,
            date_to,
        },
    )
    .await
}

#[tauri::command]
pub async fn get_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> CmdResult<DocumentDetail> {
    documents::get_document(&db.pool, &session_id, document_id)
        .await
}

#[tauri::command]
pub async fn add_attachment(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    source_path: String,
    sort_order: Option<i64>,
) -> CmdResult<i64> {
    let storage = storage_root(&app)?;
    documents::add_attachment(
        &db.pool,
        &storage,
        &session_id,
        document_id,
        AttachmentInput {
            source_path,
            sort_order,
        },
    )
    .await
}

#[tauri::command]
pub async fn remove_attachment(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    attachment_id: i64,
) -> CmdResult<()> {
    let storage = storage_root(&app)?;
    documents::remove_attachment(&db.pool, &storage, &session_id, attachment_id)
        .await
}

#[tauri::command]
pub async fn reorder_attachments(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    attachment_ids: Vec<i64>,
) -> CmdResult<()> {
    documents::reorder_attachments(&db.pool, &session_id, document_id, attachment_ids)
        .await
}

#[tauri::command]
pub async fn get_attachment_file_path(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: Option<String>,
    attachment_id: i64,
) -> CmdResult<String> {
    let storage = storage_root(&app)?;
    documents::get_attachment_file_path(&db.pool, &storage, session_id.as_deref(), attachment_id)
        .await
}

#[tauri::command]
pub async fn get_attachment_preview_info(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: Option<String>,
    attachment_id: i64,
) -> CmdResult<AttachmentPreviewInfo> {
    let storage = storage_root(&app)?;
    documents::get_attachment_preview_info(&db.pool, &storage, session_id.as_deref(), attachment_id)
        .await
}

#[tauri::command]
pub async fn get_attachment_preview_page(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: Option<String>,
    attachment_id: i64,
    page_number: Option<i64>,
) -> CmdResult<AttachmentPreviewPage> {
    let storage = storage_root(&app)?;
    documents::get_attachment_preview_page(
        &db.pool,
        &storage,
        session_id.as_deref(),
        attachment_id,
        page_number,
    )
    .await
}

#[tauri::command]
pub async fn export_document_pdf(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: Option<String>,
    document_id: i64,
    output_path: String,
) -> CmdResult<String> {
    let storage = storage_root(&app)?;
    documents::export_document_pdf(
        &db.pool,
        &storage,
        session_id.as_deref(),
        document_id,
        &output_path,
    )
    .await
}

#[tauri::command]
pub async fn list_public_categories(db: State<'_, DbState>) -> CmdResult<Vec<CategoryItem>> {
    documents::list_public_categories(&db.pool)
        .await
}

#[tauri::command]
pub async fn list_public_folders(
    db: State<'_, DbState>,
    category_id: i64,
) -> CmdResult<Vec<FolderItem>> {
    documents::list_public_folders(&db.pool, category_id)
        .await
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
) -> CmdResult<Vec<DocumentItem>> {
    documents::list_public_documents(
        &db.pool,
        DocumentListFilter {
            search,
            category_id,
            folder_id,
            office_id,
            date_from,
            date_to,
        },
    )
    .await
}

#[tauri::command]
pub async fn get_public_document(
    db: State<'_, DbState>,
    document_id: i64,
) -> CmdResult<DocumentDetail> {
    documents::get_public_document(&db.pool, document_id)
        .await
}

#[tauri::command]
pub async fn list_document_offices(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<OfficeItem>> {
    documents::list_document_offices(&db.pool, &session_id)
        .await
}

#[tauri::command]
pub async fn list_mobile_submissions(
    db: State<'_, DbState>,
    session_id: String,
    review_status: Option<String>,
    search: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> CmdResult<Vec<MobileSubmissionItem>> {
    mobile_submissions::list_mobile_submissions(
        &db.pool,
        &session_id,
        review_status,
        search,
        date_from,
        date_to,
    )
    .await
}

#[tauri::command]
pub async fn get_mobile_api_setup() -> CmdResult<MobileApiSetup> {
    Ok(mobile_api::setup_info())
}

#[tauri::command]
pub async fn create_mobile_device(
    db: State<'_, DbState>,
    session_id: String,
    device_name: String,
) -> CmdResult<CreatedMobileDevice> {
    mobile_devices::create_mobile_device(&db.pool, &session_id, &device_name)
        .await
}

#[tauri::command]
pub async fn list_mobile_devices(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<MobileDeviceItem>> {
    mobile_devices::list_mobile_devices(&db.pool, &session_id)
        .await
}

#[tauri::command]
pub async fn revoke_mobile_device(
    db: State<'_, DbState>,
    session_id: String,
    device_id: String,
) -> CmdResult<()> {
    mobile_devices::revoke_mobile_device(&db.pool, &session_id, &device_id)
        .await
}

#[tauri::command]
pub async fn get_mobile_submission(
    db: State<'_, DbState>,
    session_id: String,
    mobile_submission_id: i64,
) -> CmdResult<MobileSubmissionDetail> {
    mobile_submissions::get_mobile_submission(&db.pool, &session_id, mobile_submission_id)
        .await
}

#[tauri::command]
pub async fn get_mobile_submission_attachment_preview_page(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    mobile_submission_attachment_id: i64,
    page_number: Option<i64>,
) -> CmdResult<MobileSubmissionAttachmentPreviewPage> {
    let storage = storage_root(&app)?;
    mobile_submissions::get_mobile_submission_attachment_preview_page(
        &db.pool,
        &storage,
        &session_id,
        mobile_submission_attachment_id,
        page_number,
    )
    .await
}

#[tauri::command]
pub async fn approve_mobile_submission(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    mobile_submission_id: i64,
    review_notes: Option<String>,
) -> CmdResult<i64> {
    let storage = storage_root(&app)?;
    mobile_submissions::approve_mobile_submission(
        &db.pool,
        &storage,
        &session_id,
        mobile_submission_id,
        review_notes,
    )
    .await
}

#[tauri::command]
pub async fn reject_mobile_submission(
    db: State<'_, DbState>,
    session_id: String,
    mobile_submission_id: i64,
    rejection_reason: String,
) -> CmdResult<()> {
    mobile_submissions::reject_mobile_submission(
        &db.pool,
        &session_id,
        mobile_submission_id,
        rejection_reason,
    )
    .await
}

#[tauri::command]
pub async fn import_scan_files(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    source_paths: Vec<String>,
) -> CmdResult<Vec<i64>> {
    let storage = storage_root(&app)?;
    scan_intake::import_scan_files(&db.pool, &storage, &session_id, source_paths)
        .await
}

#[tauri::command]
pub async fn list_scan_intake(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<ScanIntakeItem>> {
    scan_intake::list_scan_intake(&db.pool, &session_id)
        .await
}

#[tauri::command]
pub async fn get_scan_intake(
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_id: i64,
) -> CmdResult<ScanIntakeItem> {
    scan_intake::get_scan_intake(&db.pool, &session_id, scan_intake_id)
        .await
}

#[tauri::command]
pub async fn get_scan_intake_preview_page(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_id: i64,
    page_number: Option<i64>,
) -> CmdResult<ScanIntakePreviewPage> {
    let storage = storage_root(&app)?;
    scan_intake::get_scan_intake_preview_page(
        &db.pool,
        &storage,
        &session_id,
        scan_intake_id,
        page_number,
    )
    .await
}

#[tauri::command]
pub async fn update_scan_intake_notes(
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_id: i64,
    notes: Option<String>,
) -> CmdResult<()> {
    scan_intake::update_scan_intake_notes(&db.pool, &session_id, scan_intake_id, notes)
        .await
}

#[tauri::command]
pub async fn remove_scan_intake(
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_id: i64,
) -> CmdResult<()> {
    scan_intake::remove_scan_intake(&db.pool, &session_id, scan_intake_id)
        .await
}

#[tauri::command]
pub async fn file_scan_as_document(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_ids: Vec<i64>,
    document_name: String,
    category_id: i64,
    folder_id: Option<i64>,
    office_id: Option<i64>,
    date_received: String,
    remarks: Option<String>,
    status: String,
) -> CmdResult<i64> {
    let storage = storage_root(&app)?;
    scan_intake::file_scan_as_document(
        &db.pool,
        &storage,
        &session_id,
        scan_intake_ids,
        DocumentInput {
            document_name,
            category_id,
            folder_id,
            office_id,
            date_received,
            remarks,
            status,
        },
    )
    .await
}

#[tauri::command]
pub async fn attach_scan_to_document(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    scan_intake_ids: Vec<i64>,
    document_id: i64,
) -> CmdResult<Vec<i64>> {
    let storage = storage_root(&app)?;
    scan_intake::attach_scan_to_document(
        &db.pool,
        &storage,
        &session_id,
        scan_intake_ids,
        document_id,
    )
    .await
}

#[tauri::command]
pub async fn get_scanner_capabilities(
    db: State<'_, DbState>,
    session_id: String,
    scanner_id: String,
) -> CmdResult<ScannerCapabilities> {
    devices::get_scanner_capabilities(&db.pool, &session_id, &scanner_id)
        .await
}

#[tauri::command]
pub async fn scan_to_intake(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    scanner_id: String,
    options: ScanOptions,
) -> CmdResult<ScanIntakeItem> {
    let storage = storage_root(&app)?;
    devices::scan_to_intake(&db.pool, &storage, &session_id, &scanner_id, options)
        .await
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
        &db.pool,
        &session_id,
        AuditLogFilter {
            search,
            actor_user_id,
            actor_search,
            action,
            entity_type,
            date_from,
            date_to,
            limit,
            offset,
        },
    )
    .await
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
        &db.pool,
        &session_id,
        AuditLogFilter {
            search,
            action,
            entity_type,
            date_from,
            date_to,
            limit,
            offset,
            ..AuditLogFilter::default()
        },
    )
    .await
}

#[tauri::command]
pub async fn list_audit_event_types(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<String>> {
    audit_log::list_audit_event_types(&db.pool, &session_id)
        .await
}

#[tauri::command]
pub async fn list_my_activity_event_types(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<String>> {
    audit_log::list_my_activity_event_types(&db.pool, &session_id)
        .await
}

#[tauri::command]
pub async fn get_audit_retention_settings(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<AuditRetentionSettings> {
    audit_log::get_audit_retention_settings(&db.pool, &session_id)
        .await
}

#[tauri::command]
pub async fn update_audit_retention_settings(
    db: State<'_, DbState>,
    session_id: String,
    retention_months: i64,
) -> CmdResult<AuditRetentionSettings> {
    audit_log::update_audit_retention_settings(&db.pool, &session_id, retention_months)
        .await
}

#[tauri::command]
pub async fn get_backup_settings(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<BackupSettings> {
    let runtime = backup_runtime(&app)?;
    backup::get_backup_settings(&db.pool, &runtime, &session_id)
        .await
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
        &db.pool,
        &runtime,
        &session_id,
        BackupSettingsInput {
            destination_path,
            schedule_enabled,
            schedule_time,
            retention_count,
        },
    )
    .await
}

#[tauri::command]
pub async fn create_backup(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<BackupSummary> {
    let runtime = backup_runtime(&app)?;
    backup::create_backup(&db.pool, &runtime, &session_id, false)
        .await
}

#[tauri::command]
pub async fn list_backup_history(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<BackupSummary>> {
    let runtime = backup_runtime(&app)?;
    backup::list_backup_history(&db.pool, &runtime, &session_id)
        .await
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
    backup::export_backup_archive(&db.pool, &runtime, &session_id, backup_name, output_path)
        .await
}

#[tauri::command]
pub async fn validate_backup_archive(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    archive_path: String,
) -> CmdResult<BackupValidation> {
    let runtime = backup_runtime(&app)?;
    backup::validate_backup_archive(&db.pool, &runtime, &session_id, archive_path)
        .await
}

#[tauri::command]
pub async fn import_backup_archive(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    archive_path: String,
) -> CmdResult<BackupSummary> {
    let runtime = backup_runtime(&app)?;
    backup::import_backup_archive(&db.pool, &runtime, &session_id, archive_path)
        .await
}

#[tauri::command]
pub async fn restore_from_backup(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    backup_name: String,
) -> CmdResult<RestoreResult> {
    let runtime = backup_runtime(&app)?;
    backup::restore_from_backup(&db.pool, &runtime, &session_id, backup_name)
        .await
}

#[tauri::command]
pub async fn restore_from_backup_folder(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    folder_path: String,
) -> CmdResult<RestoreResult> {
    let runtime = backup_runtime(&app)?;
    backup::restore_from_backup_folder(&db.pool, &runtime, &session_id, folder_path)
        .await
}

#[tauri::command]
pub async fn run_scheduled_backup_check(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Option<BackupSummary>> {
    let runtime = backup_runtime(&app)?;
    backup::run_scheduled_backup_check(&db.pool, &runtime, &session_id)
        .await
}

#[tauri::command]
pub async fn list_scanners(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<ScannerDevice>> {
    devices::list_scanners(&db.pool, &session_id)
        .await
}

#[tauri::command]
pub async fn list_printers(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<PrinterDevice>> {
    devices::list_printers(&db.pool, &session_id)
        .await
}

#[tauri::command]
pub async fn get_default_printer(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Option<PrinterDevice>> {
    devices::get_default_printer(&db.pool, &session_id)
        .await
}

#[tauri::command]
pub async fn get_device_settings(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<DeviceSettings> {
    devices::get_device_settings(&db.pool, &session_id)
        .await
}

#[tauri::command]
pub async fn update_device_settings(
    db: State<'_, DbState>,
    session_id: String,
    default_scanner_id: Option<String>,
    default_printer_id: Option<String>,
    scan_default_dpi: i64,
    scan_default_color_mode: String,
    scan_default_output_format: String,
) -> CmdResult<DeviceSettings> {
    devices::update_device_settings(
        &db.pool,
        &session_id,
        DeviceSettingsInput {
            default_scanner_id,
            default_printer_id,
            scan_default_dpi,
            scan_default_color_mode,
            scan_default_output_format,
        },
    )
    .await
}

#[tauri::command]
pub async fn list_print_printers(
    db: State<'_, DbState>,
    session_id: Option<String>,
) -> CmdResult<Vec<PrinterDevice>> {
    printing::list_print_printers(&db.pool, session_id.as_deref())
        .await
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
    printing::print_document_pdf(
        &db.pool,
        &storage,
        session_id.as_deref(),
        document_id,
        &printer_id,
        PrintOptions { copies },
    )
    .await
}

fn storage_root(app: &AppHandle) -> CmdResult<StorageRoot> {
    let root = app.path().app_data_dir().map_err(|e| AppError::Validation(e.to_string()))?.join("storage");
    StorageRoot::new(root)
}

fn backup_runtime(app: &AppHandle) -> CmdResult<BackupRuntime> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| AppError::Validation(e.to_string()))?;
    let db_path = app_data_dir.join("filing_system.db");
    let storage = storage_root(app)?;
    Ok(BackupRuntime::new(app_data_dir, db_path, storage))
}

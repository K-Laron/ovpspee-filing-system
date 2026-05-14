use tauri::{AppHandle, Manager, State};

use crate::{
    auth::{self, SessionPayload},
    db::DbState,
    documents::{
        self, AttachmentInput, DocumentDetail, DocumentInput, DocumentItem, DocumentListFilter,
        StorageRoot,
    },
    master_data::{
        self, CategoryInput, CategoryItem, FolderInput, FolderItem, OfficeInput, OfficeItem,
    },
    users::{self, ProfileInput, ProfileItem, UserInput, UserItem, UserUpdateInput},
};

#[tauri::command]
pub async fn first_run_check(db: State<'_, DbState>) -> Result<bool, String> {
    auth::first_run_required(&db.pool)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn first_run_setup(
    db: State<'_, DbState>,
    first_name: String,
    last_name: String,
    username: String,
    password: String,
) -> Result<(), String> {
    auth::create_first_admin(&db.pool, &first_name, &last_name, &username, &password)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn login(
    db: State<'_, DbState>,
    username: String,
    password: String,
) -> Result<SessionPayload, String> {
    auth::authenticate_user(&db.pool, &username, &password)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn logout(db: State<'_, DbState>, session_id: String) -> Result<(), String> {
    auth::logout_session(&db.pool, &session_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn validate_session(
    db: State<'_, DbState>,
    session_id: String,
) -> Result<SessionPayload, String> {
    auth::validate_session(&db.pool, &session_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn list_categories(
    db: State<'_, DbState>,
    session_id: String,
    include_inactive: Option<bool>,
) -> Result<Vec<CategoryItem>, String> {
    master_data::list_categories(&db.pool, &session_id, include_inactive)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn create_category(
    db: State<'_, DbState>,
    session_id: String,
    category_name: String,
    description: Option<String>,
    color_code: String,
    icon: Option<String>,
) -> Result<i64, String> {
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
    .map_err(|err| err.to_string())
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
) -> Result<(), String> {
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
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn list_folders(
    db: State<'_, DbState>,
    session_id: String,
    category_id: Option<i64>,
    include_inactive: Option<bool>,
) -> Result<Vec<FolderItem>, String> {
    master_data::list_folders(&db.pool, &session_id, category_id, include_inactive)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn create_folder(
    db: State<'_, DbState>,
    session_id: String,
    category_id: i64,
    folder_name: String,
    description: Option<String>,
    folder_color: String,
) -> Result<i64, String> {
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
    .map_err(|err| err.to_string())
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
) -> Result<(), String> {
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
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn list_offices(
    db: State<'_, DbState>,
    session_id: String,
    include_inactive: Option<bool>,
) -> Result<Vec<OfficeItem>, String> {
    master_data::list_offices(&db.pool, &session_id, include_inactive)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn create_office(
    db: State<'_, DbState>,
    session_id: String,
    office_name: String,
    description: Option<String>,
) -> Result<i64, String> {
    master_data::create_office(
        &db.pool,
        &session_id,
        OfficeInput {
            office_name,
            description,
        },
    )
    .await
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn update_office(
    db: State<'_, DbState>,
    session_id: String,
    office_id: i64,
    office_name: String,
    description: Option<String>,
    is_active: bool,
) -> Result<(), String> {
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
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn list_users(
    db: State<'_, DbState>,
    session_id: String,
    search: Option<String>,
) -> Result<Vec<UserItem>, String> {
    users::list_users(&db.pool, &session_id, search.as_deref())
        .await
        .map_err(|err| err.to_string())
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
) -> Result<i64, String> {
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
    .map_err(|err| err.to_string())
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
) -> Result<(), String> {
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
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn admin_reset_password(
    db: State<'_, DbState>,
    session_id: String,
    user_id: i64,
    new_password: String,
) -> Result<(), String> {
    users::admin_reset_password(&db.pool, &session_id, user_id, &new_password)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_my_profile(
    db: State<'_, DbState>,
    session_id: String,
) -> Result<ProfileItem, String> {
    users::get_my_profile(&db.pool, &session_id)
        .await
        .map_err(|err| err.to_string())
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
) -> Result<(), String> {
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
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn change_my_password(
    db: State<'_, DbState>,
    session_id: String,
    current_password: String,
    new_password: String,
) -> Result<(), String> {
    users::change_my_password(&db.pool, &session_id, &current_password, &new_password)
        .await
        .map_err(|err| err.to_string())
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
) -> Result<i64, String> {
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
    .map_err(|err| err.to_string())
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
) -> Result<(), String> {
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
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn move_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    category_id: i64,
    folder_id: Option<i64>,
) -> Result<(), String> {
    documents::move_document(&db.pool, &session_id, document_id, category_id, folder_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn set_document_status(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    status: String,
) -> Result<(), String> {
    documents::set_document_status(&db.pool, &session_id, document_id, status)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn set_document_hidden(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    is_hidden: bool,
) -> Result<(), String> {
    documents::set_document_hidden(&db.pool, &session_id, document_id, is_hidden)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn trash_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> Result<(), String> {
    documents::trash_document(&db.pool, &session_id, document_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn restore_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> Result<(), String> {
    documents::restore_document(&db.pool, &session_id, document_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn list_trash_documents(
    db: State<'_, DbState>,
    session_id: String,
) -> Result<Vec<DocumentItem>, String> {
    documents::list_trash_documents(&db.pool, &session_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn purge_document(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> Result<(), String> {
    let storage = storage_root(&app)?;
    documents::purge_document(&db.pool, &storage, &session_id, document_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn empty_trash(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
) -> Result<i64, String> {
    let storage = storage_root(&app)?;
    documents::empty_trash(&db.pool, &storage, &session_id)
        .await
        .map_err(|err| err.to_string())
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
) -> Result<Vec<DocumentItem>, String> {
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
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_document(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
) -> Result<DocumentDetail, String> {
    documents::get_document(&db.pool, &session_id, document_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn add_attachment(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    source_path: String,
    sort_order: Option<i64>,
) -> Result<i64, String> {
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
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn remove_attachment(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    attachment_id: i64,
) -> Result<(), String> {
    let storage = storage_root(&app)?;
    documents::remove_attachment(&db.pool, &storage, &session_id, attachment_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn reorder_attachments(
    db: State<'_, DbState>,
    session_id: String,
    document_id: i64,
    attachment_ids: Vec<i64>,
) -> Result<(), String> {
    documents::reorder_attachments(&db.pool, &session_id, document_id, attachment_ids)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_attachment_file_path(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: Option<String>,
    attachment_id: i64,
) -> Result<String, String> {
    let storage = storage_root(&app)?;
    documents::get_attachment_file_path(&db.pool, &storage, session_id.as_deref(), attachment_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn list_public_categories(db: State<'_, DbState>) -> Result<Vec<CategoryItem>, String> {
    documents::list_public_categories(&db.pool)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn list_public_folders(
    db: State<'_, DbState>,
    category_id: i64,
) -> Result<Vec<FolderItem>, String> {
    documents::list_public_folders(&db.pool, category_id)
        .await
        .map_err(|err| err.to_string())
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
) -> Result<Vec<DocumentItem>, String> {
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
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_public_document(
    db: State<'_, DbState>,
    document_id: i64,
) -> Result<DocumentDetail, String> {
    documents::get_public_document(&db.pool, document_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn list_document_offices(
    db: State<'_, DbState>,
    session_id: String,
) -> Result<Vec<OfficeItem>, String> {
    documents::list_document_offices(&db.pool, &session_id)
        .await
        .map_err(|err| err.to_string())
}

fn storage_root(app: &AppHandle) -> Result<StorageRoot, String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|err| err.to_string())?
        .join("storage");
    StorageRoot::new(root).map_err(|err| err.to_string())
}

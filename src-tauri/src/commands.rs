use tauri::State;

use crate::{
    auth::{self, SessionPayload},
    db::DbState,
    master_data::{
        self, CategoryInput, CategoryItem, FolderInput, FolderItem, OfficeInput, OfficeItem,
    },
    users::{
        self, ProfileInput, ProfileItem, UserInput, UserItem, UserUpdateInput,
    },
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

pub mod auth;
pub mod commands;
pub mod db;
pub mod error;
pub mod master_data;
pub mod users;

use db::{connect_database, DbState};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let db_path = app_data_dir.join("filing_system.db");
            let handle = tauri::async_runtime::block_on(async move { connect_database(&db_path).await })
                .map_err(|err| Box::<dyn std::error::Error>::from(err.to_string()))?;
            app.manage(DbState { pool: handle });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::first_run_check,
            commands::first_run_setup,
            commands::login,
            commands::logout,
            commands::validate_session,
            commands::list_categories,
            commands::create_category,
            commands::update_category,
            commands::list_folders,
            commands::create_folder,
            commands::update_folder,
            commands::list_offices,
            commands::create_office,
            commands::update_office,
            commands::list_users,
            commands::create_user,
            commands::update_user,
            commands::admin_reset_password,
            commands::get_my_profile,
            commands::update_my_profile,
            commands::change_my_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running OVPSPEE Filing System");
}

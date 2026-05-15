pub mod audit_log;
pub mod auth;
pub mod backup;
pub mod commands;
pub mod db;
pub mod devices;
pub mod documents;
pub mod error;
pub mod master_data;
pub mod printing;
pub mod scan_intake;
pub mod users;

use db::{connect_database, DbState};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let db_path = app_data_dir.join("filing_system.db");
            let handle =
                tauri::async_runtime::block_on(async move { connect_database(&db_path).await })
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
            commands::change_my_password,
            commands::create_document,
            commands::update_document,
            commands::move_document,
            commands::set_document_status,
            commands::set_document_hidden,
            commands::trash_document,
            commands::restore_document,
            commands::list_trash_documents,
            commands::purge_document,
            commands::empty_trash,
            commands::list_documents,
            commands::get_document,
            commands::add_attachment,
            commands::remove_attachment,
            commands::reorder_attachments,
            commands::get_attachment_file_path,
            commands::get_attachment_preview_info,
            commands::get_attachment_preview_page,
            commands::export_document_pdf,
            commands::list_public_categories,
            commands::list_public_folders,
            commands::list_public_documents,
            commands::get_public_document,
            commands::list_document_offices,
            commands::import_scan_files,
            commands::list_scan_intake,
            commands::get_scan_intake,
            commands::update_scan_intake_notes,
            commands::remove_scan_intake,
            commands::file_scan_as_document,
            commands::attach_scan_to_document,
            commands::get_scanner_capabilities,
            commands::scan_to_intake,
            commands::list_audit_logs,
            commands::list_my_activity,
            commands::list_audit_event_types,
            commands::list_my_activity_event_types,
            commands::get_audit_retention_settings,
            commands::update_audit_retention_settings,
            commands::get_backup_settings,
            commands::update_backup_settings,
            commands::create_backup,
            commands::list_backup_history,
            commands::export_backup_archive,
            commands::validate_backup_archive,
            commands::import_backup_archive,
            commands::restore_from_backup,
            commands::restore_from_backup_folder,
            commands::run_scheduled_backup_check,
            commands::list_scanners,
            commands::list_printers,
            commands::get_default_printer,
            commands::get_device_settings,
            commands::update_device_settings,
            commands::list_print_printers,
            commands::print_document_pdf
        ])
        .run(tauri::generate_context!())
        .expect("error while running OVPSPEE Filing System");
}

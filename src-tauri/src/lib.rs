pub mod audit_log;
pub mod auth;
pub mod backup;
pub mod commands;
pub mod db;
pub mod devices;
pub mod documents;
pub mod error;
pub mod master_data;
pub mod mobile_api;
pub mod mobile_devices;
pub mod mobile_submissions;
pub mod pdf_export;
pub mod preview;
pub mod printing;
pub mod scan_intake;
pub mod users;
pub mod util;

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
            if std::env::var("OVPSPEE_MOBILE_API_ENABLED").as_deref() == Ok("1") {
                let api_pool = handle.clone();
                let storage = documents::StorageRoot::new(app_data_dir.join("storage"))
                    .map_err(|err| Box::<dyn std::error::Error>::from(err.to_string()))?;
                let addr = std::env::var("OVPSPEE_MOBILE_API_ADDR")
                    .unwrap_or_else(|_| "0.0.0.0:1421".to_owned());
                tauri::async_runtime::spawn(async move {
                    if mobile_api::serve(api_pool, storage, &addr).await.is_err() {
                        eprintln!("Mobile API stopped.");
                    }
                });
            }
            app.manage(DbState { pool: handle });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth_cmds::first_run_check,
            commands::auth_cmds::first_run_setup,
            commands::auth_cmds::login,
            commands::auth_cmds::logout,
            commands::auth_cmds::validate_session,
            commands::admin_cmds::list_categories,
            commands::admin_cmds::create_category,
            commands::admin_cmds::update_category,
            commands::admin_cmds::list_folders,
            commands::admin_cmds::create_folder,
            commands::admin_cmds::update_folder,
            commands::admin_cmds::list_offices,
            commands::admin_cmds::create_office,
            commands::admin_cmds::update_office,
            commands::admin_cmds::list_users,
            commands::admin_cmds::create_user,
            commands::admin_cmds::update_user,
            commands::admin_cmds::admin_reset_password,
            commands::admin_cmds::get_my_profile,
            commands::admin_cmds::update_my_profile,
            commands::admin_cmds::change_my_password,
            commands::document_cmds::create_document,
            commands::document_cmds::update_document,
            commands::document_cmds::move_document,
            commands::document_cmds::set_document_status,
            commands::document_cmds::set_document_hidden,
            commands::document_cmds::trash_document,
            commands::document_cmds::restore_document,
            commands::document_cmds::list_trash_documents,
            commands::document_cmds::purge_document,
            commands::document_cmds::empty_trash,
            commands::document_cmds::list_documents,
            commands::document_cmds::get_document,
            commands::document_cmds::add_attachment,
            commands::document_cmds::remove_attachment,
            commands::document_cmds::reorder_attachments,
            commands::document_cmds::get_attachment_file_path,
            commands::document_cmds::get_attachment_preview_info,
            commands::document_cmds::get_attachment_preview_page,
            commands::document_cmds::export_document_pdf,
            commands::public_cmds::list_public_categories,
            commands::public_cmds::list_public_folders,
            commands::public_cmds::list_public_documents,
            commands::public_cmds::get_public_document,
            commands::public_cmds::list_document_offices,
            commands::public_cmds::list_print_printers,
            commands::public_cmds::print_document_pdf,
            commands::mobile_cmds::list_mobile_submissions,
            commands::mobile_cmds::get_mobile_api_setup,
            commands::mobile_cmds::create_mobile_device,
            commands::mobile_cmds::list_mobile_devices,
            commands::mobile_cmds::revoke_mobile_device,
            commands::mobile_cmds::get_mobile_submission,
            commands::mobile_cmds::get_mobile_submission_attachment_preview_page,
            commands::mobile_cmds::approve_mobile_submission,
            commands::mobile_cmds::reject_mobile_submission,
            commands::scan_cmds::import_scan_files,
            commands::scan_cmds::list_scan_intake,
            commands::scan_cmds::get_scan_intake,
            commands::scan_cmds::get_scan_intake_preview_page,
            commands::scan_cmds::update_scan_intake_notes,
            commands::scan_cmds::remove_scan_intake,
            commands::scan_cmds::file_scan_as_document,
            commands::scan_cmds::attach_scan_to_document,
            commands::device_cmds::get_scanner_capabilities,
            commands::device_cmds::scan_to_intake,
            commands::device_cmds::list_scanners,
            commands::device_cmds::list_printers,
            commands::device_cmds::get_default_printer,
            commands::device_cmds::get_device_settings,
            commands::device_cmds::update_device_settings,
            commands::admin_cmds::list_audit_logs,
            commands::admin_cmds::list_my_activity,
            commands::admin_cmds::list_audit_event_types,
            commands::admin_cmds::list_my_activity_event_types,
            commands::admin_cmds::get_audit_retention_settings,
            commands::admin_cmds::update_audit_retention_settings,
            commands::admin_cmds::get_backup_settings,
            commands::admin_cmds::update_backup_settings,
            commands::admin_cmds::create_backup,
            commands::admin_cmds::list_backup_history,
            commands::admin_cmds::export_backup_archive,
            commands::admin_cmds::validate_backup_archive,
            commands::admin_cmds::import_backup_archive,
            commands::admin_cmds::restore_from_backup,
            commands::admin_cmds::restore_from_backup_folder,
            commands::admin_cmds::run_scheduled_backup_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OVPSPEE Filing System");
}

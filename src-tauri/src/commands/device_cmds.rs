use tauri::{AppHandle, State};

use crate::db::DbState;
use crate::devices::{self, DeviceSettings, DeviceSettingsInput, PrinterDevice, ScanOptions, ScannerCapabilities, ScannerDevice};

use super::{storage_root, CmdResult};

#[tauri::command]
pub async fn get_scanner_capabilities(
    db: State<'_, DbState>,
    session_id: String,
    scanner_id: String,
) -> CmdResult<ScannerCapabilities> {
    devices::get_scanner_capabilities(&db.pool, &session_id, &scanner_id).await
}

#[tauri::command]
pub async fn scan_to_intake(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    scanner_id: String,
    options: ScanOptions,
) -> CmdResult<crate::scan_intake::ScanIntakeItem> {
    let storage = storage_root(&app)?;
    devices::scan_to_intake(&db.pool, &storage, &session_id, &scanner_id, options).await
}

#[tauri::command]
pub async fn list_scanners(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<ScannerDevice>> {
    devices::list_scanners(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn list_printers(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Vec<PrinterDevice>> {
    devices::list_printers(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn get_default_printer(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<Option<PrinterDevice>> {
    devices::get_default_printer(&db.pool, &session_id).await
}

#[tauri::command]
pub async fn get_device_settings(
    db: State<'_, DbState>,
    session_id: String,
) -> CmdResult<DeviceSettings> {
    devices::get_device_settings(&db.pool, &session_id).await
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
        &db.pool, &session_id,
        DeviceSettingsInput { default_scanner_id, default_printer_id, scan_default_dpi, scan_default_color_mode, scan_default_output_format },
    ).await
}

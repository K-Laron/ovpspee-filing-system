use std::future::Future;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{require_admin_role, require_session, write_audit_log},
    db::DbPool,
    error::{AppError, AppResult},
};

#[cfg(target_os = "windows")]
mod windows;

pub mod mock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScannerDevice {
    pub device_id: String,
    pub name: String,
    pub manufacturer: Option<String>,
    pub connection_type: Option<String>,
    pub is_available: bool,
    pub status: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrinterDevice {
    pub printer_id: String,
    pub name: String,
    pub is_default: bool,
    pub status: String,
    pub is_available: bool,
    pub is_network: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceSettings {
    pub default_scanner_id: Option<String>,
    pub default_printer_id: Option<String>,
    pub scan_default_dpi: i64,
    pub scan_default_color_mode: String,
    pub scan_default_output_format: String,
    pub device_detection_last_checked_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceSettingsInput {
    pub default_scanner_id: Option<String>,
    pub default_printer_id: Option<String>,
    pub scan_default_dpi: i64,
    pub scan_default_color_mode: String,
    pub scan_default_output_format: String,
}

pub trait DeviceProvider {
    fn list_scanners(&self) -> impl Future<Output = AppResult<Vec<ScannerDevice>>> + Send;
    fn list_printers(&self) -> impl Future<Output = AppResult<Vec<PrinterDevice>>> + Send;
}

#[derive(Clone, Copy)]
pub struct SystemDeviceProvider;

impl DeviceProvider for SystemDeviceProvider {
    async fn list_scanners(&self) -> AppResult<Vec<ScannerDevice>> {
        list_system_scanners().await
    }

    async fn list_printers(&self) -> AppResult<Vec<PrinterDevice>> {
        list_system_printers().await
    }
}

pub async fn list_scanners(pool: &DbPool, session_id: &str) -> AppResult<Vec<ScannerDevice>> {
    list_scanners_with_provider(pool, session_id, &SystemDeviceProvider).await
}

pub async fn list_printers(pool: &DbPool, session_id: &str) -> AppResult<Vec<PrinterDevice>> {
    list_printers_with_provider(pool, session_id, &SystemDeviceProvider).await
}

pub async fn get_default_printer(pool: &DbPool, session_id: &str) -> AppResult<Option<PrinterDevice>> {
    Ok(list_printers(pool, session_id)
        .await?
        .into_iter()
        .find(|printer| printer.is_default))
}

pub async fn get_device_settings(pool: &DbPool, session_id: &str) -> AppResult<DeviceSettings> {
    let session = require_session(pool, session_id).await?;
    require_device_reader(&session.role)?;
    read_settings(pool).await
}

pub async fn update_device_settings(
    pool: &DbPool,
    session_id: &str,
    input: DeviceSettingsInput,
) -> AppResult<DeviceSettings> {
    update_device_settings_with_provider(pool, session_id, input, &SystemDeviceProvider).await
}

pub async fn list_scanners_with_provider(
    pool: &DbPool,
    session_id: &str,
    provider: &impl DeviceProvider,
) -> AppResult<Vec<ScannerDevice>> {
    let session = require_session(pool, session_id).await?;
    require_device_reader(&session.role)?;
    let devices = provider.list_scanners().await?;
    upsert_setting(pool, "device_detection_last_checked_at", &Utc::now().to_rfc3339()).await?;
    Ok(sanitize_scanners(devices))
}

pub async fn list_printers_with_provider(
    pool: &DbPool,
    session_id: &str,
    provider: &impl DeviceProvider,
) -> AppResult<Vec<PrinterDevice>> {
    let session = require_session(pool, session_id).await?;
    require_device_reader(&session.role)?;
    let devices = provider.list_printers().await?;
    upsert_setting(pool, "device_detection_last_checked_at", &Utc::now().to_rfc3339()).await?;
    Ok(sanitize_printers(devices))
}

pub async fn update_device_settings_with_provider(
    pool: &DbPool,
    session_id: &str,
    input: DeviceSettingsInput,
    provider: &impl DeviceProvider,
) -> AppResult<DeviceSettings> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    let input = validate_settings(input)?;
    let scanners = sanitize_scanners(provider.list_scanners().await?);
    let printers = sanitize_printers(provider.list_printers().await?);
    if let Some(scanner_id) = &input.default_scanner_id {
        if !scanners.iter().any(|device| &device.device_id == scanner_id) {
            return Err(AppError::Validation("Selected scanner is not available.".into()));
        }
    }
    if let Some(printer_id) = &input.default_printer_id {
        if !printers.iter().any(|device| &device.printer_id == printer_id) {
            return Err(AppError::Validation("Selected printer is not available.".into()));
        }
    }

    upsert_setting_optional(pool, "default_scanner_id", input.default_scanner_id.as_deref()).await?;
    upsert_setting_optional(pool, "default_printer_id", input.default_printer_id.as_deref()).await?;
    upsert_setting(pool, "scan_default_dpi", &input.scan_default_dpi.to_string()).await?;
    upsert_setting(pool, "scan_default_color_mode", &input.scan_default_color_mode).await?;
    upsert_setting(pool, "scan_default_output_format", &input.scan_default_output_format).await?;
    upsert_setting(pool, "device_detection_last_checked_at", &Utc::now().to_rfc3339()).await?;
    write_audit_log(
        pool,
        "UPDATE",
        Some("settings"),
        None,
        "Updated device detection defaults",
        Some(session.user_id),
    )
    .await?;
    read_settings(pool).await
}

fn require_device_reader(role: &str) -> AppResult<()> {
    if role == "Admin" || role == "Secretary" {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}

fn validate_settings(input: DeviceSettingsInput) -> AppResult<DeviceSettingsInput> {
    if ![200, 300, 600].contains(&input.scan_default_dpi) {
        return Err(AppError::Validation("Scan DPI must be 200, 300, or 600.".into()));
    }
    if !["color", "grayscale", "black_white"].contains(&input.scan_default_color_mode.as_str()) {
        return Err(AppError::Validation("Scan color mode is not supported.".into()));
    }
    if !["pdf", "png", "jpg"].contains(&input.scan_default_output_format.as_str()) {
        return Err(AppError::Validation("Scan output format is not supported.".into()));
    }
    if input
        .default_scanner_id
        .as_deref()
        .is_some_and(unsafe_device_id)
        || input
            .default_printer_id
            .as_deref()
            .is_some_and(unsafe_device_id)
    {
        return Err(AppError::Validation("Device ID is not valid.".into()));
    }
    Ok(input)
}

fn sanitize_scanners(devices: Vec<ScannerDevice>) -> Vec<ScannerDevice> {
    devices
        .into_iter()
        .filter(|device| !unsafe_device_id(&device.device_id) && !unsafe_device_id(&device.name))
        .collect()
}

fn sanitize_printers(devices: Vec<PrinterDevice>) -> Vec<PrinterDevice> {
    devices
        .into_iter()
        .filter(|device| !unsafe_device_id(&device.printer_id) && !unsafe_device_id(&device.name))
        .collect()
}

fn unsafe_device_id(value: &str) -> bool {
    value.contains("..")
        || value.contains('\\')
        || value.contains('/')
        || value.contains(':')
        || value.contains('\0')
}

async fn read_settings(pool: &DbPool) -> AppResult<DeviceSettings> {
    let default_scanner_id = get_optional_setting(pool, "default_scanner_id").await?;
    let default_printer_id = get_optional_setting(pool, "default_printer_id").await?;
    let dpi = get_setting(pool, "scan_default_dpi", "300")
        .await?
        .parse::<i64>()
        .unwrap_or(300);
    let color_mode = get_setting(pool, "scan_default_color_mode", "color").await?;
    let output_format = get_setting(pool, "scan_default_output_format", "pdf").await?;
    let last_checked = get_optional_setting(pool, "device_detection_last_checked_at").await?;
    Ok(DeviceSettings {
        default_scanner_id,
        default_printer_id,
        scan_default_dpi: if [200, 300, 600].contains(&dpi) { dpi } else { 300 },
        scan_default_color_mode: if ["color", "grayscale", "black_white"].contains(&color_mode.as_str()) {
            color_mode
        } else {
            "color".to_owned()
        },
        scan_default_output_format: if ["pdf", "png", "jpg"].contains(&output_format.as_str()) {
            output_format
        } else {
            "pdf".to_owned()
        },
        device_detection_last_checked_at: last_checked,
    })
}

async fn get_setting(pool: &DbPool, key: &str, default: &str) -> AppResult<String> {
    Ok(get_optional_setting(pool, key)
        .await?
        .unwrap_or_else(|| default.to_owned()))
}

async fn get_optional_setting(pool: &DbPool, key: &str) -> AppResult<Option<String>> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|row| sqlx::Row::get::<String, _>(&row, "value")))
}

async fn upsert_setting_optional(pool: &DbPool, key: &str, value: Option<&str>) -> AppResult<()> {
    upsert_setting(pool, key, value.unwrap_or_default()).await
}

async fn upsert_setting(pool: &DbPool, key: &str, value: &str) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at)
         VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(value)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(target_os = "windows")]
async fn list_system_scanners() -> AppResult<Vec<ScannerDevice>> {
    windows::list_scanners().await
}

#[cfg(not(target_os = "windows"))]
async fn list_system_scanners() -> AppResult<Vec<ScannerDevice>> {
    Ok(Vec::new())
}

#[cfg(target_os = "windows")]
async fn list_system_printers() -> AppResult<Vec<PrinterDevice>> {
    windows::list_printers().await
}

#[cfg(not(target_os = "windows"))]
async fn list_system_printers() -> AppResult<Vec<PrinterDevice>> {
    Ok(Vec::new())
}

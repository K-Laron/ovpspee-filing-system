use std::{future::Future, path::Path};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{require_admin_role, require_session, write_audit_log},
    db::DbPool,
    documents::{mime_for_extension, now_text, validate_magic, StorageRoot, MAX_ATTACHMENT_BYTES},
    error::{AppError, AppResult},
    scan_intake::{self, ScanIntakeItem},
};

#[cfg(target_os = "windows")]
pub mod windows;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScannerCapabilities {
    pub scanner_id: String,
    pub is_available: bool,
    pub status: String,
    pub supports_flatbed: bool,
    pub supports_adf: bool,
    pub supported_dpi: Vec<i64>,
    pub supported_color_modes: Vec<String>,
    pub supported_output_formats: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanOptions {
    pub dpi: i64,
    pub color_mode: String,
    pub output_format: String,
    pub source: String,
}

pub trait DeviceProvider {
    fn list_scanners(&self) -> impl Future<Output = AppResult<Vec<ScannerDevice>>> + Send;
    fn list_printers(&self) -> impl Future<Output = AppResult<Vec<PrinterDevice>>> + Send;
    fn scanner_capabilities(
        &self,
        scanner_id: &str,
    ) -> impl Future<Output = AppResult<ScannerCapabilities>> + Send;
    fn scan_to_path(
        &self,
        scanner_id: &str,
        options: &ScanOptions,
        destination: &Path,
    ) -> impl Future<Output = AppResult<()>> + Send;
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

    async fn scanner_capabilities(&self, scanner_id: &str) -> AppResult<ScannerCapabilities> {
        system_scanner_capabilities(scanner_id).await
    }

    async fn scan_to_path(
        &self,
        scanner_id: &str,
        options: &ScanOptions,
        destination: &Path,
    ) -> AppResult<()> {
        system_scan_to_path(scanner_id, options, destination).await
    }
}

pub async fn list_scanners(pool: &DbPool, session_id: &str) -> AppResult<Vec<ScannerDevice>> {
    list_scanners_with_provider(pool, session_id, &SystemDeviceProvider).await
}

pub async fn list_printers(pool: &DbPool, session_id: &str) -> AppResult<Vec<PrinterDevice>> {
    list_printers_with_provider(pool, session_id, &SystemDeviceProvider).await
}

pub async fn get_default_printer(
    pool: &DbPool,
    session_id: &str,
) -> AppResult<Option<PrinterDevice>> {
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

pub async fn get_scanner_capabilities(
    pool: &DbPool,
    session_id: &str,
    scanner_id: &str,
) -> AppResult<ScannerCapabilities> {
    get_scanner_capabilities_with_provider(pool, session_id, scanner_id, &SystemDeviceProvider)
        .await
}

pub async fn scan_to_intake(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    scanner_id: &str,
    options: ScanOptions,
) -> AppResult<ScanIntakeItem> {
    scan_to_intake_with_provider(
        pool,
        storage,
        session_id,
        scanner_id,
        options,
        &SystemDeviceProvider,
    )
    .await
}

pub async fn list_scanners_with_provider(
    pool: &DbPool,
    session_id: &str,
    provider: &impl DeviceProvider,
) -> AppResult<Vec<ScannerDevice>> {
    let session = require_session(pool, session_id).await?;
    require_device_reader(&session.role)?;
    let devices = provider.list_scanners().await?;
    upsert_setting(
        pool,
        "device_detection_last_checked_at",
        &Utc::now().to_rfc3339(),
    )
    .await?;
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
    upsert_setting(
        pool,
        "device_detection_last_checked_at",
        &Utc::now().to_rfc3339(),
    )
    .await?;
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
        if !scanners
            .iter()
            .any(|device| &device.device_id == scanner_id)
        {
            return Err(AppError::Validation(
                "Selected scanner is not available.".into(),
            ));
        }
    }
    if let Some(printer_id) = &input.default_printer_id {
        if !printers
            .iter()
            .any(|device| &device.printer_id == printer_id)
        {
            return Err(AppError::Validation(
                "Selected printer is not available.".into(),
            ));
        }
    }

    upsert_setting_optional(
        pool,
        "default_scanner_id",
        input.default_scanner_id.as_deref(),
    )
    .await?;
    upsert_setting_optional(
        pool,
        "default_printer_id",
        input.default_printer_id.as_deref(),
    )
    .await?;
    upsert_setting(
        pool,
        "scan_default_dpi",
        &input.scan_default_dpi.to_string(),
    )
    .await?;
    upsert_setting(
        pool,
        "scan_default_color_mode",
        &input.scan_default_color_mode,
    )
    .await?;
    upsert_setting(
        pool,
        "scan_default_output_format",
        &input.scan_default_output_format,
    )
    .await?;
    upsert_setting(
        pool,
        "device_detection_last_checked_at",
        &Utc::now().to_rfc3339(),
    )
    .await?;
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

pub async fn get_scanner_capabilities_with_provider(
    pool: &DbPool,
    session_id: &str,
    scanner_id: &str,
    provider: &impl DeviceProvider,
) -> AppResult<ScannerCapabilities> {
    let session = require_session(pool, session_id).await?;
    require_device_reader(&session.role)?;
    validate_device_id(scanner_id)?;
    let scanners = sanitize_scanners(provider.list_scanners().await?);
    let scanner = scanners
        .iter()
        .find(|device| device.device_id == scanner_id)
        .ok_or_else(|| AppError::Validation("Selected scanner is not available.".into()))?;
    if !scanner.is_available {
        return Err(AppError::Validation(
            "Selected scanner is not available.".into(),
        ));
    }
    let capabilities = provider.scanner_capabilities(scanner_id).await?;
    Ok(sanitize_capabilities(capabilities))
}

pub async fn scan_to_intake_with_provider(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    scanner_id: &str,
    options: ScanOptions,
    provider: &impl DeviceProvider,
) -> AppResult<ScanIntakeItem> {
    let session = require_session(pool, session_id).await?;
    if session.role != "Secretary" {
        return Err(AppError::Unauthorized);
    }
    validate_device_id(scanner_id)?;
    let options = validate_scan_options(options)?;
    let capabilities =
        get_scanner_capabilities_with_provider(pool, session_id, scanner_id, provider).await?;
    if !capabilities
        .supported_output_formats
        .contains(&options.output_format)
    {
        return Err(AppError::Validation(
            "Scan output format is not supported by this scanner.".into(),
        ));
    }
    if !capabilities.supported_dpi.contains(&options.dpi) {
        return Err(AppError::Validation(
            "Scan DPI is not supported by this scanner.".into(),
        ));
    }
    if !capabilities
        .supported_color_modes
        .contains(&options.color_mode)
    {
        return Err(AppError::Validation(
            "Scan color mode is not supported by this scanner.".into(),
        ));
    }
    if options.source == "adf" && !capabilities.supports_adf {
        return Err(AppError::Validation(
            "ADF scan source is not supported by this scanner.".into(),
        ));
    }
    if options.source == "flatbed" && !capabilities.supports_flatbed {
        return Err(AppError::Validation(
            "Flatbed scan source is not supported by this scanner.".into(),
        ));
    }

    let ext = options.output_format.clone();
    let relative = format!("intake/scanner-capture-{}.{}", uuid::Uuid::new_v4(), ext);
    let destination = storage.resolve_checked(&relative)?;
    if let Err(err) = provider
        .scan_to_path(scanner_id, &options, &destination)
        .await
    {
        let _ = std::fs::remove_file(&destination);
        return Err(err);
    }
    let size = std::fs::metadata(&destination)?.len();
    if size > MAX_ATTACHMENT_BYTES {
        let _ = std::fs::remove_file(&destination);
        return Err(AppError::Validation(
            "Scan file exceeds 1 GB maximum.".into(),
        ));
    }
    if let Err(err) = validate_magic(&destination, &ext) {
        let _ = std::fs::remove_file(&destination);
        return Err(err);
    }
    let mime_type = mime_for_extension(&ext).to_owned();
    let original_file_name = format!(
        "scanner-capture-{}.{}",
        now_text().replace([':', '-'], ""),
        ext
    );
    scan_intake::create_scan_intake_from_stored_file(
        pool,
        session.user_id,
        original_file_name,
        relative,
        mime_type,
        size as i64,
        "Captured scan into intake",
    )
    .await
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
        return Err(AppError::Validation(
            "Scan DPI must be 200, 300, or 600.".into(),
        ));
    }
    if !["color", "grayscale", "black_white"].contains(&input.scan_default_color_mode.as_str()) {
        return Err(AppError::Validation(
            "Scan color mode is not supported.".into(),
        ));
    }
    if !["png", "jpg"].contains(&input.scan_default_output_format.as_str()) {
        return Err(AppError::Validation(
            "Scan output format is not supported.".into(),
        ));
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

fn validate_scan_options(input: ScanOptions) -> AppResult<ScanOptions> {
    if ![200, 300, 600].contains(&input.dpi) {
        return Err(AppError::Validation(
            "Scan DPI must be 200, 300, or 600.".into(),
        ));
    }
    if !["color", "grayscale", "black_white"].contains(&input.color_mode.as_str()) {
        return Err(AppError::Validation(
            "Scan color mode is not supported.".into(),
        ));
    }
    if !["png", "jpg"].contains(&input.output_format.as_str()) {
        return Err(AppError::Validation(
            "Scanner capture currently supports PNG or JPG output.".into(),
        ));
    }
    if !["flatbed", "adf"].contains(&input.source.as_str()) {
        return Err(AppError::Validation("Scan source is not supported.".into()));
    }
    Ok(input)
}

fn validate_device_id(value: &str) -> AppResult<()> {
    if value.trim().is_empty() || unsafe_device_id(value) {
        return Err(AppError::Validation("Device ID is not valid.".into()));
    }
    Ok(())
}

fn sanitize_capabilities(mut capabilities: ScannerCapabilities) -> ScannerCapabilities {
    if unsafe_device_id(&capabilities.scanner_id) {
        capabilities.scanner_id.clear();
        capabilities.is_available = false;
        capabilities.status = "Invalid scanner identifier.".to_owned();
    }
    capabilities
        .supported_dpi
        .retain(|dpi| [200, 300, 600].contains(dpi));
    capabilities
        .supported_color_modes
        .retain(|mode| ["color", "grayscale", "black_white"].contains(&mode.as_str()));
    capabilities
        .supported_output_formats
        .retain(|format| ["png", "jpg"].contains(&format.as_str()));
    capabilities
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
    let output_format = get_setting(pool, "scan_default_output_format", "png").await?;
    let last_checked = get_optional_setting(pool, "device_detection_last_checked_at").await?;
    Ok(DeviceSettings {
        default_scanner_id,
        default_printer_id,
        scan_default_dpi: if [200, 300, 600].contains(&dpi) {
            dpi
        } else {
            300
        },
        scan_default_color_mode: if ["color", "grayscale", "black_white"]
            .contains(&color_mode.as_str())
        {
            color_mode
        } else {
            "color".to_owned()
        },
        scan_default_output_format: if ["png", "jpg"].contains(&output_format.as_str()) {
            output_format
        } else {
            "png".to_owned()
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

#[cfg(target_os = "windows")]
async fn system_scanner_capabilities(scanner_id: &str) -> AppResult<ScannerCapabilities> {
    windows::scanner_capabilities(scanner_id).await
}

#[cfg(not(target_os = "windows"))]
async fn system_scanner_capabilities(scanner_id: &str) -> AppResult<ScannerCapabilities> {
    Ok(default_capabilities(
        scanner_id,
        false,
        "Scanner capture is only available on Windows.",
    ))
}

#[cfg(target_os = "windows")]
async fn system_scan_to_path(
    scanner_id: &str,
    options: &ScanOptions,
    destination: &Path,
) -> AppResult<()> {
    windows::scan_to_path(scanner_id, options, destination).await
}

#[cfg(not(target_os = "windows"))]
async fn system_scan_to_path(
    _scanner_id: &str,
    _options: &ScanOptions,
    _destination: &Path,
) -> AppResult<()> {
    Err(AppError::Validation(
        "Scanner capture is only available on Windows.".into(),
    ))
}

pub fn default_capabilities(
    scanner_id: &str,
    is_available: bool,
    status: &str,
) -> ScannerCapabilities {
    ScannerCapabilities {
        scanner_id: scanner_id.to_owned(),
        is_available,
        status: status.to_owned(),
        supports_flatbed: true,
        supports_adf: false,
        supported_dpi: vec![200, 300, 600],
        supported_color_modes: vec![
            "color".to_owned(),
            "grayscale".to_owned(),
            "black_white".to_owned(),
        ],
        supported_output_formats: vec!["png".to_owned(), "jpg".to_owned()],
    }
}

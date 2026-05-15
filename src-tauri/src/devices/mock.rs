use std::{future::Future, path::Path};

use crate::{
    devices::{
        default_capabilities, DeviceProvider, PrinterDevice, ScanOptions, ScannerCapabilities,
        ScannerDevice,
    },
    error::{AppError, AppResult},
};

#[derive(Clone, Default)]
pub struct MockDeviceProvider {
    pub scanners: Vec<ScannerDevice>,
    pub printers: Vec<PrinterDevice>,
    pub scan_fails: bool,
    pub captured_bytes: Vec<u8>,
}

impl DeviceProvider for MockDeviceProvider {
    fn list_scanners(&self) -> impl Future<Output = AppResult<Vec<ScannerDevice>>> + Send {
        let scanners = self.scanners.clone();
        async move { Ok(scanners) }
    }

    fn list_printers(&self) -> impl Future<Output = AppResult<Vec<PrinterDevice>>> + Send {
        let printers = self.printers.clone();
        async move { Ok(printers) }
    }

    fn scanner_capabilities(
        &self,
        scanner_id: &str,
    ) -> impl Future<Output = AppResult<ScannerCapabilities>> + Send {
        let scanner_id = scanner_id.to_owned();
        let available = self
            .scanners
            .iter()
            .any(|scanner| scanner.device_id == scanner_id && scanner.is_available);
        async move {
            if available {
                Ok(default_capabilities(&scanner_id, true, "Ready"))
            } else {
                Err(AppError::Validation(
                    "Selected scanner is not available.".into(),
                ))
            }
        }
    }

    fn scan_to_path(
        &self,
        scanner_id: &str,
        options: &ScanOptions,
        destination: &Path,
    ) -> impl Future<Output = AppResult<()>> + Send {
        let scanner_id = scanner_id.to_owned();
        let output_format = options.output_format.clone();
        let destination = destination.to_path_buf();
        let available = self
            .scanners
            .iter()
            .any(|scanner| scanner.device_id == scanner_id && scanner.is_available);
        let scan_fails = self.scan_fails;
        let captured_bytes = self.captured_bytes.clone();
        async move {
            if scan_fails {
                return Err(AppError::Validation("Scanner capture failed.".into()));
            }
            if !available {
                return Err(AppError::Validation(
                    "Selected scanner is not available.".into(),
                ));
            }
            let bytes = if captured_bytes.is_empty() {
                match output_format.as_str() {
                    "jpg" => vec![0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46],
                    _ => vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'],
                }
            } else {
                captured_bytes
            };
            std::fs::write(destination, bytes)?;
            Ok(())
        }
    }
}

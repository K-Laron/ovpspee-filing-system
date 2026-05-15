use std::future::Future;

use crate::{
    devices::{DeviceProvider, PrinterDevice, ScannerDevice},
    error::AppResult,
};

#[derive(Clone, Default)]
pub struct MockDeviceProvider {
    pub scanners: Vec<ScannerDevice>,
    pub printers: Vec<PrinterDevice>,
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
}

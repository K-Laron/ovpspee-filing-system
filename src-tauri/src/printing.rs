use std::{future::Future, path::Path};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{require_session, write_audit_log},
    db::DbPool,
    devices::{PrinterDevice, SystemDeviceProvider},
    documents::{self, StorageRoot},
    error::{AppError, AppResult},
    util::unsafe_device_id,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrintOptions {
    pub copies: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PrintResult {
    pub document_id: i64,
    pub printer_name: String,
    pub copies: i64,
    pub status: String,
}

pub trait PrintProvider {
    fn list_printers(&self) -> impl Future<Output = AppResult<Vec<PrinterDevice>>> + Send;
    fn print_pdf(
        &self,
        pdf_path: &Path,
        printer: &PrinterDevice,
        options: &PrintOptions,
    ) -> impl Future<Output = AppResult<()>> + Send;
}

#[derive(Clone, Copy)]
pub struct SystemPrintProvider;

impl PrintProvider for SystemPrintProvider {
    async fn list_printers(&self) -> AppResult<Vec<PrinterDevice>> {
        crate::devices::DeviceProvider::list_printers(&SystemDeviceProvider).await
    }

    async fn print_pdf(
        &self,
        pdf_path: &Path,
        printer: &PrinterDevice,
        options: &PrintOptions,
    ) -> AppResult<()> {
        system_print_pdf(pdf_path, printer, options).await
    }
}

pub async fn list_print_printers(
    pool: &DbPool,
    session_id: Option<&str>,
) -> AppResult<Vec<PrinterDevice>> {
    list_print_printers_with_provider(pool, session_id, &SystemPrintProvider).await
}

pub async fn print_document_pdf(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: Option<&str>,
    document_id: i64,
    printer_id: &str,
    options: PrintOptions,
) -> AppResult<PrintResult> {
    print_document_pdf_with_provider(
        pool,
        storage,
        session_id,
        document_id,
        printer_id,
        options,
        &SystemPrintProvider,
    )
    .await
}

pub async fn list_print_printers_with_provider(
    pool: &DbPool,
    session_id: Option<&str>,
    provider: &impl PrintProvider,
) -> AppResult<Vec<PrinterDevice>> {
    if let Some(session_id) = session_id {
        let session = require_session(pool, session_id).await?;
        if session.role != "Secretary" {
            return Err(AppError::Unauthorized);
        }
    }
    Ok(sanitize_printers(provider.list_printers().await?))
}

pub async fn print_document_pdf_with_provider(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: Option<&str>,
    document_id: i64,
    printer_id: &str,
    options: PrintOptions,
    provider: &impl PrintProvider,
) -> AppResult<PrintResult> {
    let actor_user_id = if let Some(session_id) = session_id {
        let session = require_session(pool, session_id).await?;
        if session.role != "Secretary" {
            return Err(AppError::Unauthorized);
        }
        Some(session.user_id)
    } else {
        None
    };
    validate_printer_id(printer_id)?;
    let options = validate_print_options(options)?;
    let printers = sanitize_printers(provider.list_printers().await?);
    let printer = printers
        .into_iter()
        .find(|printer| printer.printer_id == printer_id)
        .ok_or_else(|| AppError::Validation("Selected printer is not available.".into()))?;
    if !printer.is_available {
        return Err(AppError::Validation(
            "Selected printer is not available.".into(),
        ));
    }

    let relative = format!("print-tmp/document-{document_id}-{}.pdf", Uuid::new_v4());
    let output = storage.resolve_checked(&relative)?;
    let output_string = output.to_string_lossy().into_owned();
    if let Err(err) =
        documents::export_document_pdf(pool, storage, session_id, document_id, &output_string).await
    {
        let _ = std::fs::remove_file(&output);
        return Err(err);
    }
    let print_result = provider.print_pdf(&output, &printer, &options).await;
    let _ = std::fs::remove_file(&output);
    print_result?;
    write_audit_log(
        pool,
        "EXPORT",
        Some("document"),
        Some(document_id),
        &format!("Printed document PDF to {}", printer.name),
        actor_user_id,
    )
    .await?;
    Ok(PrintResult {
        document_id,
        printer_name: printer.name,
        copies: options.copies,
        status: "Submitted to printer".to_owned(),
    })
}

fn validate_print_options(input: PrintOptions) -> AppResult<PrintOptions> {
    if !(1..=20).contains(&input.copies) {
        return Err(AppError::Validation(
            "Copies must be between 1 and 20.".into(),
        ));
    }
    Ok(input)
}

fn sanitize_printers(devices: Vec<PrinterDevice>) -> Vec<PrinterDevice> {
    devices
        .into_iter()
        .filter(|device| !unsafe_device_id(&device.printer_id) && !unsafe_device_id(&device.name))
        .collect()
}

fn validate_printer_id(value: &str) -> AppResult<()> {
    if value.trim().is_empty() || unsafe_device_id(value) {
        return Err(AppError::Validation("Printer ID is not valid.".into()));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
async fn system_print_pdf(
    pdf_path: &Path,
    printer: &PrinterDevice,
    options: &PrintOptions,
) -> AppResult<()> {
    crate::devices::windows::print_pdf(pdf_path, &printer.name, options.copies).await
}

#[cfg(not(target_os = "windows"))]
async fn system_print_pdf(
    _pdf_path: &Path,
    _printer: &PrinterDevice,
    _options: &PrintOptions,
) -> AppResult<()> {
    Err(AppError::Validation(
        "PDF printing is only available on Windows.".into(),
    ))
}

#[cfg(test)]
pub mod mock {
    use std::{future::Future, path::Path};

    use crate::{
        devices::PrinterDevice,
        error::{AppError, AppResult},
        printing::{PrintOptions, PrintProvider},
    };

    #[derive(Clone, Default)]
    pub struct MockPrintProvider {
        pub printers: Vec<PrinterDevice>,
        pub fail_print: bool,
    }

    impl PrintProvider for MockPrintProvider {
        fn list_printers(&self) -> impl Future<Output = AppResult<Vec<PrinterDevice>>> + Send {
            let printers = self.printers.clone();
            async move { Ok(printers) }
        }

        fn print_pdf(
            &self,
            pdf_path: &Path,
            _printer: &PrinterDevice,
            _options: &PrintOptions,
        ) -> impl Future<Output = AppResult<()>> + Send {
            let exists = pdf_path.is_file();
            let fail_print = self.fail_print;
            async move {
                if fail_print {
                    return Err(AppError::Validation("Print failed.".into()));
                }
                if !exists {
                    return Err(AppError::Validation("Print source is unavailable.".into()));
                }
                Ok(())
            }
        }
    }
}

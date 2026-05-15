use std::path::Path;
use std::process::Command;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    devices::{
        default_capabilities, PrinterDevice, ScanOptions, ScannerCapabilities, ScannerDevice,
    },
    error::{AppError, AppResult},
};

#[derive(Deserialize)]
struct WiaScanner {
    #[serde(rename = "DeviceId")]
    device_id: Option<String>,
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "Manufacturer")]
    manufacturer: Option<String>,
    #[serde(rename = "ConnectionType")]
    connection_type: Option<String>,
}

#[derive(Deserialize)]
struct CimPrinter {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "DeviceID")]
    device_id: Option<String>,
    #[serde(rename = "Default")]
    is_default: Option<bool>,
    #[serde(rename = "WorkOffline")]
    work_offline: Option<bool>,
    #[serde(rename = "Network")]
    is_network: Option<bool>,
    #[serde(rename = "PrinterStatus")]
    printer_status: Option<i64>,
}

pub async fn list_scanners() -> AppResult<Vec<ScannerDevice>> {
    let rows = scanner_rows()?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let raw_device_id = row.device_id?;
            let device_id = opaque_id("scanner", &raw_device_id);
            Some(ScannerDevice {
                name: row.name.unwrap_or_else(|| device_id.clone()),
                device_id,
                manufacturer: row.manufacturer,
                connection_type: row.connection_type,
                is_available: true,
                status: Some("Detected".to_owned()),
            })
        })
        .collect())
}

pub async fn scanner_capabilities(scanner_id: &str) -> AppResult<ScannerCapabilities> {
    let _raw = find_raw_scanner_id(scanner_id)?;
    Ok(default_capabilities(scanner_id, true, "Ready"))
}

pub async fn scan_to_path(
    scanner_id: &str,
    options: &ScanOptions,
    destination: &Path,
) -> AppResult<()> {
    let raw_device_id = find_raw_scanner_id(scanner_id)?;
    let image_format = match options.output_format.as_str() {
        "jpg" => "Jpeg",
        "png" => "Png",
        _ => {
            return Err(AppError::Validation(
                "Scanner capture currently supports PNG or JPG output.".into(),
            ))
        }
    };
    let color_intent = match options.color_mode.as_str() {
        "color" => 1,
        "grayscale" => 2,
        "black_white" => 4,
        _ => 1,
    };
    let destination = destination
        .to_str()
        .ok_or_else(|| AppError::Validation("Scan output path is not valid.".into()))?;
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$rawDeviceId = {raw}
$destination = {destination}
$savePath = $destination
if ($savePath.StartsWith('\\?\')) {{ $savePath = $savePath.Substring(4) }}
$bmpGuid = '{{B96B3CAB-0728-11D3-9D7B-0000F81EF32E}}'
$imageFormat = '{image_format}'
$dpi = {dpi}
$colorIntent = {color_intent}
$manager = New-Object -ComObject WIA.DeviceManager
$deviceInfo = $null
foreach ($info in $manager.DeviceInfos) {{
  if ([string]$info.DeviceID -eq $rawDeviceId) {{ $deviceInfo = $info; break }}
}}
if ($null -eq $deviceInfo) {{ throw 'Selected scanner is not available.' }}
$device = $deviceInfo.Connect()
$item = $device.Items.Item(1)
foreach ($prop in $item.Properties) {{
  try {{
    switch ($prop.PropertyID) {{
      6146 {{ $prop.Value = $colorIntent }}
      6147 {{ $prop.Value = $dpi }}
      6148 {{ $prop.Value = $dpi }}
    }}
  }} catch {{}}
}}
$temp = [System.IO.Path]::ChangeExtension([System.IO.Path]::GetTempFileName(), 'bmp')
$image = $item.Transfer($bmpGuid)
if (Test-Path -LiteralPath $temp) {{ Remove-Item -LiteralPath $temp -Force }}
$image.SaveFile($temp)
Add-Type -AssemblyName System.Drawing
$bitmap = [System.Drawing.Image]::FromFile($temp)
if (Test-Path -LiteralPath $savePath) {{ Remove-Item -LiteralPath $savePath -Force }}
$bitmap.Save($savePath, [System.Drawing.Imaging.ImageFormat]::$imageFormat)
$bitmap.Dispose()
Remove-Item -LiteralPath $temp -Force
"#,
        raw = ps_single_quote(&raw_device_id),
        destination = ps_single_quote(destination),
        image_format = image_format,
        dpi = options.dpi,
        color_intent = color_intent,
    );
    run_powershell(&script)?;
    Ok(())
}

fn scanner_rows() -> AppResult<Vec<WiaScanner>> {
    let script = r#"
$manager = New-Object -ComObject WIA.DeviceManager
$items = @()
foreach ($info in $manager.DeviceInfos) {
  $props = @{}
  foreach ($prop in $info.Properties) { $props[$prop.Name] = [string]$prop.Value }
  $items += [pscustomobject]@{
    DeviceId = [string]$info.DeviceID
    Name = if ($props['Name']) { $props['Name'] } else { [string]$info.DeviceID }
    Manufacturer = $props['Manufacturer']
    ConnectionType = $props['Port']
  }
}
$items | ConvertTo-Json -Compress
"#;
    let output = run_powershell(script)?;
    parse_json_array(&output)
}

fn find_raw_scanner_id(scanner_id: &str) -> AppResult<String> {
    scanner_rows()?
        .into_iter()
        .filter_map(|row| row.device_id)
        .find(|raw| opaque_id("scanner", raw) == scanner_id)
        .ok_or_else(|| AppError::Validation("Selected scanner is not available.".into()))
}

pub async fn list_printers() -> AppResult<Vec<PrinterDevice>> {
    let script = r#"
Get-CimInstance Win32_Printer |
  Select-Object Name, DeviceID, Default, WorkOffline, Network, PrinterStatus |
  ConvertTo-Json -Compress
"#;
    let output = run_powershell(script)?;
    let rows: Vec<CimPrinter> = parse_json_array(&output)?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let name = row.name.or(row.device_id)?;
            let is_available = !row.work_offline.unwrap_or(false);
            Some(PrinterDevice {
                printer_id: opaque_id("printer", &name),
                name,
                is_default: row.is_default.unwrap_or(false),
                status: printer_status(row.printer_status, is_available),
                is_available,
                is_network: row.is_network.unwrap_or(false),
            })
        })
        .collect())
}

pub async fn print_pdf(pdf_path: &Path, printer_name: &str, copies: i64) -> AppResult<()> {
    let pdf_path = pdf_path
        .to_str()
        .ok_or_else(|| AppError::Validation("Print source path is not valid.".into()))?;
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$pdf = {pdf}
$printer = {printer}
$copies = {copies}
if ($pdf.StartsWith('\\?\')) {{ $pdf = $pdf.Substring(4) }}
if (!(Test-Path -LiteralPath $pdf)) {{ throw 'Print source is unavailable.' }}
function Invoke-EdgePdfPrint($sourcePdf, $targetPrinter) {{
  $edge = @(
    "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe",
    "${{env:ProgramFiles(x86)}}\Microsoft\Edge\Application\msedge.exe",
    "${{env:ProgramFiles(x86)}}\Microsoft\EdgeCore\*\msedge.exe",
    "${{env:ProgramFiles(x86)}}\Microsoft\EdgeWebView\Application\*\msedge.exe"
  ) | ForEach-Object {{ Get-ChildItem -Path $_ -ErrorAction SilentlyContinue }} |
      Sort-Object FullName -Descending |
      Select-Object -First 1
  if ($null -eq $edge) {{ throw 'PDF print handler is unavailable. Install or configure a local PDF printer handler.' }}
  $oldDefault = (Get-CimInstance Win32_Printer | Where-Object {{ $_.Default }} | Select-Object -First 1 -ExpandProperty Name)
  $network = New-Object -ComObject WScript.Network
  try {{
    $network.SetDefaultPrinter($targetPrinter)
    $fileUrl = ([System.Uri]::new($sourcePdf)).AbsoluteUri
    $html = [System.IO.Path]::ChangeExtension([System.IO.Path]::GetTempFileName(), 'html')
    $markup = @"
<!doctype html>
<html>
<head><meta charset="utf-8"><title>OVPSPEE Print</title></head>
<body style="margin:0">
  <embed src="$fileUrl" type="application/pdf" style="width:100vw;height:100vh;border:0" />
  <script>setTimeout(function(){{ window.print(); }}, 1200);</script>
</body>
</html>
"@
    Set-Content -LiteralPath $html -Value $markup -Encoding UTF8
    $process = Start-Process -FilePath $edge.FullName -ArgumentList @('--kiosk-printing', '--new-window', ([System.Uri]::new($html)).AbsoluteUri) -PassThru
    Start-Sleep -Seconds 10
    if ($null -ne $process -and -not $process.HasExited) {{
      $process.CloseMainWindow() | Out-Null
      Start-Sleep -Seconds 1
      if (-not $process.HasExited) {{ $process.Kill() }}
    }}
    Remove-Item -LiteralPath $html -Force -ErrorAction SilentlyContinue
  }} finally {{
    if ($oldDefault) {{ $network.SetDefaultPrinter($oldDefault) }}
  }}
}}
for ($i = 0; $i -lt $copies; $i++) {{
  try {{
    $process = Start-Process -FilePath $pdf -Verb PrintTo -ArgumentList $printer -PassThru
    if ($null -ne $process) {{
      $process.WaitForExit(30000) | Out-Null
    }}
  }} catch {{
    Invoke-EdgePdfPrint $pdf $printer
  }}
}}
"#,
        pdf = ps_single_quote(pdf_path),
        printer = ps_single_quote(printer_name),
        copies = copies,
    );
    run_powershell(&script).map_err(|err| match err {
        AppError::Validation(message) if message.contains("PDF print handler is unavailable") => {
            AppError::Validation(
                "PDF print handler is unavailable. Install or configure a local PDF printer handler."
                    .into(),
            )
        }
        other => other,
    })?;
    Ok(())
}

fn opaque_id(prefix: &str, value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let hash = hasher.finalize();
    format!("{prefix}-{}", hex16(&hash))
}

fn hex16(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn run_powershell(script: &str) -> AppResult<String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|err| AppError::Validation(format!("Device detection failed: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(AppError::Validation(if stderr.is_empty() {
            "Device detection failed.".to_owned()
        } else {
            stderr
        }));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn ps_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn parse_json_array<T: for<'de> Deserialize<'de>>(value: &str) -> AppResult<Vec<T>> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    match serde_json::from_str::<Vec<T>>(value) {
        Ok(rows) => Ok(rows),
        Err(_) => serde_json::from_str::<T>(value)
            .map(|row| vec![row])
            .map_err(AppError::from),
    }
}

fn printer_status(status: Option<i64>, is_available: bool) -> String {
    if !is_available {
        return "Offline".to_owned();
    }
    match status.unwrap_or_default() {
        3 => "Idle",
        4 => "Printing",
        5 => "Warmup",
        6 => "Stopped",
        7 => "Offline",
        _ => "Unknown",
    }
    .to_owned()
}

param(
    [ValidateSet("debug", "release")]
    [string] $BuildType = "debug",
    [string] $AdbPath = "$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $AdbPath)) {
    throw "ADB not found at $AdbPath"
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$apkPath = Join-Path $repoRoot "mobile\android\android\app\build\outputs\apk\$BuildType\app-$BuildType.apk"
if (-not (Test-Path -LiteralPath $apkPath)) {
    throw "APK not found at $apkPath. Build it first."
}

$devices = & $AdbPath devices | Select-String -Pattern "device$"
if ($devices.Count -lt 1) {
    throw "No authorized Android device found. Connect phone, enable USB debugging, accept RSA prompt, then rerun."
}
if ($devices.Count -gt 1) {
    throw "Multiple Android devices found. Connect one device or pass a specific ADB command manually."
}

& $AdbPath install -r $apkPath
& $AdbPath shell monkey -p com.ovpspeemobile 1
Write-Host "Installed and launch-tested $apkPath"

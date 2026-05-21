param(
    [string] $SigningEnvPath = ".local\android-signing\release-signing.env.ps1",
    [switch] $SkipDesktopBundle
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$signingEnvFullPath = Join-Path $repoRoot $SigningEnvPath

Push-Location $repoRoot
try {
    if (Test-Path -LiteralPath $signingEnvFullPath) {
        . $signingEnvFullPath
    }

    pnpm build
    pnpm exec vitest run --dir src --pool=threads

    Push-Location "mobile\android"
    try {
        npm test -- --runInBand
        npm run typecheck
        $env:JAVA_HOME = "C:\Program Files\Java\jdk-24"
        $env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
        $env:ANDROID_SDK_ROOT = $env:ANDROID_HOME
        $env:JAVA_TOOL_OPTIONS = "--enable-native-access=ALL-UNNAMED"
        .\android\gradlew.bat -p android :app:assembleRelease
    } finally {
        Pop-Location
    }

    Push-Location "src-tauri"
    try {
        cargo fmt --check
        cargo test --test mobile_submissions_slice18 --test mobile_api_slice18 --test mobile_devices_slice19
    } finally {
        Pop-Location
    }

    if (-not $SkipDesktopBundle) {
        cargo tauri build
    }

    $apkPath = Join-Path $repoRoot "mobile\android\android\app\build\outputs\apk\release\app-release.apk"
    if (-not (Test-Path -LiteralPath $apkPath)) {
        throw "Release APK not found at $apkPath"
    }

    Write-Host "Final release checks passed."
    Write-Host "Android APK: $apkPath"
    if (-not $SkipDesktopBundle) {
        Write-Host "Desktop bundles: $(Join-Path $repoRoot 'src-tauri\target\release\bundle')"
    }
} finally {
    Pop-Location
}

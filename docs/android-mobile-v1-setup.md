# Android Mobile V1 Setup

## Office PC Hub

1. Build and run the desktop app on the office PC.
2. Start the mobile API by setting:

```powershell
$env:OVPSPEE_MOBILE_API_ENABLED = "1"
```

3. Keep the desktop app open while Android devices submit documents.
4. Open `Admin Console > Mobile Devices` and create one token per Android phone.
5. Copy the generated Device ID and Device token to the phone immediately. The token is shown once.
6. Open `Mobile Submissions` on the desktop and use the `Android Setup` panel for the LAN IP and setup link.
7. Use the office PC LAN IP address and port `1421` in the Android app, for example:

```text
http://192.168.1.20:1421
```

8. Allow inbound office-network traffic to port `1421` only on the trusted office network.

Set a custom bind address when needed:

```powershell
$env:OVPSPEE_MOBILE_API_ADDR = "0.0.0.0:1421"
```

## Android App

The debug APK builds to:

```text
mobile/android/android/app/build/outputs/apk/debug/app-debug.apk
```

Install it on an Android device connected to the same office network as the PC. On first launch:

1. Enter the hub URL.
2. Enter the desktop-generated Device ID, device name, and token.
3. Log in with a Secretary account.
4. Add attachments.
5. Complete all document metadata.
6. Submit. The record appears in the desktop `Mobile Submissions` queue as `Pending`.

Drafts, hub URL, device ID/name/token, and retry queue are persisted in Android SharedPreferences. Failed uploads remain queued and can be retried from Capture or History when office Wi-Fi returns. If a phone is lost, revoke it from `Admin Console > Mobile Devices`; the revoked token cannot use the mobile API.

## Desktop Review

Secretaries review submissions in:

```text
/s/mobile-submissions
```

Approving creates the official desktop document and copies the submitted attachments. Rejecting leaves the submission in rejected status with a reason. The review queue supports attachment preview, status/search/date filters, rejection templates, keyboard review shortcuts, and device/submission audit details.

## Build Notes

This project pins the Android wrapper to Gradle `8.14.3` so the app can build with the installed JDK 24. Use:

```powershell
$env:JAVA_HOME = "C:\Program Files\Java\jdk-24"
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
$env:ANDROID_SDK_ROOT = $env:ANDROID_HOME
$env:JAVA_TOOL_OPTIONS = "--enable-native-access=ALL-UNNAMED"
cd mobile/android
.\android\gradlew.bat -p android :app:assembleDebug
```

Final release build:

```powershell
.\scripts\build-final-release.ps1
```

The script runs the desktop build, desktop UI tests, Android tests/typecheck, Android release APK build, Rust mobile tests, and desktop bundle build.

For signed release builds, keep secrets outside git and set:

```powershell
$env:OVPSPEE_ANDROID_KEYSTORE = "C:\secure\ovpspee-release.keystore"
$env:OVPSPEE_ANDROID_KEYSTORE_PASSWORD = "<password>"
$env:OVPSPEE_ANDROID_KEY_ALIAS = "<alias>"
$env:OVPSPEE_ANDROID_KEY_PASSWORD = "<password>"
.\android\gradlew.bat -p android :app:assembleRelease
```

Recommended deployment is approved-device tokens plus trusted office Wi-Fi/firewall. Use the included HTTPS reverse proxy when phones require TLS before reaching port `1421`.

## HTTPS Proxy

Generate a local certificate and key outside git:

```powershell
mkdir .local\mobile-https
openssl req -x509 -newkey rsa:2048 -sha256 -days 825 -nodes `
  -keyout .local\mobile-https\ovpspee-mobile.key `
  -out .local\mobile-https\ovpspee-mobile.crt `
  -subj "/CN=OVPSPEE Mobile Hub"
```

Start the HTTPS reverse proxy:

```powershell
$env:OVPSPEE_MOBILE_HTTP_TARGET = "http://127.0.0.1:1421"
$env:OVPSPEE_MOBILE_HTTPS_PORT = "1443"
pnpm mobile:https-proxy
```

Android hub URL becomes:

```text
https://<office-pc-lan-ip>:1443
```

For office phones, install/trust the generated `.local\mobile-https\ovpspee-mobile.crt` or replace it with an office CA-issued certificate.

## Phone Install Check

With USB debugging enabled and one phone connected:

```powershell
.\scripts\install-android-apk.ps1 -BuildType debug
.\scripts\install-android-apk.ps1 -BuildType release
```

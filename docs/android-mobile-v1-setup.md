# Android Mobile V1 Setup

## Office PC Hub

1. Build and run the desktop app on the office PC.
2. Start the mobile API by setting:

```powershell
$env:OVPSPEE_MOBILE_API_ENABLED = "1"
```

3. Keep the desktop app open while Android devices submit documents.
4. Use the office PC LAN IP address and port `1421` in the Android app, for example:

```text
http://192.168.1.20:1421
```

5. Allow inbound office-network traffic to port `1421` only on the trusted office network.

## Android App

The debug APK builds to:

```text
mobile/android/android/app/build/outputs/apk/debug/app-debug.apk
```

Install it on an Android device connected to the same office network as the PC. On first launch:

1. Enter the hub URL.
2. Log in with a Secretary account.
3. Add attachments.
4. Complete all document metadata.
5. Submit. The record appears in the desktop `Mobile Submissions` queue as `Pending`.

## Desktop Review

Secretaries review submissions in:

```text
/s/mobile-submissions
```

Approving creates the official desktop document and copies the submitted attachments. Rejecting leaves the submission in rejected status with a reason.

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

The v1 Android draft store is intentionally abstracted. Current debug build keeps draft data in app memory to avoid adding another native storage dependency before field testing.

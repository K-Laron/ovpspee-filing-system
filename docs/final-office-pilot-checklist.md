# Final Office Mobile Pilot Checklist

Date: 2026-05-21

## Setup

- Office PC is on trusted office network.
- Desktop app is running with `OVPSPEE_MOBILE_API_ENABLED=1`.
- Android phone is connected to the same office network.
- Admin created one token in `Admin Console > Mobile Devices`.
- Android app has hub URL, Device ID, device name, and token saved.

## Install

```powershell
$adb="$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe"
& $adb devices -l
.\scripts\install-android-apk.ps1 -BuildType release
```

Expected: one authorized device, APK installs, app launches.

## Submission Matrix

- 3 camera photo submissions.
- 3 PDF file picker submissions.
- 2 image file picker submissions.
- 1 invalid/unsupported file type attempt.
- 1 offline queued submission, then retry on office Wi-Fi.

## Desktop Review

- Approve all valid submissions.
- Reject invalid or incorrect submissions with a rejection reason.
- Preview at least one PDF, one image, and one text/unsupported attachment state.
- Confirm approved records appear in Documents with attachments.

## Data Checks

```powershell
cd src-tauri
cargo test --test mobile_submissions_slice18
```

Expected: duplicate `client_submission_id` protection remains green.

## Backup Restore Drill

- Create backup after pilot submissions.
- Restore into a test profile or test PC.
- Confirm documents, attachments, mobile submissions, review statuses, and audit logs exist after restore.

## Sign-Off

- No data loss.
- No duplicate submissions.
- Unknown/revoked device blocked.
- Backup restored successfully.

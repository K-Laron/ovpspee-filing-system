# Upgrade Notes from v0.1.0 MVP

Upgrade target: `v0.2.0-device-scan-print`

Tagged commit: `b9e4616`

## What Changed Since v0.1.0 MVP

- Added device detection/settings.
- Added scanner capture to Scan Intake.
- Added document PDF printing.
- Added evidence for integration audit.

## Database Notes

- Slice 11 device defaults use existing settings table.
- No separate Slice 11 migration was added.
- Scanner capture uses existing Scan Intake/document storage rules.
- SQLite continues storing relative paths for app-controlled stored files.

## Operational Notes

- Admin should review Devices after upgrade and save default scanner/printer.
- Secretary should confirm Scan Intake still shows manual import and scanner capture panel.
- Staff/Head Viewer should confirm public visible document export/print controls.
- Backup/restore page still opens and backup creation was manually confirmed.

## Compatibility Notes

- Windows WIA-compatible scanner needed for scanner capture.
- Windows printer setup needed for document printing.
- Edge print fallback supports systems without PDF file association.
- OCR, TWAIN, reliable ADF/multi-page scanning, and QR/report printing remain deferred.


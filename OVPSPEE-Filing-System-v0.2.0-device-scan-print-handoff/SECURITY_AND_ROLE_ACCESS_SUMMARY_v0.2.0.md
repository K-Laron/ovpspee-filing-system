# Security and Role Access Summary v0.2.0

## Admin / IT Staff

- Can open Devices page.
- Can detect scanners and printers.
- Can save global device defaults.
- Can open Audit Log.
- Can open Backup & Restore.
- Still has no normal filing/create/edit/move/status navigation.

## Secretary

- Can open Documents.
- Can print documents they can access, including hidden/confidential documents.
- Can open Scan Intake.
- Can use scanner capture into intake.
- Manual import remains available.
- Can use attachment preview.

## Staff/Head Viewer / Public Viewer

- Can browse visible public documents.
- Can export/print visible public documents.
- Cannot see hidden/confidential/trashed documents.
- Has no admin device settings.
- Has no scanner capture.
- Has no restricted document print/export path.

## Safe Data Handling

- Scan capture stores app-controlled relative paths in SQLite.
- Print temp/internal paths are not returned to UI.
- Device DTOs do not expose absolute driver paths.
- Audit summaries did not expose file contents, passwords, password hashes, or secrets.
- Device UI may show scanner port string such as `\\.\Usbscan0`; this is not a file path or secret, but can be masked later if stricter device privacy is required.


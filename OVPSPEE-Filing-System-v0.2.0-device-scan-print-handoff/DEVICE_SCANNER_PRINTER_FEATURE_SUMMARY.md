# Device, Scanner, and Printer Feature Summary

## Slice 11: Device Detection and Settings

- Added scanner detection.
- Added printer detection.
- Added Admin Devices page.
- Added Secretary read-only Devices page.
- Stored default scanner/printer/settings using existing settings table.
- Added audit for device settings update.

## Slice 12: Scanner Capture to Scan Intake

- Added scanner capture panel to Secretary Scan Intake.
- Preserved manual file import workflow.
- Used Windows WIA capture through PowerShell.
- Captured scan into app-controlled scan intake storage.
- Stored relative path only in SQLite.
- Avoided large byte transfer through IPC.
- Admin, Staff/Head Viewer, and unauthenticated users cannot scan into intake.

## Slice 13: Document PDF Printing

- Added document PDF print command and UI actions.
- Reused existing PDF export flow.
- Generated temporary app-controlled PDF.
- Did not return temp/internal paths to frontend.
- Cleaned temp PDF after print attempt.
- Used Windows `PrintTo` first.
- Added Edge kiosk print fallback for systems without PDF file association.
- Public viewer can print visible public documents only.
- Secretary can print accessible normal and hidden/confidential documents.
- Admin still has no normal filing/create/edit/move/status navigation.
- Safe print audit events shown.


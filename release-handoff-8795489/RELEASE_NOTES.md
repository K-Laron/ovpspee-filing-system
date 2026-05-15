# OVPSPEE Filing & Tracking System MVP Release Notes

Release commit: `8795489`
Branch: `master`
Release date: May 15, 2026

## Summary

This MVP release includes the approved vertical slices through Backup, Restore, and Deployment Packaging. It is a local Tauri v2 desktop app using Rust, React, TypeScript, and SQLite.

## Included Capabilities

- First-run setup and Admin account creation.
- Argon2id authentication, logout, session validation, and role guards.
- Admin master data management for categories, folders, offices, and settings.
- Admin user management and shared profile management.
- Secretary document filing, attachments, visibility, trash lifecycle, move, and status management.
- Staff/Head Viewer public document browsing.
- Secretary scan intake workflow using native file picker.
- Admin Audit Log and Secretary My Activity.
- PDF export and attachment preview.
- Admin Backup & Restore.
- MSI and NSIS installer packaging.

## Installer Output

- MSI: `D:\codex-target-ovpspee\release\bundle\msi\OVPSPEE Filing System_0.1.0_x64_en-US.msi`
- NSIS: `D:\codex-target-ovpspee\release\bundle\nsis\OVPSPEE Filing System_0.1.0_x64-setup.exe`

## Validation Status

Final MVP release audit passed. Release-blocking issues: none found.


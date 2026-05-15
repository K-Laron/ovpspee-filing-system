# Backup Restore Guide

Release commit: `8795489`

## Admin Backup & Restore

Admin uses Backup & Restore page.

Supported actions:

- View backup settings.
- Update backup destination/schedule settings.
- Create manual backup.
- View backup history.
- Export portable `.ovpspee-backup` archive.
- Validate/import portable archive.
- Restore from selected backup.

## Backup Contents

Backup includes:

- SQLite database copy.
- App storage files.
- Manifest.
- SHA-256 checksums.
- Backup metadata.

## Restore Behavior

- Restore validates backup before applying.
- Restore creates pre-restore safety backup.
- Restore replaces DB/storage with backup data.
- Restore reports restart required.
- App restart is manual by design.

## Confirmed Validation

- Manual backup created DB + storage + manifest/checksum.
- Portable backup export works.
- Restore confirmation appears.
- Restore reports restart required.
- Backup/restore audit events recorded.

## Evidence

- `manual-final-release-audit/final-admin-backup-restore.png`
- `manual-final-release-audit/final-backup-created.png`


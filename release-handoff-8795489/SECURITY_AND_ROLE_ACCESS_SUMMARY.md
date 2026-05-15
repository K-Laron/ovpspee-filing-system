# Security And Role Access Summary

Release commit: `8795489`

## Security Controls

- Passwords use Argon2id hashing.
- Plain-text passwords are not logged.
- Password hashes are not exposed in safe audit DTOs.
- Audit responses sanitize sensitive strings such as password, password hash, token, and secret patterns.
- SQLite stores app-managed relative file paths for documents, attachments, scan intake, and backups where applicable.
- Backend validates path traversal for file, backup, restore, archive, export, and preview flows.
- Large file limits are enforced by backend tests.

## Role Boundaries

### Admin / IT Staff

- Can manage users, master data, audit log, retention setting, backup, restore, purge/admin areas.
- Should not have normal document filing/create/edit/move/status workflows.

### Secretary

- Can manage documents according to approved MVP scope.
- Can create/edit/file/hide/trash/restore/move/status documents.
- Can use scan intake.
- Can export accessible documents, including hidden/confidential documents.
- Can view own activity only.

### Staff/Head Viewer

- Read-only.
- Can browse public visible documents.
- Can export visible non-hidden, non-confidential, non-trashed public documents.
- Cannot access hidden/confidential/trashed documents.

### Viewer / Unauthenticated

- No admin audit access.
- No backup/restore access.
- No secretary document management access.

## Validation

Role boundaries are covered by final passing backend tests and manual smoke evidence.


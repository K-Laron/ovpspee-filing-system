# Security Compliance Checklist
## OVPSPEE Filing & Tracking System — CDHP Document 12

---

## 1. Purpose

This checklist is a developer-facing security reference. Each item maps to a security requirement from the PRD or the system design. Review this document before implementing any feature that touches authentication, file I/O, session management, or user data.

---

## 2. Authentication & Password Security

| # | Requirement | Implementation | Verified By |
|---|---|---|---|
| SEC-01 | Passwords stored as Argon2id hash (min recommended parameters: memory=64MB, iterations=3, parallelism=4) | `argon2` crate in Rust | Unit test: `test_password_hash_and_verify` |
| SEC-02 | Plain-text passwords never written to disk, logged, or included in audit log entries | Code review — no `password` field in any log write | Manual code audit |
| SEC-03 | Password complexity enforced: min 8 chars, ≥1 number, ≥1 special character | Validation in `create_user`, `change_my_password`, `admin_reset_password` | Unit test: `test_create_user_weak_password` |
| SEC-04 | Deactivated users (`is_active = 0`) cannot authenticate, regardless of correct credentials | `require_session()` checks `is_active` after password verification | Unit test: `test_login_deactivated_user` |
| SEC-05 | Session tokens are cryptographically random UUIDs (v4) | `uuid::Uuid::new_v4()` in Rust | Code review |
| SEC-06 | Sessions expire after 8 hours of inactivity (or absolute expiry) | `session.expires_at` checked in `require_session()` | Unit test: `test_validate_session_expired` |
| SEC-07 | Expired sessions are cleaned up on app startup | Startup hook in `lib.rs` deletes expired session records | Integration test |
| SEC-08 | No self-registration. Only Admin creates accounts | No `register` command exists; `create_user` requires Admin session | Manual: no register endpoint |
| SEC-09 | No email/token password reset flow (Admin-only reset) | `admin_reset_password` only; no token generation or email sending | Code review |

---

## 3. Authorization & Role Enforcement

| # | Requirement | Implementation | Verified By |
|---|---|---|---|
| AUTHZ-01 | Every protected Tauri command calls `require_session()` before any logic | Present at the top of every non-public command | Code review per command |
| AUTHZ-02 | Admin-only commands call `require_admin_role()` after session validation | Present in all commands under `commands/users.rs`, `commands/backup.rs`, `commands/audit_log.rs`, `commands/categories.rs` | Code review |
| AUTHZ-03 | Secretary cannot access Admin commands | `require_admin_role()` returns `ERR_UNAUTHORIZED` for Secretary sessions | Unit test: `test_non_admin_cannot_manage_users` |
| AUTHZ-04 | Staff/Head Viewer (no session) can only call explicitly public commands (`list_documents`, `get_document`, `export_document_pdf`, `list_public_categories`, `list_public_folders`) | Public commands accept `session_id: Option<String>`; apply guest-level filtering when `None` | Manual: attempt admin command without session |
| AUTHZ-05 | `user_id` used in operations is always derived from the validated session, never trusted from client input | Session validation returns `user_id`; no command accepts `user_id` as a parameter for privileged operations | Code review |
| AUTHZ-06 | System categories (`is_system = 1`) cannot be edited, deactivated, or have folders created under them | Guards in `update_category` and `create_folder` commands | Unit tests: `test_cannot_edit_system_category`, `test_create_folder_under_trash` |

---

## 4. File System Security

| # | Requirement | Implementation | Verified By |
|---|---|---|---|
| FS-01 | All file paths in the database are stored as relative paths | `attachment.file_path` and `scan_intake.file_path` are relative; resolved at runtime | Code review |
| FS-02 | Path traversal attacks are blocked | `safe_path()` in `storage/mod.rs` — joins relative path with base dir, canonicalizes, and verifies the result is still within base dir | Unit test: `test_serve_attachment_path_traversal` |
| FS-03 | Storage base directory is validated and created on startup | `db/init.rs` creates `storage/`, `storage/documents/`, `storage/intake/`, `storage/intake/thumbnails/`, `storage/profiles/` on first run | Manual: fresh install verification |
| FS-04 | File type validation on upload and intake import | Check MIME type and file extension; reject non-allowed types | Unit test (attachment file type validation) |
| FS-05 | File size limits enforced | Max 1 GB per attachment/scan file; UI warns above 250 MB; checked before writing to disk | Unit test (file size limit) |
| FS-06 | Files are not executable — they are stored and served as binary data only | No file execution path in the codebase | Code review |
| FS-07 | Backup files are not encrypted by the application | Documented in Admin deployment notes; default local backups must show warning to copy/store securely on external/network storage | Documentation review |

---

## 5. Database Security

| # | Requirement | Implementation | Verified By |
|---|---|---|---|
| DB-01 | SQLite WAL mode enabled | `PRAGMA journal_mode=WAL;` set in connection setup | Code review |
| DB-02 | Foreign keys enforced | `PRAGMA foreign_keys=ON;` set in connection setup | Code review |
| DB-03 | All SQL queries use parameterized queries (no string interpolation) | `sqlx::query_as!` macro only; no raw string SQL construction | Code review |
| DB-04 | Database file stored in OS-protected app data directory, not user-browsable by default | `%APPDATA%\ovpspee-filing-system\` (Windows) — not in `Documents` or `Desktop` | Manual: path verification on install |
| DB-05 | No hard deletes on lookup tables | `is_active = 0` for deactivation; no `DELETE FROM user/category/folder/office` | Code review; database query audit |

---

## 6. Session & Transport Security

| # | Requirement | Implementation | Verified By |
|---|---|---|---|
| SESS-01 | Session IDs are not stored in frontend local storage or cookies | Session ID held in Zustand in-memory store only; not persisted to localStorage | Code review |
| SESS-02 | Session is cleared from memory on logout | `clearSession()` called in Zustand store; session deleted from DB | Manual: logout + inspect Zustand devtools |
| SESS-03 | No HTTP server — all communication via Tauri IPC | No HTTP listener opened; no network port exposed | Confirmed by Tauri architecture |
| SESS-04 | IPC commands cannot be called from external web pages | Tauri's Content Security Policy restricts allowed origins | `tauri.conf.json` CSP review |

---

## 7. Audit & Accountability

| # | Requirement | Implementation | Verified By |
|---|---|---|---|
| AUDIT-01 | Every data-modifying operation produces an `audit_log` entry | `write_audit_log()` called in all create, update, delete, move, hide, trash, restore, purge, login, logout, backup, restore, scan commands | Code review per command; integration tests |
| AUDIT-02 | Audit log entries include `user_id`, `timestamp`, `table_affected`, `record_id`, `description` | `write_audit_log()` signature enforces these fields | Code review |
| AUDIT-03 | Move operations log previous and new location | `move_document` writes `"Moved document #N from category_id=X, folder_id=Y to category_id=A, folder_id=B"` | Unit test: `test_audit_log_written_on_move` |
| AUDIT-04 | Audit log records are not user-deletable (only system retention cleanup) | No `delete_audit_log` command; only `run_audit_cleanup` which uses the retention policy | Code review |
| AUDIT-05 | Retention cleanup itself is logged | `run_audit_cleanup` writes a CLEANUP entry before deleting | Code review |

---

## 8. Input Validation Rules

All user inputs must be validated in the Rust backend before any database write or file operation. Frontend validation is UX only — backend validation is the security boundary.

| Field | Rule |
|---|---|
| `username` | 3–50 characters; alphanumeric, underscore, hyphen only; case-insensitive uniqueness |
| `password` (new) | Min 8 characters; ≥1 digit; ≥1 special character (`!@#$%^&*()-_=+[]{}|;:,.<>?`) |
| `email` | Valid email format (RFC 5322 basic check); optional field |
| `color_code` | Valid 6-digit hex color (`#RRGGBB`) |
| `category_name` | 1–100 characters; non-empty after trim |
| `folder_name` | 1–100 characters; non-empty after trim |
| `document_name` | 1–255 characters; non-empty after trim |
| `date_received` | ISO 8601 date format (`YYYY-MM-DD`); must not be in the future |
| `document_status` | Must be one of: `Filed`, `Archived`, `Confidential`, `Other`; `Confidential` forces `is_hidden = 1` |
| `status_other` | Required and non-empty if `document_status = Other`; max 155 characters |
| `retention_months` | Integer; default 36 months; policy range 24–36 months unless client policy changes |
| `trash_purge_days` | Integer; minimum 1; 0 = disabled |
| `file_type` (attachment) | Must be in allowed list; validated against actual file signature (magic bytes), not just extension |
| `file_size_bytes` | Max 1 GB (1,073,741,824 bytes) per attachment/scan |

---

## 9. Pre-Release Security Checklist

Run through this list before tagging any release version.

- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `cargo audit` shows no HIGH or CRITICAL advisories
- [ ] `pnpm audit` shows no HIGH or CRITICAL vulnerabilities
- [ ] Path traversal test passes: `test_serve_attachment_path_traversal`
- [ ] All SQL queries use parameterized form (no string concatenation in SQL)
- [ ] No plain-text password appears in any log output (manual check during login/create user)
- [ ] Session is invalidated after logout (manual check: logout, then invoke a protected command with the old session_id → `ERR_UNAUTHORIZED`)
- [ ] Admin commands reject Secretary sessions (manual check with Secretary session)
- [ ] Staff/Head Viewer filtering correct: hidden/confidential and trashed documents not returned (manual check)
- [ ] TRASH category cannot be edited (manual check in Admin → Master Data)
- [ ] File upload rejects disallowed types (manual: upload a .exe file)
- [ ] Installer code-signed (if certificate available)
- [ ] `tauri.conf.json` CSP is set and does not include `unsafe-eval` or `unsafe-inline` for scripts

---

*End of Security Compliance Checklist*
*End of CDHP — All 12 documents complete.*

# Database Schema Documentation
## OVPSPEE Filing & Tracking System — CDHP Document 02

---

## 1. Overview

The system uses a single **SQLite** database file. No external database server is required. The database is embedded within the Tauri application and stored at:

```
Windows: %APPDATA%\ovpspee-filing-system\filing_system.db
Linux:   ~/.local/share/ovpspee-filing-system/filing_system.db
```

All structured data lives in this file. Binary attachments (files, scans) are stored on the filesystem and referenced by relative path.

**ORM:** `SQLx` (async, compile-time verified queries). Migrations managed by `sqlx-cli` using the `migrations/` folder at the project root.

---

## 2. Schema Design Principles

1. **No hard deletes on lookup tables.** Categories, folders, offices, and users use `is_active` for soft deactivation. Records are never deleted; references remain intact.
2. **Relative file paths only.** `attachment.file_path` stores paths relative to the configured base storage directory. Never absolute paths.
3. **Immutable TRASH category.** The TRASH category is seeded during first-run setup and cannot be edited or deleted. `category.is_system = TRUE` marks it.
4. **Document-level visibility.** Documents have `is_hidden` and `is_trashed` flags. Category-level visibility traits are removed (all categories are OPEN/public).
5. **Move is metadata-only.** Moving a document updates `category_id` and `folder_id` on the `document` table only. No files are moved on disk.
6. **Referential integrity via FK constraints.** All foreign keys are declared with `ON DELETE RESTRICT` to prevent orphaned records.

---

## 3. Tables

### 3.1 `role`

Stores the two system roles. Seeded at first run. Never modified.

```sql
CREATE TABLE role (
    role_id   INTEGER PRIMARY KEY AUTOINCREMENT,
    role_name TEXT    NOT NULL UNIQUE  -- 'Admin', 'Secretary'
);
```

**Seeds:**
```sql
INSERT INTO role (role_name) VALUES ('Admin'), ('Secretary');
```

---

### 3.2 `user`

Stores all authenticated user accounts (Admin and Secretary).

```sql
CREATE TABLE user (
    user_id          INTEGER PRIMARY KEY AUTOINCREMENT,
    role_id          INTEGER NOT NULL REFERENCES role(role_id) ON DELETE RESTRICT,
    first_name       TEXT    NOT NULL,
    middle_name      TEXT,
    last_name        TEXT    NOT NULL,
    username         TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    email            TEXT    UNIQUE COLLATE NOCASE,
    contact_number   TEXT,
    address          TEXT,
    password_hash    TEXT    NOT NULL,           -- bcrypt or Argon2id hash
    profile_pic_path TEXT,                       -- relative path to profile picture file
    is_active        INTEGER NOT NULL DEFAULT 1, -- 1 = active, 0 = deactivated
    last_login_at    TEXT,                       -- ISO 8601 timestamp
    created_at       TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at       TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

**Notes:**
- Deactivated users (`is_active = 0`) cannot log in.
- `username` is case-insensitive (`COLLATE NOCASE`).
- `profile_pic_path` is relative to the storage base directory.
- `password_reset_token_hash` and `token_expires_at` are **not stored** — Admin-only password reset is used instead (see Security doc).

---

### 3.3 `session`

Stores active login sessions. In-memory is sufficient for single-user desktop, but persisting sessions allows resuming state after unexpected closure.

```sql
CREATE TABLE session (
    session_id  TEXT    PRIMARY KEY,             -- UUID v4
    user_id     INTEGER NOT NULL REFERENCES user(user_id) ON DELETE CASCADE,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    expires_at  TEXT    NOT NULL                 -- ISO 8601; session invalid after this
);
```

**Notes:**
- Session is invalidated on logout. Expired sessions are cleaned up on startup.
- Only one active session per user is enforced at the application level (single-machine desktop).

---

### 3.4 `office`

Lookup table for sender offices or departments.

```sql
CREATE TABLE office (
    office_id    INTEGER PRIMARY KEY AUTOINCREMENT,
    office_name  TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    description  TEXT,
    is_active    INTEGER NOT NULL DEFAULT 1,
    created_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

---

### 3.5 `category`

Top-level document classification. All categories are OPEN (visible to all users). Only the TRASH category has special system behavior.

```sql
CREATE TABLE category (
    category_id   INTEGER PRIMARY KEY AUTOINCREMENT,
    category_name TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    description   TEXT,
    color_code    TEXT    NOT NULL DEFAULT '#64748B', -- hex color for UI rendering
    icon          TEXT,                               -- icon identifier (Lucide icon name)
    is_system     INTEGER NOT NULL DEFAULT 0,         -- 1 = system category (e.g., TRASH), immutable
    is_active     INTEGER NOT NULL DEFAULT 1,
    created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

**Seeds (first-run):**
```sql
INSERT INTO category (category_name, description, color_code, icon, is_system)
VALUES ('TRASH', 'System trash — documents pending permanent deletion', '#64748B', 'Trash2', 1);
```

**Constraints enforced in application logic:**
- `is_system = 1` categories cannot be edited, deactivated, or deleted.
- TRASH is always rendered last in category lists, regardless of alphabetical order.
- All user-created categories are always sorted alphabetically.
- TRASH is hidden from no-login Staff/Head Viewer users (application-level filter, not a DB column).

**Removed from PRD v1.1:** `visibility_trait`, `can_soft_delete` columns. These are no longer category-level attributes.

---

### 3.6 `folder`

Organizational containers for documents, linked to a parent category.

```sql
CREATE TABLE folder (
    folder_id    INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id  INTEGER NOT NULL REFERENCES category(category_id) ON DELETE RESTRICT,
    folder_name  TEXT    NOT NULL COLLATE NOCASE,
    description  TEXT,
    folder_color TEXT    NOT NULL DEFAULT '#64748B', -- hex color for folder card UI
    is_active    INTEGER NOT NULL DEFAULT 1,
    created_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    UNIQUE (category_id, folder_name)               -- folder names unique within a category
);
```

**Notes:**
- TRASH category has **no folders**. Application logic must prevent folder creation under `is_system = 1` categories.
- Documents trashed from any category lose their folder association at the `document` level (see `document.original_folder_id`).

---

### 3.7 `document`

Core document metadata record. Central table of the system.

```sql
CREATE TABLE document (
    document_id          INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Filing location (current)
    category_id          INTEGER NOT NULL REFERENCES category(category_id) ON DELETE RESTRICT,
    folder_id            INTEGER          REFERENCES folder(folder_id)   ON DELETE RESTRICT,
    -- folder_id is nullable: trashed documents lose their folder reference

    -- Document metadata
    document_name        TEXT    NOT NULL,
    sender_name          TEXT    NOT NULL,
    sender_office_id     INTEGER REFERENCES office(office_id) ON DELETE SET NULL,
    receiver_name        TEXT,
    date_received        TEXT    NOT NULL,  -- ISO 8601 date (YYYY-MM-DD)
    remarks              TEXT,
    document_status      TEXT    NOT NULL DEFAULT 'Filed'
                         CHECK(document_status IN ('Filed', 'Archived', 'Confidential', 'Other')),
    status_other         TEXT,              -- nullable; used only when document_status = 'Other'

    -- Visibility flags (document-level, Secretary-controlled)
    is_hidden            INTEGER NOT NULL DEFAULT 0, -- 1 = hidden from no-login Staff/Head Viewer users
    hidden_at            TEXT,                        -- ISO 8601 timestamp when hidden

    -- Trash / soft-delete
    is_trashed           INTEGER NOT NULL DEFAULT 0, -- 1 = in TRASH
    trashed_at           TEXT,                        -- ISO 8601 timestamp when trashed
    original_category_id INTEGER REFERENCES category(category_id) ON DELETE SET NULL,
    original_folder_id   INTEGER REFERENCES folder(folder_id)     ON DELETE SET NULL,
    -- original_* populated when document is trashed; used to restore to original location

    -- Record-keeping
    added_by             INTEGER NOT NULL REFERENCES user(user_id) ON DELETE RESTRICT,
    date_added           TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at           TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

**Business Rules:**
- `is_hidden = 1` → document invisible to no-login Staff/Head Viewer users. Secretary and Admin can see it.
- `is_trashed = 1` → document is in the virtual TRASH view. Its `category_id` is updated to the TRASH category ID; `folder_id` is set to NULL.
- `original_category_id` and `original_folder_id` are set when document is trashed. They are cleared on permanent deletion. They are used to restore the document.
- `document_status = 'Confidential'` automatically sets `is_hidden = 1` by default. Confidential documents are hidden from the Staff/Head Viewer unless an Admin-approved future policy explicitly allows otherwise. The UI must show a warning: "Confidential documents are hidden from viewer access."
- `status_other` must be NULL unless `document_status = 'Other'`. Enforce in application logic.

---

### 3.8 `attachment`

File or scan references linked to a document. One document can have many attachments.

```sql
CREATE TABLE attachment (
    attachment_id   INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id     INTEGER NOT NULL REFERENCES document(document_id) ON DELETE CASCADE,
    file_path       TEXT    NOT NULL,   -- relative path from storage base dir
    file_name       TEXT    NOT NULL,   -- original filename or "scan_001.jpg"
    file_type       TEXT    NOT NULL,   -- MIME type: "application/pdf", "image/jpeg", etc.
    file_size_bytes INTEGER NOT NULL,
    source          TEXT    NOT NULL DEFAULT 'upload'
                    CHECK(source IN ('upload', 'scan')),
    sort_order      INTEGER NOT NULL DEFAULT 0, -- display/export order; drag-to-reorder
    uploaded_at     TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

**Storage path convention:**
```
storage/
  documents/
    {document_id}/
      uploaded/     ← files uploaded via file picker
        report.pdf
        letter.docx
      scans/        ← pages claimed from scan intake
        scan_001.jpg
        scan_002.jpg
```

---

### 3.9 `scan_intake`

Staging area for scanned pages. Scans stay here until claimed by a document (moved to `attachment` + file relocated) or deleted.

```sql
CREATE TABLE scan_intake (
    intake_id     INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path     TEXT    NOT NULL,   -- relative path under storage/intake/
    file_name     TEXT    NOT NULL,   -- e.g., "scan_20260512_143001_001.jpg"
    file_size_bytes INTEGER NOT NULL,
    thumbnail_path TEXT,              -- relative path to generated thumbnail
    scanned_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    is_claimed    INTEGER NOT NULL DEFAULT 0, -- 1 = claimed by a document (attachment created)
    claimed_at    TEXT,
    claimed_by_document_id INTEGER REFERENCES document(document_id) ON DELETE SET NULL,

    -- Recoverable delete state for unclaimed intake scans
    is_deleted    INTEGER NOT NULL DEFAULT 0, -- 1 = hidden from normal intake; recoverable until purge
    deleted_at    TEXT
);
```

**Storage path convention:**
```
storage/
  intake/
    scan_20260512_143001_001.jpg
    scan_20260512_143001_002.jpg
    scan_20260512_143015_001.jpg
    thumbnails/
      scan_20260512_143001_001_thumb.jpg
```

**Lifecycle:**
1. Secretary scans → file saved to `storage/intake/`, record inserted into `scan_intake`.
2. Secretary creates document and picks scans → selected scans are "claimed":
   - File moved from `storage/intake/` to `storage/documents/{document_id}/scans/`
   - `attachment` record created for the document
   - `scan_intake.is_claimed = 1`, `claimed_at` and `claimed_by_document_id` set
3. If secretary removes a scan from a document (during creation, before save) → file moved back to `storage/intake/`, `is_claimed` reset to 0.
4. If Secretary deletes an unclaimed scan from intake → `is_deleted = 1`, `deleted_at = now`; file remains in storage and appears in the Deleted Scans recovery view.
5. Secretary may restore a deleted unclaimed scan back to intake by clearing `is_deleted` and `deleted_at`.
6. Deleted scans are permanently purged by retention cleanup after `settings.deleted_scan_retention_days` or by Admin/IT Staff maintenance action.
7. Claimed scans are filtered from the intake picker UI (only `is_claimed = 0 AND is_deleted = 0` scans are shown as available).

---

### 3.10 `settings`

Key-value store for system-wide configuration. Admin-managed.

```sql
CREATE TABLE settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

**Default seeds:**
```sql
INSERT INTO settings (key, value) VALUES
    ('audit_log_retention_months', '36'),
    ('trash_auto_purge_days', '30'),
    ('backup_schedule', 'disabled'),       -- 'disabled', 'daily', 'weekly'
    ('backup_time', '02:00'),
    ('backup_destination', 'local_app_data_backups'), -- default local device folder
    ('backup_retention_count', '10'),
    ('deleted_scan_retention_days', '30'),
    ('storage_base_dir', '');              -- empty = use app data dir default
```

---

### 3.11 `audit_log`

System-wide immutable action log.

```sql
CREATE TABLE audit_log (
    log_id         INTEGER PRIMARY KEY AUTOINCREMENT,
    log_action     TEXT    NOT NULL
                   CHECK(log_action IN ('INSERT', 'UPDATE', 'DELETE', 'MOVE', 'LOGIN', 'LOGOUT',
                                        'BACKUP', 'RESTORE', 'EXPORT', 'IMPORT', 'CLEANUP', 'HIDE',
                                        'UNHIDE', 'TRASH', 'RESTORE_TRASH', 'PURGE', 'SCAN')),
    table_affected TEXT,                  -- e.g., 'document', 'user', 'category'
    record_id      INTEGER,               -- PK of affected record (nullable for system ops)
    description    TEXT    NOT NULL,      -- human-readable summary of the action
    user_id        INTEGER REFERENCES user(user_id) ON DELETE SET NULL,
    ip_address     TEXT,                  -- loopback IP for single-machine deployments
    timestamp      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

**New log actions (from README adjustments):**
- `HIDE` — Secretary hides a document
- `UNHIDE` — Secretary un-hides a document
- `TRASH` — Secretary trashes a document
- `RESTORE_TRASH` — Secretary or Admin restores a document from trash
- `PURGE` — Admin permanently deletes a trashed document or empties trash
- `SCAN` — Secretary scans pages via Scan Intake

**Retention:** Entries older than `settings.audit_log_retention_months` are deleted automatically on startup and by daily timer. Default is 36 months. Admin/IT Staff may configure 24–36 months unless a stricter client policy is later provided.

---

## 4. Indexes

```sql
-- Fast document lookups by location
CREATE INDEX idx_document_category    ON document(category_id);
CREATE INDEX idx_document_folder      ON document(folder_id);
CREATE INDEX idx_document_trashed     ON document(is_trashed);
CREATE INDEX idx_document_hidden      ON document(is_hidden);
CREATE INDEX idx_document_status      ON document(document_status);
CREATE INDEX idx_document_date        ON document(date_received);
CREATE INDEX idx_document_added       ON document(date_added);
CREATE INDEX idx_document_search_sort ON document(is_trashed, is_hidden, date_received);

-- Attachment lookup by document
CREATE INDEX idx_attachment_document  ON attachment(document_id);
CREATE INDEX idx_attachment_order     ON attachment(document_id, sort_order);

-- Scan intake available scans
CREATE INDEX idx_intake_unclaimed     ON scan_intake(is_claimed, is_deleted);
CREATE INDEX idx_intake_deleted       ON scan_intake(is_deleted, deleted_at);

-- Audit log filtering
CREATE INDEX idx_audit_timestamp      ON audit_log(timestamp);
CREATE INDEX idx_audit_user           ON audit_log(user_id);
CREATE INDEX idx_audit_action         ON audit_log(log_action);

-- Folder lookup by category
CREATE INDEX idx_folder_category      ON folder(category_id);
```

---

## 5. Migration Strategy

Migrations are managed via `sqlx-cli`. Migration files live in `migrations/` at the project root.

```
migrations/
  0001_initial_schema.sql        ← role, user, session tables
  0002_master_data.sql           ← category, folder, office tables
  0003_documents.sql             ← document, attachment tables
  0004_scan_intake.sql           ← scan_intake table
  0005_settings_and_audit.sql    ← settings, audit_log tables
  0006_indexes.sql               ← all CREATE INDEX statements
  0007_seeds.sql                 ← role seeds, TRASH category seed, default settings
```

At application startup, the Rust backend runs `sqlx::migrate!().run(&pool)` to apply any pending migrations. This handles first-run setup and future upgrades automatically.

---

## 6. Full Schema Relationship Diagram

```
role ──────────────────────── user
                               │
                    ┌──────────┴──────────┐
                    │                     │
                  session           audit_log

office ────────── document ──── attachment
                    │
                    ├── category (current location)
                    ├── folder   (current location)
                    ├── category (original — for trash restore)
                    └── folder   (original — for trash restore)

category ──────── folder

scan_intake (staging; claimed scans → attachment)

settings (standalone key-value config)
```

---

## 7. Data Integrity Notes

- **Do not hard-delete documents.** Use `is_trashed = 1` and move to TRASH. Permanent deletion is a separate admin-triggered action.
- **Do not hard-delete users, categories, folders, or offices.** Use `is_active = 0`.
- **Do not store absolute file paths.** All `file_path` values must be relative to the configured storage base directory.
- **SQLite WAL mode** should be enabled for better concurrency and crash recovery:
  ```sql
  PRAGMA journal_mode=WAL;
  PRAGMA foreign_keys=ON;
  PRAGMA synchronous=NORMAL;
  ```
  Set these PRAGMAs after opening the connection, before running migrations.

---

*End of Database Schema Documentation*
*Next: `03_Backend_API_Documentation.md`*

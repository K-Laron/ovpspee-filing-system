# Backend API Documentation
## OVPSPEE Filing & Tracking System — CDHP Document 03

---

## 1. Overview

The Rust backend exposes all application logic through **Tauri IPC commands**. The React frontend calls these via `@tauri-apps/api/core`'s `invoke()` function. There is no HTTP server. All communication is local and synchronous from the frontend's perspective (async under the hood via Tauri's command system).

### Calling Pattern (Frontend)

```typescript
import { invoke } from '@tauri-apps/api/core';

// Success
const result = await invoke<DocumentRecord>('get_document', { documentId: 42 });

// Error (Tauri surfaces Rust Err(...) as a rejected Promise)
try {
  await invoke('delete_scan', { intakeId: 7 });
} catch (error) {
  // error is the string returned from the Rust Err(AppError::...)
  console.error(error);
}
```

### Error Format

All commands return `Result<T, String>` from Rust. On error, the frontend receives a plain string error message. Use a centralized `handleError(e: unknown)` utility in the frontend to display toast notifications.

### Standard Error Codes

| Code String | Meaning |
|---|---|
| `ERR_NOT_FOUND` | Record does not exist |
| `ERR_UNAUTHORIZED` | Session invalid or role insufficient |
| `ERR_VALIDATION` | Input failed validation |
| `ERR_DUPLICATE` | Unique constraint violation |
| `ERR_SYSTEM_RECORD` | Attempted to modify an immutable system record |
| `ERR_IO` | File system error |
| `ERR_DB` | Database error |
| `ERR_CONFLICT` | Operation conflicts with current state |

---

## 2. Authentication Commands

### `first_run_check`
Returns whether first-run setup is needed.

```rust
#[tauri::command]
async fn first_run_check(db: State<DbPool>) -> Result<bool, String>
// Returns true if no admin account exists yet
```

---

### `first_run_setup`
Creates the initial Admin account. Only callable when no admin exists.

```rust
#[tauri::command]
async fn first_run_setup(
    db: State<DbPool>,
    first_name: String,
    last_name: String,
    username: String,
    password: String,
) -> Result<(), String>
// Hashes password with Argon2id, inserts admin user, seeds TRASH category and default settings
// Errors: ERR_CONFLICT (already set up), ERR_VALIDATION (weak password)
```

---

### `login`
Authenticates a user and creates a session.

```rust
#[tauri::command]
async fn login(
    db: State<DbPool>,
    username: String,
    password: String,
) -> Result<SessionPayload, String>

pub struct SessionPayload {
    pub session_id: String,
    pub user_id: i64,
    pub role: String,        // "Admin" | "Secretary"
    pub display_name: String,
    pub profile_pic_path: Option<String>,
}
// Errors: ERR_UNAUTHORIZED (invalid credentials or deactivated)
// Side effects: updates user.last_login_at, writes LOGIN to audit_log
```

---

### `logout`
Invalidates the current session.

```rust
#[tauri::command]
async fn logout(
    db: State<DbPool>,
    session_id: String,
) -> Result<(), String>
// Side effects: deletes session record, writes LOGOUT to audit_log
```

---

### `validate_session`
Checks if a session is still valid. Called on app resume/reload.

```rust
#[tauri::command]
async fn validate_session(
    db: State<DbPool>,
    session_id: String,
) -> Result<SessionPayload, String>
// Errors: ERR_UNAUTHORIZED (expired or not found)
```

---

## 3. User Management Commands (Admin Only)

All commands in this section require an active Admin session. Validate role before executing.

### `list_users`

```rust
#[tauri::command]
async fn list_users(
    db: State<DbPool>,
    session_id: String,
    search: Option<String>,
    role_filter: Option<String>,
    status_filter: Option<String>,  // "active" | "deactivated"
    sort_by: Option<String>,        // "name" | "role" | "last_login"
    sort_dir: Option<String>,       // "asc" | "desc"
) -> Result<Vec<UserListItem>, String>

pub struct UserListItem {
    pub user_id: i64,
    pub full_name: String,
    pub username: String,
    pub role: String,
    pub last_login_at: Option<String>,
    pub is_active: bool,
}
```

---

### `create_user`

```rust
#[tauri::command]
async fn create_user(
    db: State<DbPool>,
    session_id: String,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    username: String,
    email: Option<String>,
    contact_number: Option<String>,
    role: String,         // "Admin" | "Secretary"
    password: String,
) -> Result<i64, String>  // Returns new user_id
// Errors: ERR_DUPLICATE (username taken), ERR_VALIDATION (weak password)
// Side effects: INSERT audit_log
```

---

### `update_user`

```rust
#[tauri::command]
async fn update_user(
    db: State<DbPool>,
    session_id: String,
    user_id: i64,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    email: Option<String>,
    contact_number: Option<String>,
    address: Option<String>,
    is_active: bool,
) -> Result<(), String>
// Side effects: UPDATE audit_log
```

---

### `admin_reset_password`
Admin sets a new password for any user directly (no email/token flow).

```rust
#[tauri::command]
async fn admin_reset_password(
    db: State<DbPool>,
    session_id: String,
    target_user_id: i64,
    new_password: String,
) -> Result<(), String>
// Errors: ERR_VALIDATION (weak password)
// Side effects: UPDATE audit_log
```

---

## 4. Profile Commands (All Authenticated Users)

### `get_my_profile`

```rust
#[tauri::command]
async fn get_my_profile(
    db: State<DbPool>,
    session_id: String,
) -> Result<UserProfile, String>

pub struct UserProfile {
    pub user_id: i64,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub username: String,
    pub email: Option<String>,
    pub contact_number: Option<String>,
    pub address: Option<String>,
    pub role: String,
    pub profile_pic_path: Option<String>,
}
```

---

### `update_my_profile`

```rust
#[tauri::command]
async fn update_my_profile(
    db: State<DbPool>,
    session_id: String,
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
    email: Option<String>,
    contact_number: Option<String>,
    address: Option<String>,
) -> Result<(), String>
```

---

### `change_my_password`

```rust
#[tauri::command]
async fn change_my_password(
    db: State<DbPool>,
    session_id: String,
    current_password: String,
    new_password: String,
) -> Result<(), String>
// Errors: ERR_UNAUTHORIZED (current_password wrong), ERR_VALIDATION (weak new password)
```

---

### `upload_profile_picture`

```rust
#[tauri::command]
async fn upload_profile_picture(
    db: State<DbPool>,
    session_id: String,
    file_bytes: Vec<u8>,
    file_name: String,
) -> Result<String, String>  // Returns relative path to saved picture
// Saves to storage/profiles/{user_id}/avatar.{ext}
// Overwrites previous picture
```

---

## 5. Master Data Commands (Admin Only)

These commands are for Admin/IT Staff configuration. No-login viewer browsing must use the public browsing commands such as `list_public_categories`, not Admin master-data commands.

### Categories

#### `list_categories`

```rust
#[tauri::command]
async fn list_categories(
    db: State<DbPool>,
    session_id: String,
    include_inactive: Option<bool>,
) -> Result<Vec<CategoryItem>, String>
// Admin/IT Staff only. Returns categories sorted alphabetically; TRASH always last.
// System categories (is_system=1) are included; cannot be edited/deleted.

pub struct CategoryItem {
    pub category_id: i64,
    pub category_name: String,
    pub description: Option<String>,
    pub color_code: String,
    pub icon: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub document_count: i64,  // count of non-trashed documents
}
```

---

#### `create_category`

```rust
#[tauri::command]
async fn create_category(
    db: State<DbPool>,
    session_id: String,
    category_name: String,
    description: Option<String>,
    color_code: String,
    icon: Option<String>,
) -> Result<i64, String>
// Errors: ERR_DUPLICATE, ERR_VALIDATION (invalid hex color)
```

---

#### `update_category`

```rust
#[tauri::command]
async fn update_category(
    db: State<DbPool>,
    session_id: String,
    category_id: i64,
    category_name: String,
    description: Option<String>,
    color_code: String,
    icon: Option<String>,
    is_active: bool,
) -> Result<(), String>
// Errors: ERR_SYSTEM_RECORD (cannot edit TRASH or any is_system=1 category)
```

---

### Folders

#### `list_folders`

```rust
#[tauri::command]
async fn list_folders(
    db: State<DbPool>,
    session_id: String,
    category_id: Option<i64>,
    include_inactive: Option<bool>,
) -> Result<Vec<FolderItem>, String>

pub struct FolderItem {
    pub folder_id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub folder_name: String,
    pub description: Option<String>,
    pub folder_color: String,
    pub is_active: bool,
    pub document_count: i64,
}
```

---

#### `create_folder`

```rust
#[tauri::command]
async fn create_folder(
    db: State<DbPool>,
    session_id: String,
    category_id: i64,
    folder_name: String,
    description: Option<String>,
    folder_color: String,
) -> Result<i64, String>
// Errors: ERR_SYSTEM_RECORD (cannot create folder under TRASH), ERR_DUPLICATE
```

---

#### `update_folder`

```rust
#[tauri::command]
async fn update_folder(
    db: State<DbPool>,
    session_id: String,
    folder_id: i64,
    folder_name: String,
    description: Option<String>,
    folder_color: String,
    is_active: bool,
) -> Result<(), String>
```

---

### Offices

#### `list_offices`

```rust
#[tauri::command]
async fn list_offices(
    db: State<DbPool>,
    session_id: String,
    include_inactive: Option<bool>,
) -> Result<Vec<OfficeItem>, String>
```

#### `create_office` / `update_office`
Same pattern as category create/update. Omitted for brevity.

---

## 5.5 Public Browsing Commands (Staff/Head Viewer, No Login)

### `list_public_categories`
Returns active, non-system categories visible to the no-login Staff/Head Viewer. TRASH is never returned. Counts include only documents where `is_hidden = 0` and `is_trashed = 0`.

```rust
#[tauri::command]
async fn list_public_categories(
    db: State<DbPool>,
) -> Result<Vec<PublicCategoryItem>, String>

pub struct PublicCategoryItem {
    pub category_id: i64,
    pub category_name: String,
    pub description: Option<String>,
    pub color_code: String,
    pub icon: Option<String>,
    pub public_document_count: i64,
}
```

### `list_public_folders`
Returns active folders under one public category. Counts include only public, non-hidden, non-trashed documents.

```rust
#[tauri::command]
async fn list_public_folders(
    db: State<DbPool>,
    category_id: i64,
) -> Result<Vec<PublicFolderItem>, String>
```

## 6. Document Commands (Secretary + Public Viewer Where Noted)

### `list_documents`

```rust
#[tauri::command]
async fn list_documents(
    db: State<DbPool>,
    session_id: Option<String>,  // None = no-login Staff/Head Viewer
    category_id: Option<i64>,
    folder_id: Option<i64>,
    search: Option<String>,
    status_filter: Option<String>,
    sort_by: Option<String>,     // "date_received" | "document_name" | "date_added"
    sort_dir: Option<String>,    // "asc" | "desc"
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<PaginatedDocuments, String>

pub struct PaginatedDocuments {
    pub items: Vec<DocumentListItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

pub struct DocumentListItem {
    pub document_id: i64,
    pub document_name: String,
    pub category_id: i64,
    pub category_name: String,
    pub folder_id: Option<i64>,
    pub folder_name: Option<String>,
    pub sender_name: String,
    pub sender_office: Option<String>,
    pub date_received: String,
    pub document_status: String,
    pub is_hidden: bool,
    pub is_trashed: bool,
    pub trashed_at: Option<String>,
    pub days_until_purge: Option<i64>,  // None if not trashed or auto-purge disabled;
                                        // 0 = purges today; negative = overdue for purge
    pub attachment_count: i64,
    pub date_added: String,
}
// No-login Staff/Head Viewer filtering applied in backend:
//   - is_trashed = 0
//   - is_hidden = 0
//   - category is_system = 0 (no TRASH)
// Secretary sees all including hidden and trashed
// days_until_purge: calculated as (settings.trash_auto_purge_days - days_since_trashed)
//   Only populated when is_trashed=1 AND trash_auto_purge_days > 0 (not disabled)
```

---

### `get_document`

```rust
#[tauri::command]
async fn get_document(
    db: State<DbPool>,
    session_id: Option<String>,
    document_id: i64,
) -> Result<DocumentDetail, String>

pub struct DocumentDetail {
    pub document_id: i64,
    pub document_name: String,
    pub category_id: i64,
    pub category_name: String,
    pub folder_id: Option<i64>,
    pub folder_name: Option<String>,
    pub sender_name: String,
    pub sender_office_id: Option<i64>,
    pub sender_office: Option<String>,
    pub receiver_name: Option<String>,
    pub date_received: String,
    pub remarks: Option<String>,
    pub document_status: String,
    pub status_other: Option<String>,
    pub is_hidden: bool,
    pub is_trashed: bool,
    pub added_by_name: String,
    pub date_added: String,
    pub attachments: Vec<AttachmentItem>,
}

pub struct AttachmentItem {
    pub attachment_id: i64,
    pub file_name: String,
    pub file_type: String,
    pub file_size_bytes: i64,
    pub source: String,
    pub sort_order: i64,
}
```

---

### `create_document`

```rust
#[tauri::command]
async fn create_document(
    db: State<DbPool>,
    session_id: String,
    category_id: i64,
    folder_id: Option<i64>,
    document_name: String,
    sender_name: String,
    sender_office_id: Option<i64>,
    receiver_name: Option<String>,
    date_received: String,          // YYYY-MM-DD
    remarks: Option<String>,
    document_status: String,
    status_other: Option<String>,
    is_hidden: bool,
    uploaded_files: Vec<UploadedFileRef>, // path-based refs from file picker; max 1 GB each
    scan_intake_ids: Vec<i64>,           // intake IDs to claim
) -> Result<i64, String>  // Returns new document_id

pub struct UploadedFileRef {
    pub source_path: String,      // Absolute path returned by the OS file picker; never stored in DB
    pub file_name: String,
    pub file_type: String,
    pub file_size_bytes: i64,     // Must be <= 1 GB
}

// Implementation rule: the frontend must not send large file bytes through IPC.
// Rust copies from source_path into storage/documents/{id}/uploaded/ after validating
// source existence, file extension/MIME, max size, and destination path safety.
// Validation:
//   - If document_status == "Confidential", backend sets is_hidden=true regardless of client default.
// Side effects:
//   1. Inserts document record
//   2. Saves uploaded files to storage/documents/{id}/uploaded/
//   3. Moves claimed scans from storage/intake/ to storage/documents/{id}/scans/
//   4. Inserts attachment records for both
//   5. Updates scan_intake.is_claimed for claimed scans
//   6. Writes INSERT to audit_log for document and SCAN for each claimed intake
```

---

### `update_document`

```rust
#[tauri::command]
async fn update_document(
    db: State<DbPool>,
    session_id: String,
    document_id: i64,
    document_name: String,
    sender_name: String,
    sender_office_id: Option<i64>,
    receiver_name: Option<String>,
    date_received: String,
    remarks: Option<String>,
    document_status: String,
    status_other: Option<String>,
) -> Result<(), String>
// Validation: if document_status == "Confidential", backend sets is_hidden=true.
// Side effects: UPDATE audit_log
```

---

### `set_document_hidden`
Toggle document visibility for no-login Staff/Head Viewer users.

```rust
#[tauri::command]
async fn set_document_hidden(
    db: State<DbPool>,
    session_id: String,
    document_id: i64,
    is_hidden: bool,
) -> Result<(), String>
// Side effects: writes HIDE or UNHIDE to audit_log
```

---

### `move_document`

```rust
#[tauri::command]
async fn move_document(
    db: State<DbPool>,
    session_id: String,
    document_id: i64,
    new_category_id: i64,
    new_folder_id: Option<i64>,
) -> Result<(), String>
// Validates: if new_folder_id is Some, it belongs to new_category_id; None means category root
// Validates: target is not TRASH (use trash_document for that)
// Errors: ERR_VALIDATION, ERR_NOT_FOUND
// Side effects: MOVE audit_log with previous/new location in description
```

---

### `trash_document`
Moves a document to TRASH.

```rust
#[tauri::command]
async fn trash_document(
    db: State<DbPool>,
    session_id: String,
    document_id: i64,
) -> Result<(), String>
// Sets is_trashed=1, trashed_at=now, category_id=TRASH_ID, folder_id=NULL
// Sets original_category_id and original_folder_id to current values
// Side effects: TRASH audit_log
```

---

### `restore_document`
Restores a document from TRASH to its original location. **Both Secretary and Admin** can restore.

```rust
#[tauri::command]
async fn restore_document(
    db: State<DbPool>,
    session_id: String,
    document_id: i64,
) -> Result<RestoreResult, String>

pub struct RestoreResult {
    pub document_id: i64,
    pub restored_to_category_id: i64,
    pub restored_to_folder_id: Option<i64>,  // None = category root (folder was deleted)
    pub folder_was_missing: bool,             // true = original folder gone; restored to root
}
// Restore logic (in order):
//   1. If original_folder_id folder is still active → restore to original_category + original_folder
//   2. If original_folder_id is inactive/deleted but original_category is still active
//      → restore to original_category with folder_id = NULL (category root)
//      → folder_was_missing = true (frontend shows info toast: "Restored to category root —
//        original folder no longer exists")
//   3. If original_category is also inactive → restore to category root anyway
//      (category still exists in DB; is_active=0 means it's hidden from new documents
//      but existing documents can still live there)
// Sets is_trashed=0, trashed_at=NULL; clears original_* fields
// Side effects: RESTORE_TRASH audit_log; description notes if folder fallback was used
```

---

### `purge_document`
Permanently deletes a trashed document and all its attachment files. **Admin only.**

```rust
#[tauri::command]
async fn purge_document(
    db: State<DbPool>,
    session_id: String,
    document_id: i64,
) -> Result<(), String>
// Requires Admin role — calls require_admin_role() after session validation
// Must be trashed (is_trashed=1) to purge; ERR_CONFLICT if not trashed
// Deletes: attachment files on disk, attachment records, document record
// Side effects: PURGE audit_log
```

---

### `empty_trash`
Purges all documents in TRASH permanently. **Admin only.**

```rust
#[tauri::command]
async fn empty_trash(
    db: State<DbPool>,
    session_id: String,
) -> Result<i64, String>  // Returns count of purged documents
// Requires Admin role — calls require_admin_role() after session validation
// Side effects: one PURGE audit_log entry with count: "Emptied trash: 12 documents permanently deleted"
```

---

## 7. Attachment Commands (Secretary)

### `add_attachment`
Adds a file to an existing document.

```rust
#[tauri::command]
async fn add_attachment(
    db: State<DbPool>,
    session_id: String,
    document_id: i64,
    uploaded_file: UploadedFileRef,
) -> Result<i64, String>  // Returns new attachment_id
// Copies from uploaded_file.source_path into managed storage; rejects > 1 GB.
```

---

### `claim_scan_as_attachment`
Claims an intake scan and attaches it to a document.

```rust
#[tauri::command]
async fn claim_scan_as_attachment(
    db: State<DbPool>,
    session_id: String,
    document_id: i64,
    intake_ids: Vec<i64>,
) -> Result<Vec<i64>, String>  // Returns new attachment_ids
```

---

### `remove_attachment`
Removes an attachment from a document. Deleted files are not recoverable.

```rust
#[tauri::command]
async fn remove_attachment(
    db: State<DbPool>,
    session_id: String,
    attachment_id: i64,
) -> Result<(), String>
// Deletes file from disk + removes attachment record
```

---

### `reorder_attachments`
Updates sort_order for a document's attachments.

```rust
#[tauri::command]
async fn reorder_attachments(
    db: State<DbPool>,
    session_id: String,
    document_id: i64,
    ordered_attachment_ids: Vec<i64>,  // Full ordered list; index = new sort_order
) -> Result<(), String>
```

---

### `serve_attachment`
Returns file bytes for preview or download.

```rust
#[tauri::command]
async fn serve_attachment(
    db: State<DbPool>,
    session_id: Option<String>,
    attachment_id: i64,
) -> Result<AttachmentPayload, String>

pub struct AttachmentPayload {
    pub file_name: String,
    pub file_type: String,
    pub file_bytes: Vec<u8>,
}
// Path traversal check: resolved path must be within storage base dir
```

---

## 8. Scan Intake Commands (Secretary)

### `list_intake_scans`

```rust
#[tauri::command]
async fn list_intake_scans(
    db: State<DbPool>,
    session_id: String,
) -> Result<Vec<IntakeScanItem>, String>
// Returns only is_claimed=0 AND is_deleted=0 scans, sorted by scanned_at descending

pub struct IntakeScanItem {
    pub intake_id: i64,
    pub file_name: String,
    pub file_size_bytes: i64,
    pub thumbnail_path: Option<String>,
    pub scanned_at: String,
}
```

---

### `import_scan_files`
Imports files from the scan intake folder (watching pattern for MVP).

```rust
#[tauri::command]
async fn import_scan_files(
    db: State<DbPool>,
    session_id: String,
    file_paths: Vec<String>,  // Absolute paths selected via file dialog
) -> Result<Vec<i64>, String> // Returns new intake_ids
// Copies files to storage/intake/, generates thumbnails, inserts scan_intake records
// Side effects: SCAN audit_log
```

---

### `delete_intake_scan`
Recoverably deletes an unclaimed scan from the intake staging area.

```rust
#[tauri::command]
async fn delete_intake_scan(
    db: State<DbPool>,
    session_id: String,
    intake_id: i64,
) -> Result<(), String>
// Must be unclaimed (is_claimed=0)
// Sets is_deleted=1 and deleted_at=now; file remains recoverable.
// Errors: ERR_CONFLICT (scan is claimed by a document)
```

---

### `list_deleted_intake_scans`
Lists recoverably deleted scans.

```rust
#[tauri::command]
async fn list_deleted_intake_scans(
    db: State<DbPool>,
    session_id: String,
) -> Result<Vec<IntakeScanItem>, String>
```

---

### `restore_deleted_intake_scan`
Restores a recoverably deleted scan back to normal intake.

```rust
#[tauri::command]
async fn restore_deleted_intake_scan(
    db: State<DbPool>,
    session_id: String,
    intake_id: i64,
) -> Result<(), String>
// Clears is_deleted and deleted_at.
```

---

### `purge_deleted_intake_scans`
Permanently purges deleted scans older than retention.

```rust
#[tauri::command]
async fn purge_deleted_intake_scans(
    db: State<DbPool>,
    session_id: String,
) -> Result<i64, String>
// Uses settings.deleted_scan_retention_days (default: 30) unless run manually by Admin/IT Staff.
```

---

### `return_scan_to_intake`
Un-claims a scan from a document and returns it to the intake pool.

```rust
#[tauri::command]
async fn return_scan_to_intake(
    db: State<DbPool>,
    session_id: String,
    attachment_id: i64,
    intake_id: i64,
) -> Result<(), String>
// Moves file back from storage/documents/{id}/scans/ to storage/intake/
// Deletes attachment record, resets scan_intake.is_claimed=0
```

---

## 9. PDF Export Commands

### `export_document_pdf`

```rust
#[tauri::command]
async fn export_document_pdf(
    db: State<DbPool>,
    session_id: Option<String>,
    document_id: i64,
) -> Result<Vec<u8>, String>
// Returns PDF bytes; frontend uses Tauri save dialog to write to disk
// PDF includes: UEP/OVPSPEE letterhead, document metadata, all visible attachments rendered as pages, page numbers, generation timestamp, and footer/certification text
```

---

### `export_audit_log_pdf`

```rust
#[tauri::command]
async fn export_audit_log_pdf(
    db: State<DbPool>,
    session_id: String,
    filters: AuditLogFilters,
) -> Result<Vec<u8>, String>

pub struct AuditLogFilters {
    pub search: Option<String>,
    pub action_filter: Option<String>,
    pub user_filter: Option<i64>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}
```

---

## 10. Audit Log Commands (Admin + Secretary Own Activity)

### `list_audit_logs`

```rust
#[tauri::command]
async fn list_audit_logs(
    db: State<DbPool>,
    session_id: String,
    filters: AuditLogFilters,
    page: i64,
    page_size: i64,        // Default: 50
) -> Result<PaginatedAuditLogs, String>
// Admin/IT Staff only. Shows all users and system actions.
```

---

### `list_my_audit_logs`
Secretary-facing activity history. Returns only rows where `audit_log.user_id` equals the validated session user.

```rust
#[tauri::command]
async fn list_my_audit_logs(
    db: State<DbPool>,
    session_id: String,
    filters: AuditLogFilters,
    page: i64,
    page_size: i64,
) -> Result<PaginatedAuditLogs, String>
// Secretary/Admin authenticated users may call this for their own activity only.
// The backend ignores any user_filter value and uses the session user_id.
```

---

### `get_retention_setting`

```rust
#[tauri::command]
async fn get_retention_setting(
    db: State<DbPool>,
    session_id: String,
) -> Result<i64, String>   // Returns months value; default 36 months
```

---

### `update_retention_setting`

```rust
#[tauri::command]
async fn update_retention_setting(
    db: State<DbPool>,
    session_id: String,
    months: i64,           // Allowed default policy range: 24–36 months unless client policy changes
) -> Result<(), String>
// Side effects: writes CLEANUP audit_log entry for policy change
```

---

### `run_audit_cleanup`

```rust
#[tauri::command]
async fn run_audit_cleanup(
    db: State<DbPool>,
    session_id: String,
) -> Result<i64, String>   // Returns count of deleted entries
```

---

## 11. Backup & Restore Commands (Admin)

**Pre-restore safety backup:** Before any restore replaces current data, the backend must automatically create a timestamped safety backup of the current DB and storage directory. Restore must validate manifest and checksums first.

**Backup destination warning:** Default local backups protect against user mistakes but not device loss or drive failure. The UI must recommend copying backups to external/network storage.


### `create_backup`

```rust
#[tauri::command]
async fn create_backup(
    db: State<DbPool>,
    session_id: String,
    destination_path: Option<String>,  // None = default local app-data backup folder; otherwise absolute folder chosen by Admin
) -> Result<String, String>   // Returns path to created backup folder
// Creates manifest.json with schema/app version and checksums. Side effects: BACKUP audit_log
```

---

### `export_backup_archive`
Creates a portable `.ovpspee-backup` ZIP archive.

```rust
#[tauri::command]
async fn export_backup_archive(
    db: State<DbPool>,
    session_id: String,
    destination_path: String,  // File path for the .ovpspee-backup file
) -> Result<(), String>
// Side effects: EXPORT audit_log
```

---

### `import_backup_archive`

```rust
#[tauri::command]
async fn import_backup_archive(
    db: State<DbPool>,
    session_id: String,
    archive_path: String,
) -> Result<BackupManifest, String>
// Validates archive before committing restore
// Side effects: IMPORT audit_log. Does not replace current data until restore_from_backup is called.
```

---

### `restore_from_backup`

```rust
#[tauri::command]
async fn restore_from_backup(
    db: State<DbPool>,
    session_id: String,
    backup_folder_path: String,
) -> Result<(), String>
// Destructive: validates backup, creates a pre-restore safety backup, replaces current DB + storage, then restarts app.
// Side effects: RESTORE audit_log
```

---

### `get_backup_settings` / `update_backup_settings`
Reads and writes scheduled backup configuration from the `settings` table. Omitted for brevity — follow the standard settings pattern.

---

## 12. Settings Commands (Admin)

### `get_settings`

```rust
#[tauri::command]
async fn get_settings(
    db: State<DbPool>,
    session_id: String,
    keys: Vec<String>,
) -> Result<HashMap<String, String>, String>
```

---

### `update_settings`

```rust
#[tauri::command]
async fn update_settings(
    db: State<DbPool>,
    session_id: String,
    updates: HashMap<String, String>,
) -> Result<(), String>
```

---

## 13. Dashboard Commands (Secretary)

### `get_dashboard_stats`

```rust
#[tauri::command]
async fn get_dashboard_stats(
    db: State<DbPool>,
    session_id: String,
) -> Result<DashboardStats, String>

pub struct DashboardStats {
    pub total_documents: i64,
    pub total_categories: i64,
    pub total_folders: i64,
    pub hidden_documents: i64,
    pub trashed_documents: i64,
    pub documents_this_month: i64,
    pub pending_intake_scans: i64,   // unclaimed scans in intake
    pub recent_documents: Vec<DocumentListItem>,  // last 10
}
```

---

*End of Backend API Documentation*
*Next: `04_Frontend_Component_Documentation.md`*

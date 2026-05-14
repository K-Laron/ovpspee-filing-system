# Testing Strategy Documentation
## OVPSPEE Filing & Tracking System — CDHP Document 05

---

## 1. Testing Philosophy

Testing is done at three levels: **Rust unit tests** (pure logic), **Tauri command integration tests** (round-trip IPC with in-memory SQLite), and **manual verification** (UI, installer, cross-machine scenarios). There are no E2E browser tests — the app is a desktop binary, not a web app.

**Rule:** Every vertical slice must have its tests passing before the next slice begins.

---

## 2. Rust Unit Tests

Located in the same file as the function under test, inside a `#[cfg(test)]` module. Run with `cargo test`.

### Auth & Session

| Test | What it verifies |
|---|---|
| `test_password_hash_and_verify` | Argon2id hashing produces a verifiable hash; wrong password fails |
| `test_first_run_check_empty_db` | Returns `true` on empty DB |
| `test_first_run_check_after_setup` | Returns `false` after admin is created |
| `test_login_success` | Returns a valid SessionPayload with correct role |
| `test_login_wrong_password` | Returns `ERR_UNAUTHORIZED` |
| `test_login_deactivated_user` | Returns `ERR_UNAUTHORIZED` for `is_active = 0` |
| `test_logout_clears_session` | Session record removed from DB |
| `test_validate_session_expired` | Returns `ERR_UNAUTHORIZED` for expired session |
| `test_validate_session_valid` | Returns SessionPayload for valid session |

### User Management

| Test | What it verifies |
|---|---|
| `test_create_user_success` | User created with hashed password, correct role |
| `test_create_user_duplicate_username` | Returns `ERR_DUPLICATE` |
| `test_create_user_weak_password` | Returns `ERR_VALIDATION` (< 8 chars, no number, no special) |
| `test_update_user_deactivate` | `is_active` toggles correctly |
| `test_admin_reset_password` | Password hash updated; old hash no longer valid |
| `test_non_admin_cannot_manage_users` | Returns `ERR_UNAUTHORIZED` for Secretary session |

### Category / Folder / Office

| Test | What it verifies |
|---|---|
| `test_create_category` | Category inserted with correct fields |
| `test_create_category_duplicate_name` | Returns `ERR_DUPLICATE` |
| `test_list_categories_alphabetical` | Admin result is sorted A-Z, TRASH is always last |
| `test_list_public_categories_excludes_trash` | Staff/Head Viewer public category list excludes TRASH, inactive categories, hidden/trashed document counts |
| `test_cannot_edit_system_category` | Returns `ERR_SYSTEM_RECORD` for TRASH |
| `test_create_folder_under_trash` | Returns `ERR_SYSTEM_RECORD` |
| `test_folder_name_unique_within_category` | Duplicate name in same category fails; same name in different category succeeds |

### Document Filing

| Test | What it verifies |
|---|---|
| `test_create_document_basic` | Document inserted with all fields; attachment records created |
| `test_confidential_auto_sets_hidden` | Creating/updating a Confidential document forces `is_hidden = 1` |
| `test_list_documents_viewer_filters` | Staff/Head Viewer users cannot see `is_hidden=1` or `is_trashed=1` documents |
| `test_list_documents_viewer_no_trash` | TRASH category excluded from guest results |
| `test_list_documents_secretary_sees_all` | Secretary sees hidden and trashed documents |
| `test_set_document_hidden_toggle` | `is_hidden` flips; audit log entry created |
| `test_move_document_valid` | `category_id` and `folder_id` updated correctly |
| `test_move_document_wrong_folder` | Returns `ERR_VALIDATION` (folder not in target category) |
| `test_move_document_to_trash_blocked` | Returns `ERR_VALIDATION` (must use trash_document) |
| `test_trash_document` | `is_trashed=1`, `original_*` fields populated, `category_id` = TRASH_ID |
| `test_restore_document` | Returns to original category/folder; `is_trashed=0`; `original_*` cleared |
| `test_restore_document_missing_folder_fallback` | If original folder is inactive/deleted, restore succeeds to original category root with `folder_was_missing = true` |
| `test_purge_document_files_deleted` | Files removed from filesystem; attachment records deleted; document deleted |
| `test_purge_non_trashed_document` | Returns `ERR_CONFLICT` |
| `test_empty_trash` | All trashed documents purged; files deleted |

### Attachment

| Test | What it verifies |
|---|---|
| `test_add_attachment_saves_file` | File copied by backend from source path to `storage/documents/{id}/uploaded/` |
| `test_attachment_size_limit_1gb` | File larger than 1 GB is rejected with `ERR_VALIDATION` |
| `test_remove_attachment_deletes_file` | File removed from disk; attachment record deleted |
| `test_reorder_attachments` | `sort_order` updated correctly for all IDs |
| `test_serve_attachment_path_traversal` | Path `../../etc/passwd` returns `ERR_UNAUTHORIZED` |

### Scan Intake

| Test | What it verifies |
|---|---|
| `test_import_scan_files` | Files copied to `storage/intake/`; records inserted |
| `test_list_intake_unclaimed_only` | Claimed scans not returned |
| `test_delete_intake_scan_soft_delete` | Unclaimed scan sets `is_deleted=1` and disappears from normal intake |
| `test_restore_deleted_intake_scan` | Deleted unclaimed scan can be restored to normal intake |
| `test_purge_deleted_intake_scan` | Retention cleanup permanently removes deleted scan file and record |
| `test_delete_claimed_intake_scan` | Returns `ERR_CONFLICT` |
| `test_claim_scan_as_attachment` | File moved to `storage/documents/{id}/scans/`; `is_claimed=1` |
| `test_return_scan_to_intake` | File moved back; `is_claimed=0`; attachment deleted |

### Audit Log

| Test | What it verifies |
|---|---|
| `test_audit_log_written_on_create` | INSERT entry present after create_document |
| `test_audit_log_written_on_move` | MOVE entry has correct description with old/new IDs |
| `test_audit_log_written_on_hide` | HIDE entry present |
| `test_audit_log_written_on_trash` | TRASH entry present |
| `test_audit_log_written_on_purge` | PURGE entry present |
| `test_list_my_audit_logs_scoped_to_user` | Secretary activity history only returns rows for their own `user_id` |
| `test_retention_cleanup_deletes_old` | Entries older than 36-month default retention threshold deleted |
| `test_retention_cleanup_keeps_recent` | Entries newer than threshold not deleted |

### Backup

| Test | What it verifies |
|---|---|
| `test_create_backup_produces_db_copy` | DB file appears in destination |
| `test_create_backup_copies_storage` | Storage directory contents replicated |
| `test_export_archive_valid_zip` | Output is a valid ZIP with manifest.json |
| `test_manifest_fields_complete` | manifest.json has all required fields |
| `test_import_archive_validates_manifest` | Missing or corrupt manifest returns error |
| `test_import_archive_wrong_schema_version` | Future schema version returns error |
| `test_import_archive_checksum_mismatch` | Blocks restore unless Admin explicitly confirms a recovery override |
| `test_restore_creates_pre_restore_safety_backup` | Restore creates safety backup before replacing current data |

---

## 3. Integration Tests

Integration tests run against a real Tauri command setup with an **in-memory SQLite database** (`:memory:`). They verify the full command pipeline from input validation → database mutation → audit log → return value.

Location: `src-tauri/tests/` (Rust integration test files, separate from unit tests).

```rust
// Example integration test pattern
#[tokio::test]
async fn test_create_and_retrieve_document() {
    let pool = setup_test_db().await;  // in-memory SQLite, migrations applied
    let session = create_test_secretary(&pool).await;

    let category_id = create_test_category(&pool, "BAC").await;
    let folder_id = create_test_folder(&pool, category_id, "PPMP 2025").await;

    let doc_id = create_document(
        &pool,
        &session.session_id,
        CreateDocumentParams {
            category_id,
            folder_id,
            document_name: "Test Document".into(),
            sender_name: "Jane Doe".into(),
            date_received: "2026-05-12".into(),
            document_status: "Filed".into(),
            is_hidden: false,
            uploaded_files: vec![],
            scan_intake_ids: vec![],
            ..Default::default()
        }
    ).await.expect("create_document failed");

    let doc = get_document(&pool, None, doc_id).await.expect("get_document failed");
    assert_eq!(doc.document_name, "Test Document");
    assert_eq!(doc.category_id, category_id);
    assert!(!doc.is_trashed);
    assert!(!doc.is_hidden);
}
```

### Integration Test Scenarios per Slice

| Slice | Scenario |
|---|---|
| 1 | First-run → login → validate session → logout |
| 2 | Create category → create folder → list categories (TRASH always last) |
| 3 | Create user → login as user → update profile → admin resets password |
| 4 | Create document with upload → guest list (hidden not shown) → secretary list (hidden shown) |
| 5 | Hide confidential document → verify viewer cannot see it → trash/restore → verify Admin-only purge |
| 6 | Move document → verify new location → attempt move to TRASH (blocked) |
| 7 | Export PDF with UEP/OVPSPEE letterhead returns non-empty bytes |
| 8 | Write audit entries → Admin filters all logs → Secretary sees own activity only → run 36-month retention cleanup |
| 9 | Import scan → soft-delete → restore → claim scan into document → verify file moved |
| 10 | Create local backup → verify archive structure → pre-restore safety backup → import archive → verify restored data |

---

## 4. Manual Verification Checklist

Run this checklist on a **clean Windows 10 VM** after every major slice and before release.

### Slice 1 — Auth
- [ ] Fresh install shows First-Run Setup screen
- [ ] Creating admin account works; redirects to Admin dashboard
- [ ] Login with wrong credentials shows generic error (not "user not found")
- [ ] Deactivated user cannot log in
- [ ] Logout redirects to no-login Staff/Head Viewer landing
- [ ] Session persists across app close/reopen (if session is stored)

### Slice 2 — Master Data
- [ ] Admin can create a category with color and icon
- [ ] TRASH appears last in the category list regardless of alphabetical position
- [ ] TRASH edit button is disabled / shows lock icon
- [ ] Admin cannot create a folder under TRASH
- [ ] Folder name can be duplicate across different categories but not within the same category

### Slice 3 — User Management
- [ ] Admin creates Secretary account; Secretary can log in
- [ ] Deactivating a user blocks their login immediately
- [ ] Admin password reset works; new password required at next login
- [ ] Profile picture uploads and displays correctly in sidebar
- [ ] All authenticated users can update their own profile fields

### Slice 4 — Document Filing
- [ ] Secretary can create a document with multiple file attachments
- [ ] Staff/Head Viewer sees only public, non-hidden, non-trashed documents
- [ ] Staff/Head Viewer does NOT see TRASH tab
- [ ] Secretary sees all documents including hidden ones (with EyeOff indicator)
- [ ] Search by document name works; results are debounced
- [ ] Sort by date received and document name works
- [ ] Filter by status works; active filter chips display correctly
- [ ] Large attachment (250 MB warning case and near-1GB limit case) copies without UI freeze or IPC memory spike
- [ ] Attachment page navigation ("PAGE 1 of N") works correctly for multi-page documents

### Slice 5 — Document Visibility & Trash
- [ ] Hide/Unhide toggles correctly; Staff/Head Viewer cannot see hidden/confidential documents
- [ ] Selecting Confidential auto-enables Hidden and shows warning helper text
- [ ] Trash document moves it to TRASH tab; original location no longer shows it
- [ ] Restore document returns it to original category/folder
- [ ] Restore with missing original folder falls back to category root and shows info toast
- [ ] Secretary cannot see Purge or Empty Trash actions
- [ ] Admin purge removes a trashed document permanently; file no longer on disk
- [ ] Admin Empty Trash removes all trashed documents; TRASH view is empty

### Slice 6 — Document Movement & Status
- [ ] Move document dialog shows current location correctly
- [ ] Folder dropdown updates when category is changed
- [ ] Moved document appears in new location; disappears from old location
- [ ] Move to TRASH through the Move dialog is blocked; user must use Trash action
- [ ] Status dropdown supports Filed, Archived, Confidential, and Other
- [ ] Other status requires `status_other`; non-Other status clears `status_other`

### Slice 7 — PDF Export
- [ ] Export PDF for a document prompts save dialog; saved PDF opens correctly
- [ ] PDF contains UEP/OVPSPEE letterhead, document metadata, attachment pages, page numbers, timestamp, and footer/certification text
- [ ] Paginated attachment preview renders correctly for a 100+ page document
- [ ] Lazy loading prevents UI freeze on large attachment sets

### Slice 8 — Audit Log
- [ ] Every significant action appears in the audit log
- [ ] Move action shows previous and new category/folder IDs in description
- [ ] Search/filter by date range works
- [ ] Retention cleanup deletes old entries and keeps recent ones using the 36-month default
- [ ] Audit log PDF export includes applied filters and correct entries
- [ ] Secretary My Activity page shows only current user actions

### Slice 9 — Scan Intake
- [ ] Secretary imports scan files via file picker; thumbnails appear in intake grid
- [ ] Unclaimed scans shown; claimed and deleted scans hidden from normal intake grid
- [ ] Delete intake scan moves it to Deleted Scans and hides it from normal grid
- [ ] Deleted scan can be restored before retention purge
- [ ] Add Document → From Scan Intake → picks scans → document saved with scans as attachments
- [ ] Picked scans no longer appear in intake grid after save
- [ ] Removing a scan during document creation (before save) returns it to intake grid

### Slice 10 — Backup & Restore
- [ ] Create Backup produces a timestamped folder with DB, storage directory, manifest, and checksums
- [ ] Default backup destination works using local app-data backup folder
- [ ] Export Backup produces a `.ovpspee-backup` file (valid ZIP)
- [ ] Import Backup validates the archive (corrupt file → clear error)
- [ ] Restore creates a pre-restore safety backup before replacing current data
- [ ] Restore from backup replaces all data and restarts the app
- [ ] After restore, all documents and attachments are accessible
- [ ] Cross-machine test: export from Machine A, fresh install Machine B, import → full restore

---

## 5. Performance Benchmarks

These are minimum acceptable performance targets. Test with a dataset of 10,000 documents.

| Operation | Target |
|---|---|
| Document search (10,000 records) | < 2 seconds |
| Attachment thumbnail render (50 MB file) | < 3 seconds per page |
| App cold start | < 5 seconds |
| Backup creation (1 GB storage) | < 60 seconds |
| Audit log list (50 entries, filtered) | < 1 second |

---

## 6. Regression Test Runs

Before any release tag, run the full unit test suite (`cargo test`), the integration test suite, and the manual verification checklist. Document any failures and resolve before tagging.

---

*End of Testing Strategy Documentation*
*Next: `06_Deployment_Documentation.md`*

# Plan 008: Split commands.rs boilerplate registry

> **Executor instructions**: Follow step by step. When done, update `plans/README.md`.
>
> **Drift check**: `git diff --stat 9b4b638..HEAD -- src-tauri/src/commands.rs src-tauri/src/lib.rs`

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: none (do before plan 009 for practice with same pattern)
- **Category**: tech-debt
- **Planned at**: commit `9b4b638`, 2026-06-27

## Why this matters

`commands.rs` is a 1309-line file containing 55+ `#[tauri::command]` functions. Every function is a trivial 1-3 line wrapper that extracts parameters from the Tauri IPC call and delegates to a domain function. Zero business logic. Splitting it into per-domain files makes the registry navigable and means adding a new command only touches its domain file plus the registration in `lib.rs`.

## Current state

- `src-tauri/src/commands.rs:1-1309` — all 55+ command wrappers in one file
- `src-tauri/src/lib.rs:45-133` — `invoke_handler` registration lists all 55+ command names
- Convention pattern (e.g., line 36-38):
  ```rust
  #[tauri::command]
  pub async fn first_run_check(db: State<'_, DbState>) -> CmdResult<bool> {
      auth::first_run_required(&db.pool).await
  }
  ```

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Rust build | `cargo build --manifest-path src-tauri/Cargo.toml` | exit 0 |
| Rust test | `cargo test --manifest-path src-tauri/Cargo.toml` | all pass |

## Scope

**In scope**:
- `src-tauri/src/commands.rs` — delete after splitting
- `src-tauri/src/commands/` — create directory
- Files to create in `src-tauri/src/commands/`:
  - `mod.rs` — re-exports all command modules
  - `auth_cmds.rs` — first_run_check, first_run_setup, login, logout, validate_session
  - `admin_cmds.rs` — list_users, create_user, update_user, admin_reset_password, list_categories, create_category, update_category, list_folders, create_folder, update_folder, list_offices, create_office, update_office, get_my_profile, update_my_profile, change_my_password, list_audit_logs, list_my_activity, list_audit_event_types, list_my_activity_event_types, get_audit_retention_settings, update_audit_retention_settings, get_backup_settings, update_backup_settings, create_backup, list_backup_history, export_backup_archive, validate_backup_archive, import_backup_archive, restore_from_backup, restore_from_backup_folder, run_scheduled_backup_check
  - `document_cmds.rs` — create_document, update_document, move_document, set_document_status, set_document_hidden, trash_document, restore_document, list_trash_documents, purge_document, empty_trash, list_documents, get_document, add_attachment, remove_attachment, reorder_attachments, get_attachment_file_path, get_attachment_preview_info, get_attachment_preview_page, export_document_pdf, list_document_offices
  - `public_cmds.rs` — list_public_categories, list_public_folders, list_public_documents, get_public_document, list_print_printers, print_document_pdf
  - `mobile_cmds.rs` — list_mobile_submissions, get_mobile_api_setup, create_mobile_device, list_mobile_devices, revoke_mobile_device, get_mobile_submission, get_mobile_submission_attachment_preview_page, approve_mobile_submission, reject_mobile_submission
  - `scan_cmds.rs` — import_scan_files, list_scan_intake, get_scan_intake, get_scan_intake_preview_page, update_scan_intake_notes, remove_scan_intake, file_scan_as_document, attach_scan_to_document, get_scanner_capabilities, scan_to_intake
  - `device_cmds.rs` — list_scanners, list_printers, get_default_printer, get_device_settings, update_device_settings

- `src-tauri/src/lib.rs` — update registration

**Out of scope**:
- Any business logic changes
- Any changes to domain modules (auth.rs, documents.rs, etc.)
- The `type CmdResult<T> = Result<T, AppError>;` alias (keep in each file or put in a shared place)

## Steps

### Step 1: Create src-tauri/src/commands/ directory and mod.rs

Create `src-tauri/src/commands/mod.rs`:

```rust
pub mod admin_cmds;
pub mod auth_cmds;
pub mod device_cmds;
pub mod document_cmds;
pub mod mobile_cmds;
pub mod public_cmds;
pub mod scan_cmds;
```

### Step 2: Extract auth_cmds.rs

Create `src-tauri/src/commands/auth_cmds.rs`. Move these from `commands.rs`:
- `first_run_check`
- `first_run_setup`
- `login`
- `logout`
- `validate_session`

Each function is identical to the original except the file path changes. Copy the `use` imports needed:

```rust
use tauri::State;
use crate::{auth, db::DbState, error::AppError};

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub async fn first_run_check(db: State<'_, DbState>) -> CmdResult<bool> {
    auth::first_run_required(&db.pool).await
}
// ... rest of functions
```

**Verify**: `cargo build` — should fail because commands.rs still has these functions (duplicates). That's expected — both files will compile together until step 6.

### Step 3: Extract remaining command modules

Follow the same pattern for each file listed in Scope. Each module file needs:
- The `use` imports from `crate::{...}` (domain modules, DbState, AppError)
- The `CmdResult` type alias
- The `#[tauri::command]` functions with full Tauri parameter patterns

Group by domain:
- `auth_cmds.rs` — 5 functions (auth-related commands)
- `admin_cmds.rs` — ~28 functions (admin-only: user mgmt, category/folder/office CRUD, audit, backup)
- `document_cmds.rs` — ~18 functions (document and attachment commands)
- `public_cmds.rs` — 6 functions (no-login and public endpoints)
- `mobile_cmds.rs` — 8 functions (mobile submission and device mgmt)
- `scan_cmds.rs` — 10 functions (scan intake, scanner devices)
- `device_cmds.rs` — 5 functions (printer/device settings)

### Step 4: Update lib.rs

In `src-tauri/src/lib.rs`, change:
```rust
pub mod commands;
// (this already exists)
```

And update the `invoke_handler` registration. Instead of listing each function:

```rust
.invoke_handler(tauri::generate_handler![
    commands::auth_cmds::first_run_check,
    commands::auth_cmds::first_run_setup,
    commands::auth_cmds::login,
    commands::auth_cmds::logout,
    commands::auth_cmds::validate_session,
    commands::admin_cmds::list_categories,
    commands::admin_cmds::create_category,
    // ... all 55+ functions fully qualified
])
```

The `generate_handler!` macro needs the full path to each function. You can also use a glob if all functions in a module are registered. For clarity, list them grouped by module with comments.

### Step 5: Delete src-tauri/src/commands.rs

After all functions are moved to their respective module files, delete the old `src-tauri/src/commands.rs`.

**Verify**: `cargo build` — compiles without errors (no duplicate functions). `cargo test` — all pass.

## Test plan

- `cargo test` must pass with no changes to tests. Since the command functions are identical (just moved), no behavior changes.
- Check that `generate_handler!` references match all exported functions.

## Done criteria

- [ ] `cargo build` exits 0
- [ ] `cargo test` all pass
- [ ] `ls src-tauri/src/commands/` shows mod.rs + 7 module files
- [ ] `src-tauri/src/commands.rs` no longer exists
- [ ] `plans/README.md` status row updated

## STOP conditions

- If `generate_handler!` doesn't accept the qualified paths, check the macro docs. It takes a list of paths — `module::function` syntax is correct for Tauri v2.
- If a function in the original `commands.rs` references a type not imported in its new home, add the missing import. Common missing types: `storage_root`, `backup_runtime` helper functions (defined at the bottom of `commands.rs`).
- The `storage_root` and `backup_runtime` helper functions (lines 1299-1309) must be moved somewhere accessible by all command modules. Options:
  - Move to a shared module (e.g., `commands/mod.rs` or `util.rs`)
  - Keep them in `commands.rs` during transition
  - Move to `crate::util` or create `crate::app_helpers`

## Maintenance notes

- New commands should be added to the appropriate domain module, not to a single `commands.rs`.
- The `generate_handler!` list in `lib.rs` still needs updating for new commands — that's the one unavoidable touch point.
- Consider adding a macro or build script to auto-generate the handler list from module exports in the future.

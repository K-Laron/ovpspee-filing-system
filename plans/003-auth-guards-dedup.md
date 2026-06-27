# Plan 003: Auth guards consolidation, preview dedup, location validation dedup

> **Executor instructions**: Follow step by step. Run every verification before
> moving to next step. When done, update `plans/README.md`.
>
> **Drift check**: `git diff --stat 9b4b638..HEAD -- src-tauri/src/`

## Status

- **Priority**: P1 (prerequisite for plan 010)
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: tech-debt
- **Planned at**: commit `9b4b638`, 2026-06-27

## Why this matters

Three small-but-important structural consolidations that eliminate copy-paste patterns. Fix 1 replaces 6 scattered role-check functions with 2 generic ones in auth.rs. Fix 2 extracts shared preview-info logic used by documents, scan intake, and mobile submissions. Fix 3 unifies category/folder location validation between documents and mobile submissions (one uses compile-time-checked queries, the other runtime).

## Current state

### Fix 1 — 6 auth guard functions, functionally overlapping

Six role-check functions across 5 files:
- `auth.rs:262` — `require_admin_role(role)`
- `documents.rs:929-939` — `require_document_editor(pool, session_id)` — checks Secretary
- `documents.rs:947-957` — `require_trash_viewer(pool, session_id)` — checks Secretary or Admin
- `scan_intake.rs:637-647` — `require_scan_user(pool, session_id)` — identical to `require_document_editor`
- `mobile_submissions.rs:350-360` — `require_secretary(pool, session_id)` — identical to `require_scan_user`
- `devices/mod.rs:365-371` — `require_device_reader(pool, session_id)` — identical to `require_trash_viewer`

### Fix 2 — Preview info logic duplicated thrice

Three near-identical functions:
- `documents.rs:1215-1255` — `preview_info_from_row`
- `scan_intake.rs:561-602` — `scan_preview_info`
- `mobile_submissions.rs:675-711` — `mobile_preview_info`

All check `file_exists`, determine `preview_kind` (Pdf/Image/Text/Unsupported), count PDF pages or Image=1, build a `message` string, and construct a `*PreviewInfo` struct.

### Fix 3 — Category/folder location validation duplicated

- `documents.rs:983-1015` — `validate_document_location`: uses `sqlx::query!` macros (compile-time checked)
- `mobile_submissions.rs:439-471` — `validate_mobile_location`: same logic but uses raw `sqlx::query` + `Row::try_get` (runtime checked)

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `cargo build --manifest-path src-tauri/Cargo.toml` | exit 0 |
| Test | `cargo test --manifest-path src-tauri/Cargo.toml` | all pass |
| TS typecheck | `pnpm tsc --noEmit` | exit 0 |
| TS test | `pnpm test` | all pass |

## Scope

**In scope**:
- `src-tauri/src/auth.rs`
- `src-tauri/src/documents.rs`
- `src-tauri/src/scan_intake.rs`
- `src-tauri/src/mobile_submissions.rs`
- `src-tauri/src/devices/mod.rs` (require_device_reader)
- Possibly a new `src-tauri/src/preview.rs` for shared preview logic

**Out of scope**:
- No behavior changes — pure mechanical extraction
- No frontend changes

## Steps

### Step 1: Consolidate auth guard functions

In `src-tauri/src/auth.rs`, add two generic functions:

```rust
/// Requires the session to have one of the specified roles.
pub async fn require_role(pool: &DbPool, session_id: &str, allowed_roles: &[&str]) -> AppResult<ValidSession> {
    let session = require_session(pool, session_id).await?;
    if allowed_roles.iter().any(|r| *r == session.role) {
        Ok(session)
    } else {
        Err(AppError::Unauthorized)
    }
}

/// Requires the session to have the Admin role.
pub async fn require_admin(pool: &DbPool, session_id: &str) -> AppResult<ValidSession> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    Ok(session)
}
```

Then replace each scattered guard with the generic:
- `require_document_editor` → `require_role(pool, session_id, &["Secretary"]).await`
- `require_trash_viewer` → `require_role(pool, session_id, &["Secretary", "Admin"]).await`
- `require_scan_user` → `require_role(pool, session_id, &["Secretary"]).await`
- `require_secretary` → `require_role(pool, session_id, &["Secretary"]).await`
- `require_device_reader` → `require_role(pool, session_id, &["Secretary", "Admin"]).await`

Remove the old specialized functions after replacing all call sites.

**Verify**: `cargo build` compiles. `cargo test` all pass. Grep for deleted function names returns 0 matches.

### Step 2: Extract shared preview logic into preview.rs

Create `src-tauri/src/preview.rs`. The shared logic is:

```rust
pub(crate) fn build_preview_info(
    attachment_id: i64,
    document_id: i64,
    original_file_name: String,
    mime_type: String,
    file_size_bytes: i64,
    path: &Path,
    kind_override: Option<&str>,  // for scan intake TIFF special case
) -> AttachmentPreviewInfo { ... }
```

The key shared code from `preview_info_from_row` in `documents.rs:1215-1255`. Extract `preview_kind`, `estimate_pdf_page_count`, `read_text_preview`, and `extension_from_name` into `preview.rs` as `pub(crate)`.

Make the three callers (`documents.rs`, `scan_intake.rs`, `mobile_submissions.rs`) import from `preview.rs` instead.

Add `pub mod preview;` to `src-tauri/src/lib.rs`.

**Verify**: `cargo build` compiles. `cargo test` all pass. All three modules still produce correct preview info.

### Step 3: Consolidate location validation

Make `validate_document_location` in `documents.rs:983` `pub(crate)` instead of private. Update its signature to be accessible from `mobile_submissions.rs`.

In `mobile_submissions.rs:439`, replace `validate_mobile_location` calls with `crate::documents::validate_document_location(...)`.

Remove the now-unused `validate_mobile_location` function from `mobile_submissions.rs`.

**Verify**: `cargo build` compiles. `cargo test` all pass. `grep -rn "validate_mobile_location" src-tauri/src/` returns 0.

## Test plan

- Existing tests should all pass unchanged. These are pure mechanical refactors — no behavior changes.
- Grep deleted function names after each step to confirm no stale references.

## Done criteria

- [ ] `cargo build` exits 0
- [ ] `cargo test` all pass
- [ ] `pnpm tsc --noEmit` exits 0
- [ ] `pnpm test` all pass
- [ ] `grep -rn "require_document_editor\|require_trash_viewer\|require_scan_user\|require_secretary\|require_device_reader" src-tauri/src/` returns 0
- [ ] `grep -rn "validate_mobile_location" src-tauri/src/` returns 0
- [ ] `plans/README.md` status row updated

## STOP conditions

- If any call site of the old guard functions is missed, the build will fail — that's expected and guides the fix.
- If preview info behavior differs between the three original functions (e.g., the scan intake version handles TIFF differently), document the difference and don't break it.

## Maintenance notes

- After this plan, adding a new role or changing permissions means updating only `require_role` calls and the `require_admin_role` helper — not 6 scattered functions.
- The preview module should be the single source of truth for preview-kind determination. Any new file type support (e.g., SVG) goes in one place.

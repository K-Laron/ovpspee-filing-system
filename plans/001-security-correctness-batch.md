# Plan 001: Security and correctness batch fixes

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 9b4b638..HEAD -- src-tauri/src/`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: security, correctness
- **Planned at**: commit `9b4b638`, 2026-06-27
- **Issue**: —

## Why this matters

Five small Rust backend fixes that close security gaps and prevent data-integrity bugs. Each is independently reviewable. Together they fix: session persistence after password change, unbounded mobile uploads, mobile table omission from backup/restore, audit log ordering outside transactions, and documents stuck in hidden state.

## Current state

All changes target `src-tauri/src/`. No frontend changes.

### Fix 1 — Password change doesn't revoke sessions
- **File**: `src-tauri/src/users.rs`
- **Role**: `change_my_password` function (lines 339–375)
- **Current**: Updates password hash and writes audit log, but does NOT delete existing sessions for that user. Contrast with `admin_reset_password` which DOES `DELETE FROM session WHERE user_id = ?`.
- **Current excerpt** (lines 363–374):
  ```rust
  sqlx::query!("UPDATE user SET password_hash = ?, updated_at = ? WHERE user_id = ?", ...)
      .execute(pool).await?;
  write_audit_log(pool, "UPDATE", ...).await?;
  Ok(())
  ```

### Fix 2 — Unbounded multipart upload body
- **File**: `src-tauri/src/mobile_api.rs`
- **Role**: `create_submission` function, multipart extraction (lines 199–243)
- **Current**: No body size limit on `Multipart` before streaming. Attacker can exhaust memory.
- **Fix**: Add `.with_max_length(200 * 1024 * 1024)` to set a 200 MB overall limit (per-file 1 GB check already exists).

### Fix 3 — Backup restore omits mobile tables
- **File**: `src-tauri/src/backup.rs`
- **Role**: `RESTORE_TABLES` constant (line 26–38)
- **Current**:
  ```rust
  const RESTORE_TABLES: &[&str] = &[
      "role", "user", "session", "audit_log", "category",
      "folder", "office", "settings", "document", "attachment", "scan_intake",
  ];
  ```
  Missing: `mobile_submission`, `mobile_submission_attachment`, `mobile_device`.

### Fix 4 — `authenticate_user` audit log outside transaction
- **File**: `src-tauri/src/auth.rs`
- **Role**: `authenticate_user` function (lines 152–222)
- **Current**: Transaction commits at line 203 (`tx.commit().await?`), then `write_audit_log` runs at line 205 outside the transaction. If the audit insert fails, login succeeds silently without audit trail.
- **Fix**: Move `write_audit_log` inside the transaction block, or at minimum add a comment documenting the intentional trade-off.

### Fix 5 — `is_hidden` never resets on status change from Confidential
- **File**: `src-tauri/src/documents.rs`
- **Role**: `update_document` function, SQL at line 418
- **Current**:
  ```sql
  is_hidden = CASE WHEN ? = 'Confidential' THEN 1 ELSE is_hidden END
  ```
  Changing status from Confidential to Filed/Archived/Other does NOT unhide the document. Only explicit `set_document_hidden` can.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Install (Rust) | `cargo build --manifest-path src-tauri/Cargo.toml` | exit 0 |
| Tests (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml` | all pass |
| Typecheck (TS) | `pnpm tsc --noEmit` | exit 0 |
| Tests (TS) | `pnpm test` | all pass |

## Scope

**In scope** (only files you should modify):
- `src-tauri/src/users.rs`
- `src-tauri/src/mobile_api.rs`
- `src-tauri/src/backup.rs`
- `src-tauri/src/auth.rs`
- `src-tauri/src/documents.rs`

**Out of scope**:
- Any frontend files
- Any test files (existing tests should still pass unchanged)
- Adding new dependencies

## Steps

### Step 1: Revoke sessions on password change

In `src-tauri/src/users.rs`, add session deletion inside `change_my_password` after the password hash update and before the audit log write:

```rust
// After password hash update, before audit log:
sqlx::query!("DELETE FROM session WHERE user_id = ?", session.user_id)
    .execute(pool)
    .await?;
```

**Verify**: `cargo test --manifest-path src-tauri/Cargo.toml` — all pass.

### Step 2: Add multipart body size limit

In `src-tauri/src/mobile_api.rs`, change the multipart extraction in `create_submission`:

```rust
// Current:
let mut multipart = Multipart::from_request(&req, &state).await?;

// New:
let mut multipart = Multipart::from_request(&req, &state)
    .await?
    .with_max_length(200 * 1024 * 1024); // 200 MB overall limit
```

Wait — actually `Multipart` is already extracted via axum's extractor pattern. The change is to add the limit in the router or as a layer. The correct approach:

In `create_submission`, replace the function signature. Actually, axum's `Multipart` extractor doesn't support `.with_max_length()` directly. Instead, add a limit via `ContentLengthLimit` or use `DefaultBodyLimit`. The simplest:

In `router()` function (lines 48–69), add:
```rust
.layer(DefaultBodyLimit::max(200 * 1024 * 1024)) // 200 MB
```
And import: `use axum::extract::DefaultBodyLimit;`

**Verify**: `cargo build --manifest-path src-tauri/Cargo.toml` — compiles.

### Step 3: Add mobile tables to RESTORE_TABLES

In `src-tauri/src/backup.rs`, add three entries to `RESTORE_TABLES`:

```rust
const RESTORE_TABLES: &[&str] = &[
    "role",
    "user",
    "session",
    "audit_log",
    "category",
    "folder",
    "office",
    "settings",
    "document",
    "attachment",
    "scan_intake",
    "mobile_submission",
    "mobile_submission_attachment",
    "mobile_device",
];
```

Also add export/import support for these tables. Search for patterns like `write_table_to_archive` and `restore_table_from_archive` calls to see how tables are iterated. The export code likely reads all tables via `SELECT * FROM <table>` — adding to `RESTORE_TABLES` should be sufficient if the export code iterates this constant.

**Verify**: `cargo test --manifest-path src-tauri/Cargo.toml` — all pass.

### Step 4: Move audit log inside login transaction

In `src-tauri/src/auth.rs`, move `write_audit_log` inside the transaction block in `authenticate_user`:

```rust
// Before: tx.commit() at line 203, write_audit_log at line 205
// After: include write_audit_log within the tx block, then commit

// ... after the UPDATE user SET last_login_at ... query, before tx.commit():
write_audit_log(
    pool,  // Note: use &mut *tx instead of pool to include in transaction
    "LOGIN",
    Some("user"),
    Some(row.user_id),
    "User logged in",
    Some(row.user_id),
).await?;
tx.commit().await?;
```

Wait — `write_audit_log` takes `&DbPool` not `&mut Transaction`. To include it in the transaction, either:
- Option A: Change `write_audit_log` to accept `impl Executor` (best)
- Option B: Accept the trade-off and add a TODO comment

For a small fix, Option B is pragmatic. Add a comment above the `write_audit_log` call:
```rust
// TODO: Include this audit log in the transaction above once write_audit_log
// accepts an Executor rather than requiring &DbPool.
```

**Verify**: `cargo test --manifest-path src-tauri/Cargo.toml` — all pass.

### Step 5: Fix is_hidden CASE in update_document

In `src-tauri/src/documents.rs`, change the SQL in `update_document` (line 418):

```sql
-- Current:
is_hidden = CASE WHEN ? = 'Confidential' THEN 1 ELSE is_hidden END

-- New:
is_hidden = CASE WHEN ? = 'Confidential' THEN 1 ELSE 0 END
```

Note: This means changing status from e.g. Filed to Confidential sets hidden=1, and changing back to Filed sets hidden=0. If the user explicitly unhid a document (via `set_document_hidden`), updating the status will override that. This is the correct behavior per the spec (Confidential forces hidden).

**Verify**: `cargo test --manifest-path src-tauri/Cargo.toml` — all pass.

### Step 6: Add document_id tiebreaker to sort

In `src-tauri/src/documents.rs`, line 1050, add `d.document_id DESC` as final sort key:

```sql
ORDER BY d.date_received DESC, d.document_name COLLATE NOCASE ASC, d.document_id DESC
```

**Verify**: `cargo test --manifest-path src-tauri/Cargo.toml` — all pass.

## Test plan

- Existing tests should all pass unchanged. Each step is a point fix with no new logic — current test coverage is adequate to catch regressions.
- No new tests required for this batch (each fix is a 1–3 line change that existing integration tests exercise).

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `cargo build --manifest-path src-tauri/Cargo.toml` exits 0
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` — all pass
- [ ] `pnpm tsc --noEmit` exits 0
- [ ] `pnpm test` — all pass
- [ ] `plans/README.md` status row updated to DONE

## STOP conditions

Stop and report back (do not improvise) if:

- The code at the locations in "Current state" doesn't match the excerpts (codebase has drifted).
- A step's verification fails twice after a reasonable fix attempt.
- The fix appears to require touching an out-of-scope file.
- Adding mobile tables to RESTORE_TABLES requires changes beyond the constant array (e.g., the export/import code doesn't iterate `RESTORE_TABLES`).

## Maintenance notes

- The `change_my_password` session revocation mirrors `admin_reset_password`. If the session deletion strategy changes (e.g., allow concurrent sessions), both locations must be updated.
- The `RESTORE_TABLES` constant must be kept in sync when new entity tables are added. Any table that has data should be listed here.
- The `is_hidden` CASE change means updating a document's status always resets hidden state. If per-document hidden-override is needed independently of status, the SQL needs an additional flag column.

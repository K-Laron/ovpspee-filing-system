# Plan 005: Status filter in SQL + scan intake polling fix

> **Executor instructions**: Follow step by step. When done, update `plans/README.md`.
>
> **Drift check**: `git diff --stat 9b4b638..HEAD -- src-tauri/src/documents.rs src/pages/secretary/Documents.tsx src/pages/secretary/ScanIntake.tsx`

## Status

- **Priority**: P2
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: perf
- **Planned at**: commit `9b4b638`, 2026-06-27

## Why this matters

Two small performance fixes: (1) status filter is currently applied client-side after fetching ALL documents from the backend, wasting bandwidth and serialization at scale; (2) Scan Intake page polls every 10 seconds unconditionally, causing 6 unnecessary IPC calls per minute when idle.

## Current state

### Fix 1 — Status filter in client, not SQL
- `src/pages/secretary/Documents.tsx:93-94`:
  ```typescript
  if (view === 'active' && statusFilter) {
    rows = rows.filter((row) => row.status === statusFilter);
  }
  ```
- `src-tauri/src/documents.rs:97-105` — `DocumentListFilter` has `search`, `category_id`, `folder_id`, `office_id`, `date_from`, `date_to` — no `status` field.
- `src-tauri/src/documents.rs:1017-1091` — `fetch_documents` builds SQL dynamically with bind parameters for all filter fields but not status.

### Fix 2 — 10s poll on scan intake
- `src/pages/secretary/ScanIntake.tsx:227-229`:
  ```typescript
  setInterval(() => {
    if (!document.hidden) void loadIntake();
  }, 10000);
  ```

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Rust build | `cargo build --manifest-path src-tauri/Cargo.toml` | exit 0 |
| Rust test | `cargo test --manifest-path src-tauri/Cargo.toml` | all pass |
| TS typecheck | `pnpm tsc --noEmit` | exit 0 |
| TS test | `pnpm test` | all pass |
| Verify | `pnpm verify` | exit 0 |

## Scope

**In scope**:
- `src-tauri/src/documents.rs`
- `src/pages/secretary/Documents.tsx`
- `src/pages/secretary/ScanIntake.tsx`

**Out of scope**:
- Other filter fields (date_from, date_to, etc.) — leave as-is
- Pagination (handled in plan 006)
- Public document list

## Steps

### Step 1: Add status field to DocumentListFilter

In `src-tauri/src/documents.rs`, add to the struct:

```rust
#[derive(Debug, Clone, Default)]
pub struct DocumentListFilter {
    pub search: Option<String>,
    pub category_id: Option<i64>,
    pub folder_id: Option<i64>,
    pub office_id: Option<i64>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub status: Option<String>,  // ← add this
}
```

### Step 2: Add status bind param to fetch_documents SQL

In `src-tauri/src/documents.rs`, in the `fetch_documents` function (line 1028), add to the dynamic SQL query:

```sql
-- Add to WHERE clause:
AND (? IS NULL OR d.status = ?)
```

Add bind calls:

```rust
.bind(filter.status.as_deref())
.bind(filter.status.as_deref())
```

Add `status` to the `DocumentItem` struct construction (already there — just verify).

### Step 3: Thread status through list_documents and callers

The `list_documents` function already accepts a `DocumentListFilter` (`documents.rs:542`). The Tauri command `list_documents` in `commands.rs:516` extracts filter fields individually — add `status: Option<String>` parameter there and pass it through.

In `src-tauri/src/commands.rs`, the `list_documents` command function (around line 516):

```rust
pub async fn list_documents(
    db: State<'_, DbState>,
    session_id: String,
    search: Option<String>,
    category_id: Option<i64>,
    folder_id: Option<i64>,
    office_id: Option<i64>,
    date_from: Option<String>,
    date_to: Option<String>,
    status: Option<String>,  // ← add
) -> CmdResult<Vec<DocumentItem>> {
    documents::list_documents(
        &db.pool, &session_id,
        DocumentListFilter {
            search, category_id, folder_id, office_id,
            date_from, date_to, status,  // ← add
        },
    ).await
}
```

### Step 4: Remove client-side status filter

In `src/pages/secretary/Documents.tsx`, remove lines 93-94 (the status filter block). The `invoke` call already passes `status: statusFilter || null` — if it doesn't, add the parameter:

```typescript
let rows = view === 'trash'
  ? await invoke<DocumentItem[]>('list_trash_documents', { sessionId })
  : await invoke<DocumentItem[]>('list_documents', {
      sessionId,
      search: search || null,
      categoryId: categoryId ? Number(categoryId) : null,
      folderId: folderId ? Number(folderId) : null,
      officeId: officeId ? Number(officeId) : null,
      dateFrom: dateFrom || null,
      dateTo: dateTo || null,
      status: statusFilter || null,  // ← add
    });
```

**Verify**: `cargo build` + `pnpm tsc --noEmit` compile. `pnpm test` passes.

### Step 5: Reduce scan intake polling interval

In `src/pages/secretary/ScanIntake.tsx`, change the interval from 10s to 30s:

```typescript
setInterval(() => {
  if (!document.hidden) void loadIntake();
}, 30000);
```

Optionally add a check that the intake data actually changed (e.g., compare item count or last-updated timestamp) before updating state, but the 30s interval alone is sufficient for this fix.

**Verify**: `pnpm tsc --noEmit` — no type errors. `pnpm test` all pass.

## Test plan

- Existing tests should pass. The frontend test in `Documents.test.tsx` doesn't exist (covered in plan 011), so no test-specific changes are needed.
- Verify with `cargo test` that the new status filter field doesn't break existing query logic.

## Done criteria

- [ ] `cargo build` exits 0
- [ ] `cargo test` all pass
- [ ] `pnpm verify` exits 0
- [ ] `grep -rn "rows.filter.*status" src/pages/secretary/Documents.tsx` returns 0 (status filter removed from client)
- [ ] `grep "30000" src/pages/secretary/ScanIntake.tsx` matches (interval changed to 30s)
- [ ] `plans/README.md` status row updated

## STOP conditions

- If the dynamic SQL in `fetch_documents` is built with a fixed number of placeholders (not dynamically), adding a new bind parameter will break the query. Check if `fetch_documents` uses `sqlx::query` (runtime, dynamic) or `sqlx::query!` (compile-time, fixed). It uses `sqlx::query` with binds, so adding binds is safe — but verify the bind count matches the `?` placeholders.

## Maintenance notes

- When adding new filter fields in the future, follow the same pattern: add to `DocumentListFilter`, add bind param + SQL condition, add frontend parameter.
- The scan intake polling interval can be further optimized with event-based notifications from the backend (e.g., emit a Tauri event when a scan is imported). That's a future improvement beyond this fix.

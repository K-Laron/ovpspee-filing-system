# Plan 006: Paginated document listing

> **Executor instructions**: Follow step by step. When done, update `plans/README.md`.
>
> **Drift check**: `git diff --stat 9b4b638..HEAD -- src-tauri/src/documents.rs src/pages/secretary/Documents.tsx src-tauri/src/commands.rs src/types.ts`

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MED
- **Depends on**: 005 (same query path, avoid merge conflicts)
- **Category**: perf
- **Planned at**: commit `9b4b638`, 2026-06-27

## Why this matters

`fetch_documents` returns all matching rows with no LIMIT/OFFSET. With thousands of documents, a single list invocation transfers the entire dataset over IPC. Adding pagination reduces bandwidth, serialization time, and frontend memory — especially important for the document list view, which is the app's primary browsing surface.

## Current state

- `src-tauri/src/documents.rs:1017-1091` — `fetch_documents` has no `LIMIT` or `OFFSET` clause. Returns all matching rows.
- `src-tauri/src/documents.rs:97-105` — `DocumentListFilter` with no pagination fields.
- `src/pages/secretary/Documents.tsx:80-103` — receives `DocumentItem[]`, sets state directly.
- `src/types.ts:73-92` — `DocumentItem` interface (frontend).
- The `AuditLogPage` pattern already exists (lines 252-263 in types.ts) — a paginated response with `entries`, `limit`, `offset`. Use this as a model.

**Conventions to follow**: Rust backend uses snake_case for commands and DB columns. TypeScript uses camelCase. Page components use `invoke` directly from `@tauri-apps/api/core`.

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
- `src-tauri/src/documents.rs` — add pagination fields, limit/offset to SQL, new response struct
- `src-tauri/src/commands.rs` — update `list_documents` signature
- `src/types.ts` — add paginated response type
- `src/pages/secretary/Documents.tsx` — implement "load more" pattern

**Out of scope**:
- Scan intake listing (typically fewer rows, different pattern)
- Trash listing (fewer rows)
- Public document listing (same backend — add pagination there too if the change is trivial, but not required)
- Infinite scroll or virtual scrolling (use simple "Load More" button — YAGNI for MVP)

## Steps

### Step 1: Add pagination fields to DocumentListFilter and create response type

In `src-tauri/src/documents.rs`, add to the filter struct:

```rust
pub struct DocumentListFilter {
    // ...existing fields...
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
```

Create a response struct:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct DocumentListPage {
    pub documents: Vec<DocumentItem>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}
```

**Verify**: `cargo build` compiles.

### Step 2: Add LIMIT/OFFSET + COUNT to fetch_documents

In `src-tauri/src/documents.rs`, modify `fetch_documents`:

1. Before the main SELECT, add a COUNT query (same WHERE predicates but no JOIN on attachment, no GROUP BY):
   ```rust
   let count_row = sqlx::query(
       "SELECT COUNT(*) as count FROM document d
        WHERE (? = 0 OR (d.is_hidden = 0 AND d.is_trashed = 0))
          AND ..." // same WHERE conditions as main query, no JOINs
   )
   // ... same binds as main query
   .fetch_one(pool).await?;
   let total_count: i64 = count_row.get("count");
   ```

2. Add `LIMIT ? OFFSET ?` to the main SELECT SQL and bind `filter.limit` and `filter.offset`.

3. Return `DocumentListPage` instead of `Vec<DocumentItem>`.

Default values: if `limit` is None, use 50 (reasonable page size). If `offset` is None, use 0.

**Verify**: `cargo test` passes (update any test that expects `Vec<DocumentItem>` from `list_documents`).

### Step 3: Update command signature and frontend type

In `src-tauri/src/commands.rs`, add `limit` and `offset` parameters to `list_documents`.

In `src/types.ts`, add the response type:

```typescript
export interface DocumentListPage {
  documents: DocumentItem[];
  total_count: number;
  limit: number;
  offset: number;
}
```

### Step 4: Update frontend Documents.tsx

In `src/pages/secretary/Documents.tsx`:

1. Change the invoke result type to `DocumentListPage`.
2. Add state: `page`, `hasMore`, `loadingMore`.
3. Change `loadDocuments` to pass `limit` and `offset`.
4. On initial load and filter change: reset offset to 0, replace document list.
5. Add a "Load More" button that calls the next page: increment offset by limit, append results.
6. Hide "Load More" when `!hasMore` or `loadingMore`.

```typescript
const PAGE_SIZE = 50;
const [page, setPage] = useState(0);
const [hasMore, setHasMore] = useState(true);
const [loadingMore, setLoadingMore] = useState(false);

// In loadDocuments:
const result = await invoke<DocumentListPage>('list_documents', {
    sessionId,
    search: search || null,
    // ... other filters ...
    limit: PAGE_SIZE,
    offset: 0,
});
setDocuments(result.documents);
setHasMore(result.offset + result.documents.length < result.total_count);
setPage(0);

// Load more handler:
const loadMore = async () => {
    setLoadingMore(true);
    const nextOffset = (page + 1) * PAGE_SIZE;
    const result = await invoke<DocumentListPage>('list_documents', {
        // ... same filters ...
        limit: PAGE_SIZE,
        offset: nextOffset,
    });
    setDocuments(prev => [...prev, ...result.documents]);
    setHasMore(nextOffset + result.documents.length < result.total_count);
    setPage(page + 1);
    setLoadingMore(false);
};
```

Render the button after the document list:
```tsx
{hasMore && (
  <div className="flex justify-center py-4">
    <button onClick={loadMore} disabled={loadingMore}
            className="rounded border border-border bg-surface px-4 py-2 text-sm hover:bg-accent">
      {loadingMore ? 'Loading...' : 'Load More'}
    </button>
  </div>
)}
```

**Verify**: `pnpm tsc --noEmit` — no type errors. `pnpm test` all pass.

## Test plan

- Existing tests should pass with minimal updates (the backend now returns `DocumentListPage` instead of `Vec<DocumentItem>` — any integration test calling `list_documents` needs to unwrap the new struct).
- Update integration tests in `src-tauri/tests/` that call `list_documents` to expect the new response format.

## Done criteria

- [ ] `cargo build` exits 0
- [ ] `cargo test` all pass
- [ ] `pnpm verify` exits 0
- [ ] `grep -rn "LIMIT" src-tauri/src/documents.rs` — shows the limit clause in fetch_documents
- [ ] Frontend shows "Load More" button when more documents exist
- [ ] `plans/README.md` status row updated

## STOP conditions

- If the COUNT query performance is poor (it re-uses the same WHERE but without JOINs), simplify it. The main query just needs an approximate count — consider `EXPLAIN QUERY PLAN` to verify.
- If the pagination change breaks the existing `list_trash_documents` or other callers that return full lists, leave those as-is (they return fewer rows).
- If the public document list (`list_public_documents`) also needs pagination, add it in a separate step — don't scope-creep.

## Maintenance notes

- PAGE_SIZE (50) is a reasonable default for a desktop app. If users report that the list feels short, it can be increased.
- The "Load More" pattern is simpler than infinite scroll and works well with the current UI. If infinite scroll is needed later, the backend changes are already in place.
- Any new list-style query should follow the same pagination pattern.

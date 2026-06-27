# Plan 007: spawn_blocking for PDF export

> **Executor instructions**: Follow step by step. When done, update `plans/README.md`.
>
> **Drift check**: `git diff --stat 9b4b638..HEAD -- src-tauri/src/documents.rs`

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: none
- **Category**: perf
- **Planned at**: commit `9b4b638`, 2026-06-27

## Why this matters

`export_document_pdf` calls `build_export_pdf` synchronously inside an async fn. This includes `fs::read`, image decoding (`image_crate::load_from_memory`), pixel iteration (`flatten_image_to_white`), and PDF serialization (`doc.save_to_bytes`). With a max pool of 5 DB connections and a shared async runtime, a document with large image attachments blocks the IPC thread for seconds, stalling other concurrent commands.

## Current state

- `src-tauri/src/documents.rs:792-850` — `export_document_pdf` is an async fn that calls `build_export_pdf` (sync) directly.
- `build_export_pdf` → `render_export_pdf` → `render_export_page` / `render_text_page` / `render_image_page` / `load_inline_image` / `flatten_image_to_white` — all synchronous, CPU-heavy operations.
- `load_inline_image` (line 1479) reads the entire file via `fs::read` then decodes via `image_crate::load_from_memory` — double allocation.
- `flatten_image_to_white` (line 1496) iterates every pixel in a nested loop with float math.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Rust build | `cargo build --manifest-path src-tauri/Cargo.toml` | exit 0 |
| Rust test | `cargo test --manifest-path src-tauri/Cargo.toml` | all pass |

## Scope

**In scope**:
- `src-tauri/src/documents.rs` — wrap blocking work in `spawn_blocking`

**Out of scope**:
- Refactoring PDF generation into a separate module (handled in plan 009)
- Optimizing `load_inline_image` to use `image::open` instead of `fs::read` + `load_from_memory` (nice-to-have, not required)
- Skipping alpha blend when image has no alpha channel (nice-to-have, not required)

## Steps

### Step 1: Wrap PDF generation in spawn_blocking

In `src-tauri/src/documents.rs`, change `export_document_pdf` to move the blocking work into `tokio::task::spawn_blocking`:

```rust
use tokio::task::spawn_blocking;

pub async fn export_document_pdf(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: Option<&str>,
    document_id: i64,
    output_path: &str,
) -> AppResult<String> {
    // ... existing auth checks (these stay async) ...
    let session = ...;  // keep async session validation
    let detail = get_document_internal(pool, document_id, false).await?;
    // ... existing visibility checks ...

    let output = validate_pdf_output_path(output_path)?;
    let generated_at = now_text();
    let exported_by = match &session {
        None => "Staff/Head Viewer".to_owned(),
        Some(s) => format!("{} user #{}", s.role, s.user_id),
    };

    // Clone what the closure needs
    let detail_clone = detail.clone();
    let storage_clone = storage.clone();  // StorageRoot: Clone + Send

    let pdf = spawn_blocking(move || {
        build_export_pdf(&detail_clone, &storage_clone, &generated_at, &exported_by)
    })
    .await
    .map_err(|e| AppError::Validation(format!("PDF generation failed: {}", e)))??;

    // The spawned_blocking closure returns AppResult<Vec<u8>>
    // The outer `?` unwraps the JoinError, the inner `?` unwraps the AppResult

    fs::write(&output, pdf)?;
    // ... rest of the function ...
}
```

Check that `StorageRoot` implements `Clone + Send + 'static`. Looking at `documents.rs:40-43`:

```rust
#[derive(Debug, Clone)]
pub struct StorageRoot {
    base: PathBuf,
}
```

`PathBuf` is `Send + Sync`, and the struct is `Clone` — good.

The `DocumentDetail` also needs `Clone`. Check the struct:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct DocumentDetail {
    pub document: DocumentItem,
    pub attachments: Vec<AttachmentItem>,
}
```

`DocumentItem` and `AttachmentItem` both derive `Clone` — good.

**Verify**: `cargo build` compiles. `cargo test` all pass.

### Step 2: Verify no Send/Sync issues

If the `spawn_blocking` closure fails to compile because a type isn't `Send`, the error message will name the type. Common fixes:
- Add `.clone()` for any captured references
- Change `&storage` to `storage.clone()` before the closure
- Ensure all captured values are owned, not borrowed

The critical constraint: `build_export_pdf` takes `&DocumentDetail` and `&StorageRoot` — these references must be to owned values that live long enough. Clone them into the closure.

**Verify**: `cargo build` — no "cannot be sent between threads safely" errors.

## Test plan

- Existing tests should pass unchanged. This is a runtime refactor with no behavior change.
- Consider adding a simple PDF export test (if not already present) that exports a document with a text attachment and verifies the output exists.

## Done criteria

- [ ] `cargo build` exits 0
- [ ] `cargo test` all pass
- [ ] `pnpm verify` exits 0
- [ ] The PDF export feature still works end-to-end (test with a small document containing 1 image attachment)
- [ ] `plans/README.md` status row updated

## STOP conditions

- If `spawn_blocking` causes Send errors, check that all captured types implement `Send`. `StorageRoot` and `DocumentDetail` should — but add explicit bounds if needed.
- If the refactored function introduces a lifetime issue, the `'static` bound on `spawn_blocking` requires owned data. Ensure no borrowed references are captured.

## Maintenance notes

- Any new CPU-heavy work added to `export_document_pdf` should go inside the `spawn_blocking` closure.
- The `load_inline_image` function uses `fs::read` + `image_crate::load_from_memory`. A future optimization would use `image::open(path)` which can memory-map the file, reducing peak memory allocation.
- `flatten_image_to_white` can skip the alpha blend entirely for JPEG images (which have no alpha channel). This is a separate optimization opportunity.

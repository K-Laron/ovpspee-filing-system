# Plan 009: Split documents.rs god module

> **Executor instructions**: Follow step by step. When done, update `plans/README.md`.
>
> **Drift check**: `git diff --stat 9b4b638..HEAD -- src-tauri/src/documents.rs`

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: 008 (similar structural pattern, good to have split commands.rs done first for practice)
- **Category**: tech-debt
- **Planned at**: commit `9b4b638`, 2026-06-27

## Why this matters

`documents.rs` is 1878 lines — the largest file in the Rust backend. It mixes: document CRUD, attachment I/O, FTS refresh, input validation, thumbnail generation, preview information, AND 300+ lines of inline PDF generation. This cognitive load means a simple change to document CRUD risks touching PDF code and vice versa. Extracting PDF export into its own module makes both easier to reason about and test.

## Current state

- `src-tauri/src/documents.rs:1-1878` — everything in one file
- Lines 792-850: `export_document_pdf` public function
- Lines 1341-1573+: `ExportPage` enum, `build_export_pdf`, `export_pages`, `render_export_pdf`, `render_export_page`, `render_text_page`, `render_image_page`, `render_footer`, `load_inline_image`, `flatten_image_to_white`, `wrap_text`, `append_pdf_text_markers`, etc.
- The preview functions (`preview_info_from_row`, `preview_kind`, `estimate_pdf_page_count`, `read_text_preview`, `extension_from_name`) are also in documents.rs — but plan 003 (step 2) should have moved them to `preview.rs`. If plan 003 hasn't been executed yet, extract them to `preview.rs` here.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Rust build | `cargo build --manifest-path src-tauri/Cargo.toml` | exit 0 |
| Rust test | `cargo test --manifest-path src-tauri/Cargo.toml` | all pass |

## Scope

**In scope**:
- `src-tauri/src/documents.rs` — remove PDF/export code
- `src-tauri/src/pdf_export.rs` — create (or `pdf.rs` — match naming convention)
- `src-tauri/src/lib.rs` — add `pub mod pdf_export;`
- `src-tauri/src/commands.rs` (or command modules from plan 008) — update import if `export_document_pdf` path changes

**Out of scope**:
- The `storage.rs` module (no need to create it now — StorageRoot stays in documents.rs)
- Split into more than 2 files (don't over-split)
- Behavior changes to any function
- The `flush_image_to_white` or `load_inline_image` optimization (keep as-is, just move)

## Steps

### Step 1: Create pdf_export.rs

Create `src-tauri/src/pdf_export.rs`. Move these items from `documents.rs`:

- `const PDF_PAGE_WIDTH_MM`, `PDF_PAGE_HEIGHT_MM`, `PDF_MARGIN_MM`, `PDF_FOOTER_Y_MM`, `PDF_TEXT_TOP_Y_MM`, `PDF_LINE_HEIGHT_MM`, `MAX_INLINE_IMAGE_PIXELS`
- `enum ExportPage` (with `Text` and `Image` variants)
- `pub fn build_export_pdf(...)` (make it `pub` — or `pub(crate)`)
- `fn export_pages(...)` (private)
- `fn load_inline_image(...)` (private)
- `fn flatten_image_to_white(...)` (private)
- `fn render_export_pdf(...)` (private)
- `fn render_export_page(...)` (private)
- `fn render_text_page(...)` (private)
- `fn render_image_page(...)` (private)
- `fn render_footer(...)` (private)
- `pub fn wrap_text(...)` (make it `pub(crate)`)
- `fn append_pdf_text_markers(...)` (private)

Also move the `use` imports that are only needed by PDF code:
- `use printpdf::{...}` — all of it
- `use crate::documents::StorageRoot` (or keep via `crate::documents`)
- `use image_crate::{DynamicImage, ...}` (if imported via printpdf)

Required dependencies for the new module:
```rust
use std::{fs, path::Path};
use printpdf::{
    image_crate::{self, DynamicImage, GenericImageView, Rgb, RgbImage},
    BuiltinFont, Image, ImageTransform, IndirectFontRef, Mm, PdfDocument, PdfLayerReference,
};
use crate::{
    documents::{DocumentDetail, StorageRoot, AttachmentItem},
    error::{AppError, AppResult},
};
```

Add `pub mod pdf_export;` to `src-tauri/src/lib.rs`.

**Verify**: `cargo build` — compiles.

### Step 2: Update documents.rs

In `src-tauri/src/documents.rs`:
- Remove all the moved constants, enum, and functions
- Remove `use printpdf::{...}` (now only in pdf_export.rs)
- Update `export_document_pdf` to call `crate::pdf_export::build_export_pdf(...)` instead of the local function

The `export_document_pdf` function stays in `documents.rs` because it handles auth checks, DB queries, and audit logging. The PDF generation is where the CPU-heavy work lives.

```rust
// In documents.rs, replace the build_export_pdf call:
// Old:
let pdf = build_export_pdf(&detail, storage, &generated_at, &exported_by)?;
// New:
let pdf = crate::pdf_export::build_export_pdf(&detail, storage, &generated_at, &exported_by)?;
```

**Verify**: `cargo build` compiles. `cargo test` all pass.

### Step 3: Clean up dead imports in documents.rs

After removing the PDF code, check which imports in `documents.rs` are no longer needed:
- `use printpdf::{...}` — entirely moved
- Any `image_crate` references — moved

Run `cargo build` and remove any "unused import" warnings.

**Verify**: `cargo build` — no warnings.

## Test plan

- `cargo test` must pass with no test changes. This is a pure mechanical extraction — no behavior changes.
- The PDF export integration tests (if any) should continue to pass since the public API is unchanged.

## Done criteria

- [ ] `cargo build` exits 0
- [ ] `cargo test` all pass
- [ ] `wc -l src-tauri/src/documents.rs` — significantly reduced (< 1550 lines, depending on plan 003)
- [ ] `grep -rn "printpdf\|ExportPage\|build_export_pdf\|render_export_pdf\|flatten_image_to_white\|load_inline_image" src-tauri/src/documents.rs` returns 0 (all moved to pdf_export.rs)
- [ ] `src-tauri/src/pdf_export.rs` exists and compiles
- [ ] `plans/README.md` status row updated

## STOP conditions

- If `build_export_pdf` relies on private types from `documents.rs` (like `DocumentInput` or internal helpers), make those types `pub(crate)` and move the needed helpers. Don't leave deeply-coupled types behind.
- If the printpdf imports in pdf_export.rs conflict with image crate imports (both provide `image_crate`), check that the `use printpdf::image_crate::{self, ...}` pattern works. If not, qualify calls: `printpdf::image_crate::load_from_memory(...)`.

## Maintenance notes

- PDF generation is now isolated in `pdf_export.rs`. Any changes to layout, branding, or output format touch only this file.
- The `wrap_text` function is shared between `documents.rs` (for preview text) and `pdf_export.rs` (for PDF layout). It lives in `pdf_export.rs` and is called as `crate::pdf_export::wrap_text(...)` from documents.rs if needed. If preview doesn't use it, keep it in pdf_export.rs only.
- After this split, `documents.rs` should be < 1550 lines. Future extractions (attachment logic, FTS logic) could reduce it further but are unnecessary.

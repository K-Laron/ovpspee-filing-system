# Developer Notes
## OVPSPEE Filing & Tracking System — CDHP Document 09

---

## 1. Architecture Decisions

### ADR-001: TRASH is a virtual category, not a separate table

**Decision:** The TRASH category is implemented as a system-seeded category record (`is_system = 1`) with additional columns on the `document` table (`is_trashed`, `trashed_at`, `original_category_id`, `original_folder_id`). There is no separate `trash` table.

**Rationale:** A separate table would require migrating document records between tables on trash and restore, breaking foreign key references from `attachment` and `audit_log`. Keeping everything in `document` means move, trash, and restore are all simple `UPDATE` operations with no orphan risk.

**Trade-off:** The `document` table has a few nullable columns that are only meaningful when `is_trashed = 1`. This is acceptable complexity for the benefit of a simpler relational model.

---

### ADR-002: Scan Intake is a staging table, not a filesystem watch

**Decision:** The Scan Intake feature uses an **explicit import action** (Secretary clicks "Import Scans" and picks files via a file dialog) rather than a background filesystem watcher that automatically ingests files dropped into a folder.

**Rationale:** A filesystem watcher (using `notify` crate) would require a background thread running at all times and would create confusing UX if files appeared in the intake without the secretary's explicit action. The explicit import flow is simpler to reason about and matches the mental model of a filing workflow. If the client requests auto-import in a future version, the infrastructure is already in place — add the watcher as an optional layer on top.

---

### ADR-003: Admin and Secretary roles are strictly non-overlapping

**Decision:** If a person needs both Admin and Secretary capabilities, two separate accounts must be created — one per role.

**Rationale:** This matches the PRD's explicit design intent and simplifies permission checks. There is no "superuser" role. The audit log relies on distinct user_id values to correctly attribute actions to a role context.

---

### ADR-004: Password reset is Admin-only (no email/SMTP flow)

**Decision:** Replaced the original email-based password reset with Admin-only direct password reset (`admin_reset_password` command).

**Rationale:** This is a single-machine desktop application. There is no guaranteed SMTP relay available. An email-based flow would require the Admin to configure an SMTP server — unnecessary complexity for a small office filing system. The Admin can reset any user's password directly from the User Management page.

---

### ADR-005: All categories are always public (OPEN)

**Decision:** Category-level visibility traits (`PUBLIC`, `HIDDEN`) from PRD v1.1 are removed. All categories are public/open.

**Rationale:** The "hide" concept is more granular and useful at the document level, not the category level. A secretary may need to hide specific sensitive documents within an otherwise public category (e.g., hide a confidential memo inside the BAC category while other BAC documents remain visible). Category-level hiding was a blunt instrument that would have hidden all documents in a category, which is not the desired behavior.

---

### ADR-006: File paths stored as relative paths only

**Decision:** `attachment.file_path` and `scan_intake.file_path` store paths relative to the configured storage base directory. Absolute paths are never stored in the database.

**Rationale:** Relative paths make the entire data directory portable. If the storage directory is moved (e.g., from `C:\Users\Admin\AppData\Roaming\...` to a new machine or a different drive), only the `settings.storage_base_dir` value needs to be updated — no database records need to change. This is critical for the backup/restore and cross-machine migration features.

---

### ADR-007: SQLx query approach — hybrid compile-time and runtime

**Decision:** Use `sqlx::query!` for static queries where a cached schema snapshot is available. Use `sqlx::query` (runtime) for dynamically-built queries (e.g., `IN` clauses with variable placeholder count) and when no compiled schema cache is maintained.

**Rationale:** `sqlx::query!` verifies SQL syntax and column type compatibility at compile time against a cached schema snapshot. However, it requires running `cargo sqlx prepare` after schema changes. For dynamic SQL (variable `IN (...)` placeholders, FTS5 subqueries) and in environments where maintaining a schema cache is impractical, `sqlx::query` with manual `row.get()` extraction is the pragmatic alternative. Both approaches coexist in the codebase — `query!` for stable queries, `query` for dynamic ones.

---

### ADR-008: Purge is Admin-only; Secretary can only trash and restore

**Decision:** Only Admin accounts can permanently delete (purge) documents from TRASH, whether individually or via "Empty Trash". Secretaries can move documents to TRASH and restore them, but cannot purge.

**Rationale:** Permanent deletion is irreversible. Restricting it to Admin creates a human checkpoint — if a Secretary accidentally trashes a document, it cannot be silently destroyed before an Admin reviews it. The auto-purge timer (Admin-configured) handles eventual cleanup without requiring the Admin to manually approve every purge.

**Implementation:** `purge_document` and `empty_trash` commands call `require_admin_role()`. The Purge button and Empty Trash button are not rendered in the UI for Secretary sessions — they are omitted entirely, not disabled.

---

### ADR-009: Restore falls back to category root if original folder is deleted

**Decision:** When `restore_document` is called and the original folder no longer exists (is inactive or was deleted), the document is restored to the original category with `folder_id = NULL` (category root / unfiled state), rather than raising an error.

**Rationale:** Blocking restore with an error forces the Secretary to manually purge or permanently lose data from TRASH. Falling back to the category root is always safe — the document is accessible and can be moved to a correct folder immediately. The restore result includes `folder_was_missing: true` so the UI can show an informational toast guiding the Secretary.

**Edge case:** If the original category is also inactive (`is_active = 0`), the document is still restored to that category. Inactive categories still exist in the database and can hold documents; they are simply hidden from the category creation UI. The Secretary can then move the document to an active category using the Move dialog.

---

## 2. Known Trade-offs

### Large file uploads via Tauri IPC

Tauri IPC serializes parameters as JSON. Sending file bytes as `Vec<u8>` in a command works but is memory-inefficient for large files — the entire file is held in memory twice (once in the frontend, once decoded in Rust). For MVP this is acceptable since the primary attachment types (scans, PDFs) are typically under 50 MB. If very large files become common, the correct solution is to use Tauri's streaming file upload API or have the frontend write the file to a temp location and pass only the path.

### Thumbnail generation for scan intake

Thumbnail generation for scanned images (JPEG, PNG) is handled by a Rust image processing crate (`image` crate). Thumbnail generation for PDFs requires rendering the first page, which needs a PDF rendering library. For MVP, PDF thumbnails can be omitted — show a generic PDF icon. Track this as a future improvement.

### Single-user desktop app; no concurrent access

SQLite is used with WAL mode, but the application is designed for one active user at a time on one machine. If the client ever needs multi-user concurrent access from multiple machines, the entire storage layer (SQLite → PostgreSQL, local filesystem → shared network storage) would need to be re-evaluated. This is explicitly out of scope per the PRD.

### Auto-purge trash runs on startup and daily timer

The trash auto-purge (configured in `settings.trash_auto_purge_days`) runs when the application starts and on a daily timer thereafter. If the application is closed for an extended period, documents will not be purged until the app is next opened. This is acceptable for a desktop filing system — there is no server process running in the background.

---

## 3. Open Questions (Resolved)

These were open questions from the README; all are resolved here.

| # | Question | Resolution |
|---|---|---|
| 1 | Scan arrangement when adding a document from scans | Scans are selected via checkbox in the intake picker. After selection, they appear in a reorderable thumbnail strip. The secretary drags to set the final order before saving. |
| 2 | Scan Intake workflow for multiple pages | Each scanned page is one file in the intake. The secretary groups pages for a document by selecting them together in the intake picker. |
| 3 | Why are Scan Intake and Add Document separate features? | Intake is a hardware I/O staging area (batch, anytime). Add Document is a metadata + filing workflow. Decoupling means the secretary can scan a batch of documents in one session, then file them one by one later without re-scanning. |
| 4 | Choosing scans in the intake picker | "Pick from Scan Intake" opens a full-screen picker with a thumbnail grid. Only unclaimed scans are shown. Checkboxes select individual pages. |
| 5 | Identifying which scans belong to which document | The secretary selects pages for one document, saves it, then returns to Add Document for the next. Claimed pages disappear from the picker. |
| 6 | UI for the intake picker | Thumbnail grid with checkbox multi-select, file name, timestamp, and file size. Preview on hover. Reorder rail at the bottom shows selected thumbnails in order. |
| 7 | What happens to scans after they are added to a document | Files are physically moved from `storage/intake/` to `storage/documents/{id}/scans/`. The `scan_intake` record is marked `is_claimed=1`. |
| 8 | Removing a scan during document creation | The scan is returned to the intake pool: file moved back to `storage/intake/`, `is_claimed` reset to 0. The document creation can continue without that scan. |
| 9 | Deleting a scan from the intake | Permanent delete (file + record). No trash for intake scans. Confirmation dialog required. |
| 10 | TRASH as separate table? | No separate table. TRASH is a system category; documents carry `is_trashed`, `trashed_at`, `original_category_id`, `original_folder_id` columns. See ADR-001. |
| 11 | Handling soft-deleted documents | Both Secretary and Admin can see trashed documents and restore them. **Only Admin can purge** (permanently delete). See ADR-008. |
| 12 | Settings for auto-purge | Yes — `settings.trash_auto_purge_days` (default: 30). Admin-configurable. "Empty Trash" (bulk purge) available only to Admin/IT Staff. Each trashed document shows a live countdown ("Purges in N days") visible to the Secretary. |
| 13 | Other improvements and refinements | Status counts bar on Documents page (replaced separate Dashboard page); drag-to-reorder attachments (dnd-kit); scan preview resolution/color settings before import; empty state illustrations; keyboard shortcuts; source tags ("Uploaded" vs "Scanned") on attachment thumbnails. |
| 14 | Contents of the CDHP | 12 `.md` files covering design, schema, API, components, testing, deployment, guidelines, troubleshooting, notes, roadmap, scan intake spec, and security checklist. |

---

## 3b. Performance Optimizations

### FTS5 Full-Text Search

`document_fts` FTS5 virtual table is created in migration 0003 and populated on every document INSERT/UPDATE/MOVE/TRASH/RESTORE via `refresh_document_fts()`. Previously, search used `LIKE '%term%'` (full table scan, no index). Now uses `document_fts MATCH ?` with a subquery (`d.document_id IN (SELECT rowid FROM document_fts WHERE document_fts MATCH ?)`). Search terms are split into whitespace-delimited, double-quoted tokens for AND semantics. See `src-tauri/src/documents.rs:fts5_query()`.

### N+1 Query Elimination

`validate_pending_scans()` in `scan_intake.rs` previously looped over each scan_intake_id and issued a separate `SELECT` per item. Replaced with a single `WHERE scan_intake_id IN (?, ?, ...)` batch query using `sqlx::query` with dynamic placeholder generation.

### Database Indexes (Migration 0008)

Added 9 indexes:
- FK indexes: `user(role_id)`, `scan_intake(created_by)`, `mobile_submission(category_id, folder_id, office_id, reviewed_by)`, `mobile_device(created_by)`
- Compound indexes: `document(category_id, date_received DESC)` for listing sort, `document(is_trashed, category_id, date_received DESC)` for filtered listings

### Bundle Size

Removed unused `@tanstack/react-query` dependency. Bundle reduced by ~24 KB (7 KB gzipped).

---

## 4. Future Improvements (Post-MVP)

These are explicitly out of scope for the initial release but are tracked here for future development cycles.

| Improvement | Notes |
|---|---|
| **Direct TWAIN/SANE scanner integration** | Replace the import-from-folder intake pattern with direct scanner control. Blocked on Rust TWAIN FFI maturity. |
| **Linux installer (AppImage / .deb)** | Build infrastructure is already in place via Tauri. Requires Linux-specific testing. |
| **Dark mode** | Tailwind `dark:` variants can be added incrementally. Requires updating all component color definitions. |
| **Storage health check** | Admin dashboard widget that verifies all `attachment.file_path` values point to existing files. Reports orphaned files and missing references. |
| **PDF thumbnail in scan intake** | Render the first page of a PDF scan as a thumbnail using a Rust PDF rendering crate. |
| **Bulk move / status change** | Bulk trash is done (checkbox + Promise.all), but bulk move to folder and bulk status change remain. |
| **Keyboard shortcut reference** | In-app shortcut help panel (e.g., `Ctrl+K` for search, `Ctrl+N` for new document). `/` focuses search, Escape closes detail — partially implemented. |
| **Multi-language support (i18n)** | If the system is ever deployed in a non-English context. Use `react-i18next`. |
| **Document version history** | Track edits to document metadata over time. Requires a `document_history` table. |

---

*End of Developer Notes*
*Next: `10_Vertical_Slice_Roadmap.md`*

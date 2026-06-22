# Vertical Slice Roadmap
## OVPSPEE Filing & Tracking System — CDHP Document 10

---

## Overview

Development proceeds in **10 vertical slices**. Each slice delivers a working, end-to-end feature across all layers: database migration → Rust backend → React UI → audit logging. Slices are ordered by dependency — each builds on the previous.

**Rule from README:** At the end of each slice, report current progress and ask questions before proceeding to the next slice.

---

## Slice Summary

| # | Slice | Key Deliverable | Depends On |
|---|---|---|---|
| 1 | Foundation & Auth | Running app, first-run setup, login/logout | — |
| 2 | Master Data | Categories, folders, offices CRUD (Admin) | Slice 1 |
| 3 | User Management | User CRUD, profile page | Slice 1 |
| 4 | Document Filing & Browsing | Create/edit documents, path-based file upload, staff/head viewer browsing | Slices 1, 2 |
| 5 | Document Visibility & Trash | Confidential auto-hide, hide/unhide, trash/restore, Admin-only purge | Slice 4 |
| 6 | Document Move & Status | Move between categories/folders, status management | Slice 4 |
| 7 | PDF Export & Attachment Preview | PDF export, paginated preview | Slice 4 |
| 8 | Audit Log & Retention | Admin audit log, Secretary own-activity view, search, PDF export, 36-month retention | Slice 1 |
| 9 | Scan Intake | Scan staging area, claim scans into documents | Slices 4, 8 |
| 10 | Backup, Restore & Installer | Full backup system, portable archive, production installer | Slices 1–9 |

---

## Slice 1 — Foundation & Authentication Shell

**Goal:** A running Tauri app with first-run setup, login/logout, and the layout skeleton.

| Layer | What Gets Built |
|---|---|
| **Database** | Migrations for `role`, `user`, `session` tables. Seed Admin/Secretary roles. |
| **Rust** | First-run detection, initial Admin account creation, login/logout commands, session management, Argon2id password hashing, `require_session()` helper, `write_audit_log()` helper. |
| **React** | First-Run Setup screen, Login screen, no-login Staff/Head Viewer top-nav layout, Secretary sidebar layout, Admin sidebar layout, Logout button. |
| **Config** | Tauri project scaffold, `tauri.conf.json`, window size constraints, dev build verified on Windows. |

**Deliverable:** User can install, run first-time setup, log in as Admin or Secretary, see the correct layout, and log out. Audit log captures login/logout events.

**End-of-Slice Report Topics:**
- Confirm Argon2id parameters (memory cost, iterations)
- Confirm session duration (proposed: 8 hours, reset on activity)
- Any issues with WebView2 on the target machine

---

## Slice 2 — Master Data Management (Admin)

**Goal:** Admin can configure categories, folders, and offices — the reference data everything else depends on.

| Layer | What Gets Built |
|---|---|
| **Database** | Migrations for `category`, `folder`, `office`, `settings` tables. Seed TRASH category, default settings. |
| **Rust** | CRUD commands for categories (create/update only; system categories immutable), folders (cannot be created under TRASH), offices. Validation: unique names, valid hex color. |
| **React** | Admin → Master Data page with 3 tabs (Categories, Folders, Offices). Add/Edit modals with color picker, icon picker. Table views with search. TRASH shown with lock icon and disabled Edit. |
| **Audit** | All create/edit operations logged. |

**Deliverable:** Admin configures the filing structure. Categories appear sorted alphabetically with TRASH always last. Folders are linked to categories. Default settings seeded.

**End-of-Slice Report Topics:**
- Confirm the initial set of categories and folders needed by the client
- Confirm the list of offices/departments
- Confirm default color palette

---

## Slice 3 — User Management (Admin)

**Goal:** Admin can create and manage Secretary/Admin accounts. All authenticated users can manage their own profile.

| Layer | What Gets Built |
|---|---|
| **Database** | Any missing `user` columns (address, profile_pic_path). |
| **Rust** | `create_user`, `update_user`, `admin_reset_password` commands. Profile: `get_my_profile`, `update_my_profile`, `change_my_password`, `upload_profile_picture`. |
| **React** | Admin → Users page (table, add/edit modals, deactivate toggle). Profile page (shared across roles): all fields, profile picture upload, change password form. |
| **Audit** | User CRUD, profile updates, password resets logged. |

**Deliverable:** Admin manages accounts. Deactivated users cannot log in. All authenticated users can update their own profile and profile picture.

**End-of-Slice Report Topics:**
- Confirm password complexity requirements (proposed: min 8 chars, 1 number, 1 special char)
- Confirm profile picture size/format limits (proposed: max 2 MB, JPEG/PNG only)

---

## Slice 4 — Document Filing & Browsing (Secretary + Staff/Head Viewer)

**Goal:** Secretary can create documents with file attachments. Authorized staff/heads can browse and search public documents through no-login viewer mode. **This is the core value slice.**

| Layer | What Gets Built |
|---|---|
| **Database** | Migrations for `document`, `attachment` tables. |
| **Rust** | `create_document` (metadata + file save), `update_document`, `list_documents` (role-based filtering), `get_document`, `serve_attachment`, `add_attachment`, `remove_attachment`, `reorder_attachments`. File storage helpers. |
| **React** | **Staff/Head Viewer Landing Page:** Top nav, category card grid (no TRASH), folder grid/list, document list, document view panel, AttachmentPreview (paginated), Export PDF button. **Secretary → Documents Page:** Same layout + all categories including TRASH (Secretary view). Secretary action kebab menu on document cards. **Secretary → Add Document Page:** Path-based file picker, thumbnail strip, metadata form, Save button. Confidential status auto-enables Hidden. **Secretary → Documents (inline summary bar):** Status counts by category (Filed/Archived/Confidential/Other) computed from current list results. |
| **Search** | Global search bar (debounced 300ms). Sort by date/name. Filter by category, folder, status. Active filter chips. |
| **Audit** | Document create, edit, attachment upload/remove logged. |

**Deliverable:** Primary filing workflow works end-to-end. Documents with attachments are created, browsed, searched, and viewed. Staff/Head Viewer users see only public non-hidden non-trashed documents.

**End-of-Slice Report Topics:**
- Confirm document metadata fields (any additional fields needed?)
- Confirm accepted file types for attachments (proposed: PDF, DOCX, XLSX, JPG, PNG, TIFF)
- Max attachment file size is finalized at 1 GB per file; UI warns above 250 MB
- Confirm pagination size for document list (proposed: 25 per page)

---

## Slice 5 — Document Visibility & Trash

**Goal:** Secretary can hide individual documents from Staff/Head Viewer access, trash documents, and restore them. Admin/IT Staff alone can purge documents permanently.

| Layer | What Gets Built |
|---|---|
| **Rust** | `set_document_hidden`, confidential auto-hide enforcement, `trash_document`, `restore_document`, Admin-only `purge_document`, Admin-only `empty_trash`. Auto-purge timer (startup + daily, based on `settings.trash_auto_purge_days`). |
| **React** | Hide/Unhide action on document kebab menu. Hidden indicator (EyeOff icon) on hidden documents (Secretary view). TRASH tab in Secretary's Documents page: shows trashed documents with Restore only; Purge and Empty Trash are hidden for Secretary. Admin TRASH tools show Restore, Purge, and Empty Trash. Trash auto-purge settings in Admin settings area. |
| **Audit** | HIDE, UNHIDE, TRASH, RESTORE_TRASH, PURGE logged. |

**Deliverable:** Secretary controls document-level visibility. TRASH functions as a recycle bin with configurable auto-purge. Staff/Head Viewer view is correctly filtered.

**End-of-Slice Report Topics:**
- Confirm default trash auto-purge period (proposed: 30 days)
- Purge and Empty Trash are finalized as Admin/IT Staff only
- Confirm trash auto-purge period remains 30 days after pilot use

---

## Slice 6 — Document Move & Status Management

**Goal:** Secretary can reassign a document to a different category/folder and change document statuses.

| Layer | What Gets Built |
|---|---|
| **Rust** | `move_document` command (validates folder belongs to target category; blocks move to TRASH). |
| **React** | Move Document dialog: shows current location as breadcrumb, category dropdown (excludes TRASH), dynamic folder dropdown. Status dropdown in Edit Document panel with conditional `status_other` field. Toast notifications for move success/failure. |
| **Audit** | MOVE logged with previous/new category_id and folder_id in description. |

**Deliverable:** Secretary reorganizes documents without data loss. Files stay in place on disk; only DB metadata changes.

**End-of-Slice Report Topics:**
- Confirm whether a Secretary can move documents between categories freely, or if there should be any restrictions
- Document statuses are Filed, Archived, Confidential, Other; Confidential auto-enables hidden

---

## Slice 7 — PDF Export & Attachment Preview

**Goal:** Any user can export documents as PDF. Attachment preview supports pagination for large documents.

| Layer | What Gets Built |
|---|---|
| **Rust** | `export_document_pdf` command — renders UEP/OVPSPEE letterhead, document metadata, attachment pages, page numbers, timestamp, and footer/certification text into a PDF using an offline bundled renderer. Avoid external wkhtmltopdf installation dependency. Tauri file save dialog integration. |
| **React** | Export PDF button on document view panel (Staff/Head Viewer + Secretary). Paginated attachment preview with lazy loading. "PAGE N of N" navigation with prev/next buttons. Loading indicator while pages render. |

**Deliverable:** Documents exportable as professional PDFs. Large attachments (100+ pages) browsable without UI freeze.

**End-of-Slice Report Topics:**
- PDF layout must include UEP/OVPSPEE letterhead/logo; confirm exact asset and wording
- Confirm whether exported PDF should include all attachments or only the selected page?

---

## Slice 8 — Audit Log & Retention (Admin)

**Goal:** Admin/IT Staff can view, search, filter, and export all audit logs. Secretary can view only their own activity history. Retention policy auto-cleans old entries after the 36-month default.

| Layer | What Gets Built |
|---|---|
| **Database** | Retention and trash purge config in `settings` table (already seeded in Slice 2). |
| **Rust** | Admin-only `list_audit_logs` (paginated, filtered), Secretary-safe `list_my_audit_logs`, `export_audit_log_pdf`, `update_retention_setting`, `run_audit_cleanup`. Auto-cleanup on startup + daily timer. |
| **React** | Admin → Audit Log page: table (50/page), search, filter panel (date range, action type, user), Export PDF button, ⚙ Settings icon → Retention Policy modal with months input + "Run Cleanup Now" button. Secretary → My Activity page: own actions only, no user filter. |
| **Audit** | Retention policy changes and cleanup runs are themselves logged. |

**Deliverable:** Full audit accountability. Admin controls log retention. PDF audit reports exportable.

**End-of-Slice Report Topics:**
- Default audit log retention is finalized at 36 months
- Secretary access is finalized as own-activity only

---

## Slice 9 — Scan Intake (Secretary)

**Goal:** Secretary can import scanned page files into a staging area, review and organize them, and attach selected scans to documents.

See `11_Scan_Intake_Specification.md` for the full detailed specification of this slice.

| Layer | What Gets Built |
|---|---|
| **Database** | `scan_intake` table migration. |
| **Rust** | `list_intake_scans`, `import_scan_files`, recoverable `delete_intake_scan`, `restore_deleted_intake_scan`, `purge_deleted_intake_scans`, `claim_scan_as_attachment`, `return_scan_to_intake`. Thumbnail generation for JPEG/PNG. Storage path management for intake directory. |
| **React** | **Secretary → Scan Intake Page:** Thumbnail grid of unclaimed scans, "Import Scans" button (file dialog), checkbox multi-select, recoverable delete selected, Deleted Scans recovery view. **Add Document → From Scan Intake Tab:** IntakePicker slide-over (checkbox grid, preview on hover, reorder rail). Thumbnail strip integration with Upload tab. Source tags on thumbnails. |
| **Audit** | SCAN logged on file import. Claim/return logged as part of document create/edit operations. |

**Deliverable:** Secretary can scan physical documents using native scanner software, import the resulting files into the intake staging area, and attach selected pages to document records without re-scanning.

**End-of-Slice Report Topics:**
- Confirm accepted scan file formats (proposed: JPG, PNG, PDF, TIFF)
- Confirm whether the secretary needs a resolution/color setting UI before importing (proposed: defer, handle in scanner's native software)
- Confirm thumbnail quality setting (proposed: max 300px wide, JPEG quality 70)

---

## Slice 10 — Backup, Restore & Production Installer

**Goal:** Admin can create, schedule, export, import, and restore backups. Production installer is built and verified.

| Layer | What Gets Built |
|---|---|
| **Rust** | `create_backup`, `export_backup_archive`, `import_backup_archive`, `restore_from_backup`. Scheduled backup via background timer. Backup validation (manifest.json, schema/app version, checksum). Automatic pre-restore safety backup. App restart after restore. |
| **React** | Admin → Backup & Restore page: Create Backup section, Scheduled Backup section, Export/Import Portable Backup section, Restore from Backup Folder section, Backup History list. |
| **Installer** | Final `cargo tauri build` producing `.msi` and `.exe`. Desktop shortcut, Start Menu entry, Add/Remove Programs registration. Code signing (if certificate available). |
| **Audit** | BACKUP, RESTORE, EXPORT, IMPORT logged. |

**Deliverable:** Complete data protection and portability. Production installer ready for deployment. Full cross-machine migration tested.

**End-of-Slice Report Topics:**
- Confirm backup schedule default (proposed: Daily at 02:00)
- Backup retention default is finalized at keep last 10
- Backup destination default is finalized as local app-data backup folder; Admin may change it and should copy backups off-device
- Confirm whether the installer should embed the WebView2 bootstrapper

---

## End-of-Slice Reporting Template

At the end of each slice, provide the following before proceeding:

```
## Slice N — Completion Report

### What was completed
- [List of features/commands/components built]

### What was deferred
- [Anything that was not completed and why]

### Test results
- Unit tests: X passing, Y failing
- Manual verification checklist: X/Y items passed
- Known failing items: [describe]

### Questions before proceeding to Slice N+1
1. [Question]
2. [Question]
```

---

*End of Vertical Slice Roadmap*
*Next: `11_Scan_Intake_Specification.md`*

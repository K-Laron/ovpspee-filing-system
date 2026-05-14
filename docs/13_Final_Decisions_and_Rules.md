# Final Decisions and Implementation Rules
## OVPSPEE Filing & Tracking System — CDHP Document 13

---

## 1. Purpose

This document is the source of truth for decisions finalized after review of the CDHP. If any earlier document conflicts with this file, follow this file and update the older section.

---

## 2. Final Product Decisions

| Topic | Final Decision |
|---|---|
| Real users of the app | The app is for authorized OVPSPEE staff and heads only. The no-login Guest mode is a read-only Staff/Head Viewer on the office machine, not a public kiosk or internet-facing portal. |
| Operational Admin | IT Staff is the real Admin/operator for system setup, account management, backup/restore, purge, and security-sensitive configuration. |
| Initial categories/folders/offices | Do not hard-code user-created categories, folders, or offices beyond system records. Admin configures these during Slice 2. The app may provide an optional starter-import/template, but the client must confirm the final list before deployment. |
| Confidential documents | Selecting `Confidential` must automatically set `is_hidden = true` by default. The UI must warn that confidential documents are hidden from the Staff/Head Viewer. |
| Attachment file size | 1 GB per attachment is the hard maximum. The UI should warn when files exceed 250 MB. Files must be copied by path-based backend file handling, not transferred as large byte arrays through IPC. |
| Scan Intake delete behavior | Intake deletion is recoverable. Deleting an unclaimed scan moves it to a Deleted Scans view with `is_deleted = 1`; it is restored or permanently purged later by retention cleanup. |
| Backup destination | Default backups are stored on the local device under the app data directory. Admin may choose an external, network, or removable drive. The UI must warn that local-only backups do not protect against device loss or drive failure. |
| PDF export layout | Exported PDFs must use UEP/OVPSPEE letterhead, document metadata, attachment pages, page numbers, generation timestamp, and a short system-generated certification/footer. |
| Secretary audit access | Admin sees all audit logs. Secretary sees only their own activity history through a separate My Activity view/command. |
| Audit retention | Default audit log retention is 36 months. Admin may configure any value from 24 to 36 months unless a stricter policy is later provided. |
| Trash purge | Secretary can trash and restore documents. Only Admin can permanently purge documents or empty TRASH. |
| Restore behavior | If the original folder no longer exists, restore to the original category root and show an info toast. Do not fail restore only because the folder is gone. |
| Public category listing | Keep `list_categories` Admin-only. Add `list_public_categories` for no-login Staff/Head Viewer browsing. |

---

## 3. Final Implementation Choices

| Area | Implementation Rule |
|---|---|
| File upload/copy | Use path-based file copy from OS file picker results. Rust validates existence, extension, MIME/type sniff where possible, size, destination path, and stores only relative destination paths in SQLite. |
| Search | Use SQLite indexed filters plus FTS5 for document name, sender, receiver, remarks, and optionally office/category/folder denormalized text. |
| PDF rendering | Use an offline, bundled Rust-compatible rendering approach. Preferred: a Typst template rendered via bundled/embedded Rust libraries or a pure Rust PDF pipeline. Avoid requiring an external wkhtmltopdf installation. |
| Backup format | Backup archive must include database, storage directory, `manifest.json`, schema version, app version, created_at, source machine/user metadata where safe, and checksums. |
| Restore safety | Before any restore, automatically create a pre-restore safety backup of the current state. Restore validates manifest and checksums before replacing data. |
| Session expiry during forms | Preserve unsaved form data locally in component state/local draft storage. If session expires, redirect to login and restore the draft after re-authentication where safe. |
| Backup retention | Default to keeping the last 10 local backups. Admin may configure retention count. |
| Deleted scan retention | Default recoverable deleted scan retention is 30 days. Admin may purge sooner if needed. |

---

## 4. Permission Matrix

| Feature / Command Group | Staff/Head Viewer (No Login) | Secretary | Admin / IT Staff |
|---|---:|---:|---:|
| Browse public categories | Yes | Yes | Yes |
| Browse public documents | Yes | Yes | Yes |
| View hidden/confidential documents | No | Yes | Yes |
| Create/edit documents | No | Yes | No by default, unless explicitly enabled later |
| Hide/unhide documents | No | Yes | Yes |
| Trash documents | No | Yes | Yes |
| Restore from TRASH | No | Yes | Yes |
| Purge/Empty TRASH | No | No | Yes |
| Scan Intake import/claim/restore deleted scans | No | Yes | No by default |
| Manage users | No | No | Yes |
| Manage categories/folders/offices | No | No | Yes |
| View own activity | No | Yes | Yes |
| View full audit log | No | No | Yes |
| Backup/restore | No | No | Yes |

---

## 5. Pending Client Data

The following are intentionally unresolved because the client must provide the actual operational data:

1. Final category list.
2. Folder list under each category.
3. Office/department sender list.
4. Exact PDF letterhead assets and signature/certification wording.
5. Whether backups will also be copied to an external drive or network location.

---

*End of Final Decisions and Implementation Rules*

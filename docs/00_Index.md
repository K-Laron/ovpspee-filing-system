# OVPSPEE Filing & Tracking System
## Complete Developer Handoff Pack (CDHP)

**Project:** OVPSPEE Filing and Tracking System
**Client:** Office of the Vice President for Special Programs and External/Extension Engagement — University of Eastern Philippines
**Stack:** Tauri v2 + Rust + React + TypeScript + SQLite
**Target Platform:** Windows 10+ (64-bit), Windows-first build; Linux deferred post-MVP
**CDHP Version:** 1.1
**Last Updated:** May 14, 2026

---

## How to Use This Pack

Read the documents in order on first pass. Return to individual files as reference during development. Each file is self-contained but cross-references others where relevant.

| # | File | Purpose | Read When |
|---|---|---|---|
| 00 | `00_Index.md` | This file — navigation and overview | First |
| 01 | `01_Design_System_Overview.md` | Colors, typography, spacing, component library, layout rules | Before writing any UI |
| 02 | `02_Database_Schema_Documentation.md` | Full SQLite schema, all tables, constraints, design rationale | Before writing any Rust backend |
| 03 | `03_Backend_API_Documentation.md` | All Tauri IPC commands, inputs, outputs, error codes | During Rust + IPC development |
| 04 | `04_Frontend_Component_Documentation.md` | All React components, props, state, composition patterns | During React development |
| 05 | `05_Testing_Strategy_Documentation.md` | Unit, integration, and manual test plans per slice | Before writing any tests |
| 06 | `06_Deployment_Documentation.md` | Build pipeline, installer packaging, first-run setup, update strategy | Before release |
| 07 | `07_Developer_Guidelines.md` | Code style, commit conventions, project structure, PR workflow | On project setup |
| 08 | `08_Developer_Troubleshooting.md` | Common errors, known issues, and fixes | When something breaks |
| 09 | `09_Developer_Notes.md` | Architecture decisions, open questions, known trade-offs | Anytime |
| 10 | `10_Vertical_Slice_Roadmap.md` | 10-slice development plan with deliverables and checkpoints | During planning and sprint work |
| 11 | `11_Scan_Intake_Specification.md` | Full scan intake feature: workflow, UI, file lifecycle, edge cases | Before building Slice 9 |
| 12 | `12_Security_Compliance_Checklist.md` | Security requirements checklist, password rules, path safety, audit coverage | Before any auth or file I/O work |
| 13 | `13_Final_Decisions_and_Rules.md` | Final product decisions, permission matrix, implementation rules | Before development and whenever conflicts appear |

---

## Project Summary

The OVPSPEE Filing and Tracking System is a **single-machine desktop application** that serves as a centralized, organized repository for fully processed documents. It is not a document workflow or routing system — it is a structured digital filing cabinet.

### Core Hierarchy

```
Category (e.g., BAC, BOR, Tracer Studies)
  └── Folder (e.g., PPMP 2025, Minutes 2024)
        └── Document (with metadata + attachments)
```

### Three User Roles

| Role | Access |
|---|---|
| **Staff/Head Viewer (No Login)** | Read-only; public, non-hidden, non-trashed documents only; intended only for authorized OVPSPEE staff/heads using the office machine |
| **Secretary** | Full read/write; can file, edit, hide, move, and trash documents; can scan |
| **Admin / IT Staff** | System configuration: users, categories, folders, offices, full audit logs, backup/restore, purge |

### Key Design Decisions (Finalized)

| Decision | Resolution |
|---|---|
| Category traits | Categories are always **OPEN** (public). Admin cannot set visibility traits. |
| Document hiding | Secretaries toggle individual documents as **hidden**. Hidden documents are invisible to the Staff/Head Viewer. Selecting **Confidential** auto-enables hidden by default. |
| TRASH | System-generated, immutable category. Virtual view — no separate database table. Documents carry `is_trashed`, `trashed_at`, `original_category_id`, `original_folder_id` columns. |
| TRASH visibility | Hidden from no-login Staff/Head Viewer users. Visible to Secretary and Admin only. |
| Category ordering | Alphabetical; TRASH always appears last, regardless of name. |
| Scanner integration | Scan Intake Folder pattern (MVP). Direct TWAIN/SANE deferred post-MVP. |
| Password reset | Admin-only password reset. No email/SMTP dependency. |
| Platform target | Windows 10+ first. Linux AppImage/deb deferred post-MVP. |
| Scan + Document workflow | Scan Intake is a staging area. Add/Create Document is the filing workflow. They are decoupled. |
| Attachment size | Hard maximum: 1 GB per file. Files are copied by backend path-based handling, not large IPC byte transfer. |
| Scan deletion | Deleted intake scans are recoverable for 30 days by default. |
| Audit retention | Default audit log retention is 36 months; Secretary can view own activity only. |
| Backup default | Local device backup folder by default, with warning to copy to external/network storage. |
| PDF export | Uses UEP/OVPSPEE letterhead, metadata, attachments, page numbers, generation timestamp, and footer/certification text. |
| Soft-delete / trash purge | Secretary can trash/restore. Only Admin/IT Staff can purge or empty TRASH. Admin-configurable auto-purge (default: 30 days). |

---

## Source Documents

This CDHP synthesizes and supersedes the following planning documents:

1. `OVPSPEE_Filing_Tracking_System_PRD_UPDATED_V_3.md` — Product Requirements Document v1.1
2. `implementation_plan.md` — Feasibility verdict, vertical slices, backup specification
3. `README.md` — Latest system adjustments (document-level hiding, TRASH category, Secretary dashboard, Scan Intake page)

In any conflict, `13_Final_Decisions_and_Rules.md` takes precedence over this pack. If a conflict predates Document 13, update the older section to match Document 13.

---

*End of Index*

# Scan Intake Specification
## OVPSPEE Filing & Tracking System — CDHP Document 11

---

## 1. Purpose

The Scan Intake feature is a **staging area for scanned pages**. It allows the Secretary to import scanned image files — produced by the scanner's native software — into the system without immediately attaching them to a document. The Secretary can later select scans from this staging area when creating or editing a document.

This design is chosen because:
- Scanning is a **batch hardware operation** (scan many pages at once).
- Filing is a **metadata + organization operation** (assign pages to the correct document, in the correct order, with correct metadata).
- Decoupling the two operations allows the Secretary to scan a batch in the morning and file documents throughout the day.

---

## 2. Scope (MVP)

**In scope for MVP:**
- Import scan files via file picker dialog (Secretary selects files produced by the scanner's native app).
- Thumbnail preview of each scan in a grid.
- Checkbox multi-select for batch delete.
- Claim scans into a document (from Add Document or Edit Document).
- Return a claimed scan back to the intake pool.
- Delete scans from the intake recoverably, with a Deleted Scans recovery view and retention-based purge.

**Out of scope for MVP (future):**
- Direct scanner control via TWAIN/SANE (auto-detect scanner, send scan command from the app).
- Scanner settings UI (resolution, color mode, duplex) — handled in the scanner's native software.
- Auto-import (filesystem watcher that ingests files without a manual import action).
- OCR (text extraction from scanned pages).

---

## 3. File Format Support

The intake importer accepts the following file types:

| Format | Extension | Notes |
|---|---|---|
| JPEG | `.jpg`, `.jpeg` | Most common output from scanners |
| PNG | `.png` | Lossless, larger files |
| TIFF | `.tif`, `.tiff` | Multi-page TIFF: each page is split into a separate intake record |
| PDF | `.pdf` | Each page treated as a separate scan thumbnail (future); for MVP, imported as a single unit |

**Rejected file types** (show error toast, do not import): `.exe`, `.zip`, `.docx`, and any format not in the list above. Maximum scan file size follows the attachment cap of 1 GB per file; warn above 250 MB.

---

## 4. File Storage Layout

```
storage/
  intake/
    {filename_with_timestamp}.jpg      ← imported scan files
    {filename_with_timestamp}.png
    thumbnails/
      {filename_with_timestamp}_thumb.jpg   ← generated thumbnails (max 300px wide, JPEG 70%)
```

**File naming on import:**
Original filenames are preserved but prefixed with a timestamp to avoid collisions:
```
scan_20260512_143001_001.jpg     ← first file in a batch imported at 14:30:01 on May 12
scan_20260512_143001_002.jpg     ← second file in the same import batch
```

If the original filename is used as-is and a file with that name already exists in intake, append `_1`, `_2`, etc. before the extension.

---

## 5. Scan Intake Page Layout

```
┌────────────────────────────────────────────────────────────┐
│  Scan Intake                     [Import Scans]  [3 scans] │
├────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                 │
│  │ ☐ [img]  │  │ ☐ [img]  │  │ ☐ [img]  │                 │
│  │ scan_001 │  │ scan_002 │  │ scan_003 │                 │
│  │ 2.4 MB   │  │ 1.8 MB   │  │ 3.1 MB   │                 │
│  │ May 12   │  │ May 12   │  │ May 12   │                 │
│  │ 14:30    │  │ 14:30    │  │ 14:31    │                 │
│  └──────────┘  └──────────┘  └──────────┘                 │
│                                                            │
│  [Delete Selected (1)] ← recoverable; appears when checked│
└────────────────────────────────────────────────────────────┘
```

### Grid behavior

- Default view: icon/thumbnail grid, 4 columns, responsive.
- Each card shows: thumbnail image, file name, file size, scanned timestamp.
- Claimed scans and deleted scans are **not shown** in normal intake (filtered out in `list_intake_scans`). Deleted scans appear only in the Deleted Scans recovery view.
- Empty state: centered illustration + "No scans in intake. Click 'Import Scans' to begin."

### Import Scans button

- Opens the OS file picker dialog (multi-select enabled).
- Accepted types: image (JPEG, PNG, TIFF) and PDF.
- After selection:
  1. Files are copied to `storage/intake/`.
  2. Thumbnails are generated.
  3. `scan_intake` records are inserted.
  4. Grid refreshes automatically.
  5. Success toast: "5 scans imported successfully."

### Delete Selected / Deleted Scans Recovery

- Appears in a bottom action bar when ≥1 scan is checked.
- Clicking opens a `ConfirmDialog`: "Delete 2 scans? This cannot be undone."
- On confirm: files and records are permanently deleted; grid refreshes.

---

## 6. Intake Picker (Inside Add/Create Document)

The Secretary reaches this when clicking **"Pick from Scan Intake"** on the Add Document page (From Scan Intake tab) or Edit Document panel.

### Picker Layout

```
┌────────────────────────────────────────────────────────────┐
│  Pick Scans                                          [×]   │
│  ─────────────────────────────────────────────────────     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ ☑ [img]  │  │ ☐ [img]  │  │ ☑ [img]  │  │ ☐ [img]  │   │
│  │ scan_001 │  │ scan_002 │  │ scan_003 │  │ scan_004 │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│                                                            │
│  Selected (2):  [scan_001] [scan_003]  ← reorder rail      │
│  Drag to reorder ↕                                         │
│  ─────────────────────────────────────────────────────     │
│              [Cancel]    [Add 2 Scans to Document]         │
└────────────────────────────────────────────────────────────┘
```

### Picker behavior

1. Opens as a full-width slide-over panel or a large modal (`max-w-4xl`).
2. Shows all unclaimed scans (`is_claimed = 0`) in a thumbnail grid.
3. Hover over a thumbnail → shows a larger preview.
4. Checking a scan adds it to the **reorder rail** at the bottom.
5. Unchecking removes it from the rail.
6. Rail items are draggable (dnd-kit) — the Secretary sets the final attachment order here.
7. "Add N Scans to Document" button → closes picker, passes the ordered list of `intake_id` values back to the Add Document form.
8. The selected scans appear in the **thumbnail strip** on the Add Document page, tagged as "🖷 Scanned".

---

## 7. Attachment Thumbnail Strip (Add Document Page)

The thumbnail strip shows all attachments (uploaded files + selected scans) in the order they will be attached and exported.

```
┌────────────────────────────────────────────────────────────┐
│  Attachments (3)                                           │
│                                                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  [+ Add More]   │
│  │ [img]    │  │ [img]    │  │ PDF icon │                  │
│  │ scan_001 │  │ scan_003 │  │ letter.  │                  │
│  │ 🖷 Scanned│  │ 🖷 Scanned│  │ 📄 Upload│                  │
│  │ [×]      │  │ [×]      │  │ [×]      │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
│  ↕ Drag to reorder                                         │
└────────────────────────────────────────────────────────────┘
```

- [×] button on a thumbnail: removes from the strip.
  - If it's an uploaded file: file is discarded from the pending upload list (not saved yet, so no disk change).
  - If it's a scan: the scan is **returned to the intake pool** (not deleted; available again in the picker).
- Drag-to-reorder: sets `sort_order` on the resulting `attachment` records.
- "+ Add More": opens file picker for uploads OR the intake picker for scans, depending on the active tab.

---

## 8. Full Scan Lifecycle

```
Scanner (native software)
       │
       │ Secretary scans; native software saves files to desktop/folder
       │
       ▼
[Import Scans button]
       │
       │ File picker → Secretary selects files
       │
       ▼
storage/intake/scan_YYYYMMDD_HHMMSS_NNN.jpg
scan_intake record (is_claimed = 0, is_deleted = 0)
       │
       ├──── Secretary deletes from intake ──────────► scan_intake.is_deleted = 1
       │                                               appears in Deleted Scans recovery view
       │
       ├──── Secretary restores deleted scan ────────► scan_intake.is_deleted = 0
       │
       ├──── Retention/Admin purge ──────────────────► File deleted permanently + record removed
       │
       └──── Secretary adds to document ─────────────► File moved to:
                    │                                  storage/documents/{id}/scans/
                    │                                  attachment record created
                    │                                  scan_intake.is_claimed = 1
                    │
                    └── Secretary removes from document ──► File moved back to intake
                        (before or after save)                scan_intake.is_claimed = 0
                                                              attachment record deleted
```

---

## 9. Edge Cases

| Scenario | Handling |
|---|---|
| Secretary imports the same file twice | Timestamp prefix ensures different filenames; both are imported as separate records. No deduplication. |
| Thumbnail generation fails | Set `thumbnail_path = NULL`; show generic file type icon in grid. Do not block import. |
| Scan file is corrupted/unreadable | Import fails for that file; show error toast with file name; continue importing other files in the batch. |
| Secretary closes Add Document without saving after selecting scans | Scans remain in the intake pool (`is_claimed = 0`). The claim only becomes permanent when the document is saved. |
| Original folder of a document is deleted before trash restore | `restore_document` succeeds by restoring to the original category root with `folder_was_missing = true`. Frontend shows info toast: "Restored to category root — original folder no longer exists." |
| Intake storage directory is full | Import fails with `ERR_IO: Disk full`. Error toast shown. Secretary must free disk space. |
| Very large number of scans in intake (>500) | Grid virtualizes using `react-window` or similar. Performance target: scrolling at 60fps with 500+ thumbnails. |

---

## 10. Acceptance Criteria for Slice 9

- [ ] Secretary can import JPEG, PNG, and PDF files from a file picker into the intake grid.
- [ ] Thumbnails render within 3 seconds per image.
- [ ] Claimed scans and deleted scans are hidden from the normal intake grid.
- [ ] Delete Selected soft-deletes scans, hides them from the normal grid, and shows them in Deleted Scans.
- [ ] Deleted scans can be restored before retention purge.
- [ ] Intake picker shows unclaimed scans with checkboxes and a reorder rail.
- [ ] Selecting scans in the picker and confirming adds them to the Add Document thumbnail strip tagged as "Scanned".
- [ ] Removing a scan from the strip (before save) returns it to the intake pool.
- [ ] Saving the document with selected scans moves the files to `storage/documents/{id}/scans/` and marks them as claimed.
- [ ] Audit log contains a SCAN entry for each imported scan batch.

---

*End of Scan Intake Specification*
*Next: `12_Security_Compliance_Checklist.md`*

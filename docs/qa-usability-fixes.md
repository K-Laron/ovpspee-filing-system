# QA Usability Fixes

Date: May 18, 2026

## What Changed

- Destructive actions now ask for confirmation before they run.
- Permanent Trash purge and backup restore require typed confirmation.
- User-facing screens avoid raw technical error messages.
- Password forms show the rules and require password confirmation.
- Empty states explain what happened and what the user can do next.
- The app shell supports smaller desktop windows with a collapsible menu.
- Scanner settings no longer offer unsupported direct PDF capture.
- Dates are displayed in a readable long format.

## Manual QA Checklist

- At 1024x700, Admin and Secretary pages should be usable without horizontal page scroll.
- At 900x640, navigation should collapse behind a menu button.
- Trash purge should require typing `PURGE`.
- Backup restore should require typing the selected backup name.
- Moving a document to Trash should show a confirmation and remain restorable.
- Attachment removal should show a confirmation.
- Scanner and printer empty states should explain connection and refresh steps.
- Password creation, reset, and change should reject mismatched or weak passwords before calling the backend.
- Direct scanner capture should allow image formats only. PDF files should still be importable through Scan Intake.
- Empty search/results screens should tell users what to try next.

## Notes For Non-Technical QA

- If an error appears, it should describe the problem in plain language. It should not show source file paths, database codes, stack traces, or backend command names.
- If a button deletes, purges, restores, or removes data, the user should see a confirmation first.
- If a list is empty, the screen should explain the next action instead of stopping at "No records."

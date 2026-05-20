# OVPSPEE Android Mobile V1 Design

Date: May 20, 2026
Owner: Kenneth / Awani
Project: OVPSPEE Filing and Tracking System

## Status

Approved for planning. This document defines the mobile v1 design direction only; implementation still requires a separate implementation plan and approval before code changes.

## Existing Project Context

The current app is a Windows-first desktop system built with Vite, React, TypeScript, Tauri v2, Rust, and SQLite. Desktop users interact with Tauri IPC commands; Android cannot use those commands directly. The current product documentation describes mobile as out of MVP scope, so Android support is a new phase.

## Approved Direction

Mobile v1 is a real Android app, not a PWA. It targets Android only, uses React Native with TypeScript, and connects over the office network to the existing Windows office PC acting as the hub.

Mobile v1 is Secretary-only. Staff/Head mobile viewing is deferred to a later authenticated viewer phase, and full mobile parity is deferred to phase 2.

## Goals

- Give Secretaries a phone-native way to capture documents with the camera or choose files.
- Require complete Add Document metadata before upload.
- Send mobile submissions to desktop review before they become official document records.
- Keep Admin, backup/restore, audit-log management, master data, user management, and full system configuration on desktop for v1.
- Keep the first release usable inside the office network without requiring internet or cloud hosting.

## Non-Goals

- No iPhone support in v1.
- No public no-login mobile viewer in v1.
- No Admin mobile screens in v1.
- No direct mobile approval into official records without desktop review.
- No outside-office access in v1.
- No app-store distribution requirement in v1.

## Architecture

The existing Windows office PC remains the source of truth for SQLite data and managed file storage. The Android app talks to a new small authenticated local-network API exposed by the office PC hub.

The desktop app continues to use existing Tauri IPC for desktop workflows. The local API is added only for Android workflows and should expose the smallest surface needed for Secretary mobile submissions.

Recommended high-level components:

1. React Native Android app.
2. Local office-network API hosted by the Windows hub.
3. Shared validation rules for document metadata where practical.
4. Mobile Submissions backend storage for uploaded files, metadata, status, review notes, reviewer, and timestamps.
5. Desktop Mobile Submissions review page for Secretary review and approval.

## Mobile UX

The Android app opens to a capture-first home after login.

Primary screens:

- Login: office PC address/status, username, password, office-Wi-Fi-required message.
- Capture Home: large camera action, secondary file upload action, recent draft/submission list.
- Metadata Wizard: full Add Document fields before upload.
- Attachment Review: preview, filename, retake/remove, upload readiness.
- Submit Result: pending-review confirmation, reference number, new-capture action.
- Submission History: pending, approved, rejected, failed-upload states.
- Settings: hub address, connection test, logout.

Required metadata before upload:

- Document title.
- Category.
- Folder, with category root allowed when the desktop workflow allows it.
- Sender office, with not specified allowed when the desktop workflow allows it.
- Date received.
- Status: Filed, Archived, Confidential, or Other.
- Remarks.
- One or more attachment files from camera capture or file picker.

UX rules:

- Save drafts locally until upload succeeds.
- Use large touch targets and bottom-pinned primary actions.
- Make required fields obvious.
- Show friendly network and validation errors.
- Do not expose passwords, tokens, internal paths, or raw backend errors in the UI.
- Confidential status must clearly warn that the document will be hidden from Staff/Head Viewer after approval.

## Workflow

1. Secretary opens the Android app on the office network.
2. Secretary logs in against the Windows hub.
3. Secretary captures with camera or chooses a file.
4. Secretary completes the full metadata wizard.
5. Android uploads metadata and attachment files to the hub.
6. The hub stores the item as a pending Mobile Submission.
7. Desktop Secretary opens the Mobile Submissions review page.
8. Desktop Secretary previews attachments and reviews or edits metadata.
9. Desktop Secretary approves the submission to create the official document record, or rejects it with a reason.
10. Android submission history shows pending, approved, rejected, or failed-upload state.

Approved submissions create official document records using the existing document rules. Rejected submissions remain visible in submission history with the rejection reason. A rejected submission is not edited in place; Android can duplicate it into a new local draft and submit a corrected copy.

## Desktop Review

Mobile submissions should not be mixed directly into the existing Scan Intake page. Scan Intake is mainly for raw scanned/imported files. Mobile submissions already include full metadata and need an approve/reject review flow.

Add a separate desktop Mobile Submissions page for Secretaries.

Desktop review actions:

- View pending mobile submissions.
- Preview attachments.
- Review metadata.
- Edit metadata before approval.
- Approve into official document record.
- Reject with a reason.
- View submission status and submitter.

## Backend And Security

The local API must be explicit and LAN-only by default. It should not expose Admin capabilities in v1.

Security requirements:

- Reuse existing Secretary credentials and session rules.
- Authenticate all mobile API requests.
- Authorize every endpoint by role.
- Validate file type, size, metadata, session, and role at the API boundary.
- Store uploaded files under controlled app storage, not arbitrary client paths.
- Prevent path traversal and unsafe filenames.
- Never log passwords, tokens, full uploaded file contents, or unnecessary PII. Use existing approved audit fields only.
- Audit login, upload, approve, reject, remove, and failed authorization events.
- Return safe user-facing errors to Android.

The implementation plan must decide the exact API host process and binding model. Acceptable candidates include a Rust local API embedded alongside the desktop hub or a small companion service started with the desktop app. The chosen approach must preserve the existing SQLite source of truth.

## Data Model

Mobile Submissions need persistent fields equivalent to:

- Submission ID.
- Submitter user ID.
- Metadata payload matching Add Document fields.
- Attachment records and stored relative paths.
- Status: Pending, Approved, Rejected, Removed, or Failed.
- Review notes or rejection reason.
- Reviewer user ID.
- Created, uploaded, reviewed, and updated timestamps.
- Resulting document ID after approval.

The exact schema belongs in the implementation plan and may require a migration. Any migration must include a rollback plan before implementation.

## Testing

Required verification for implementation:

- API validation tests for required fields, file types, size limits, role checks, and invalid sessions.
- Backend integration tests for upload, list, preview, approve, reject, remove, and audit events.
- Android tests for required metadata, local draft persistence, upload retry, submission history, and failure states.
- Desktop review tests for approving a mobile submission into an official document.
- Manual Android tests on office Wi-Fi for login, camera capture, file upload, preview, submit, reconnect, and logout.
- Security tests for unsupported files, oversized files, path traversal attempts, non-Secretary access, expired sessions, and LAN-only assumptions.

## Rollout

V1 rollout should use internal APK sideloading for office testing. The desktop workflow remains unchanged until a Mobile Submission is reviewed and approved.

Setup documentation must cover:

- Android APK installation.
- Office PC hub setup.
- Hub address discovery or manual entry.
- Office Wi-Fi requirement.
- Connection test.
- Recovery steps when the hub is offline.

## Later Phases

Phase 2 can add:

- Full mobile system parity.
- Authenticated Staff/Head mobile viewer access.
- iPhone support.
- Outside-office access through a proper server or secure remote-access model.
- Admin mobile features if there is a clear operational need.

## Risks

- Adding a LAN API changes the security boundary of a previously local desktop app.
- Mobile file upload and camera capture increase file-validation and storage risks.
- Full metadata on mobile improves review quality but can slow capture speed.
- The existing office PC must be reliably reachable on the office network.
- React Native adds a new build and testing surface to the project.

## Approval Summary

Kenneth approved these decisions on May 20, 2026:

- Build an actual Android app, not a PWA.
- Use React Native with TypeScript.
- Android only for v1.
- Office-network-only for v1.
- Existing Windows office PC is the hub.
- Secretary login only for v1.
- Capture-first mobile home.
- Camera capture and file upload are both supported.
- Full Add Document metadata is required before upload.
- Mobile submissions remain pending for desktop review.
- Use a separate Mobile Submissions page instead of mixing mobile items into Scan Intake.

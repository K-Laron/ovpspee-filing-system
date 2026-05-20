# Android Mobile V1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an Android-only React Native Secretary companion that uploads full-metadata mobile submissions to the existing Windows office PC for desktop review.

**Architecture:** The Windows Tauri app remains the SQLite and file-storage source of truth. A new Rust mobile-submissions domain module backs both desktop Tauri commands and a small explicit LAN HTTP API for Android. The Android app is a React Native TypeScript app that logs in, gathers full Add Document metadata, captures or chooses files, uploads to pending Mobile Submissions, and shows submission status.

**Tech Stack:** Existing Vite + React + TypeScript desktop, Tauri v2, Rust, SQLite, sqlx; new Rust HTTP API with `axum = "0.8.9"` and supporting dependencies; new React Native Android app with `react-native = "0.85.3"`.

---

## Approval Gates

Stop for Kenneth approval before executing these items:

- Dependency additions: Rust HTTP API dependencies, React Native Android dependencies, and Android local draft storage dependency.
- Public API surface: every `/api/mobile/*` route listed in Task 4.
- Database migration: `src-tauri/migrations/0005_mobile_submissions.sql`.
- New exported Rust/TypeScript types used by desktop or Android.

## Scope Check

This is one product feature with three implementation areas: hub backend/API, desktop review UI, and Android app. They are not independent releases because Android upload is useless without hub storage and desktop review. The plan is split into testable vertical tasks so each task can be reviewed and committed independently.

## File Structure

Create:

- `src-tauri/migrations/0005_mobile_submissions.sql`: mobile submission tables and indexes.
- `src-tauri/src/mobile_submissions.rs`: domain logic for validation, storage, listing, approval, rejection, and preview data.
- `src-tauri/src/mobile_api.rs`: LAN HTTP API router and handlers for Android.
- `src-tauri/tests/mobile_submissions_slice18.rs`: Rust integration tests for mobile submissions.
- `src-tauri/tests/mobile_api_slice18.rs`: Rust API tests for auth, uploads, and role checks.
- `src/pages/secretary/MobileSubmissions.tsx`: desktop Secretary review page.
- `src/pages/secretary/MobileSubmissions.test.tsx`: desktop review page tests.
- `mobile/android/`: React Native Android app.
- `mobile/android/src/api/client.ts`: Android API client.
- `mobile/android/src/types.ts`: Android shared types.
- `mobile/android/src/storage/drafts.ts`: local draft persistence.
- `mobile/android/src/screens/LoginScreen.tsx`: Android login.
- `mobile/android/src/screens/CaptureHomeScreen.tsx`: capture-first home.
- `mobile/android/src/screens/MetadataWizardScreen.tsx`: full metadata wizard.
- `mobile/android/src/screens/AttachmentReviewScreen.tsx`: file preview and submit.
- `mobile/android/src/screens/SubmissionHistoryScreen.tsx`: pending/approved/rejected history.
- `mobile/android/src/screens/SettingsScreen.tsx`: hub address and logout.
- `mobile/android/src/__tests__/mobile-flow.test.tsx`: Android app flow tests.
- `docs/android-mobile-v1-setup.md`: office setup and sideload guide.

Modify:

- `src-tauri/Cargo.toml`: add approved HTTP API dependencies.
- `src-tauri/src/lib.rs`: register `mobile_submissions` and start the optional hub API.
- `src-tauri/src/commands.rs`: add Tauri commands for desktop Mobile Submissions review.
- `src/types.ts`: add desktop `MobileSubmission*` types.
- `src/lib/invoke.ts`: add desktop mobile-submission invoke wrappers.
- `src/App.tsx`: add Secretary route `/s/mobile-submissions`.
- `src/components/layout/SecretaryLayout.tsx`: add Mobile Submissions nav item.
- `.gitignore`: add Android build outputs if needed after scaffolding.

## Task 1: Add Mobile Submissions Schema

**Files:**

- Create: `src-tauri/migrations/0005_mobile_submissions.sql`
- Test: `src-tauri/tests/mobile_submissions_slice18.rs`

- [ ] **Step 1: Write failing migration test**

Add this test skeleton:

```rust
use ovpspee_filing_system::db::create_test_pool;

#[tokio::test]
async fn mobile_submission_tables_exist_after_migration() {
    let pool = create_test_pool().await.expect("test pool");

    let submission_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'mobile_submission'",
    )
    .fetch_one(&pool)
    .await
    .expect("mobile_submission table query");

    let attachment_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'mobile_submission_attachment'",
    )
    .fetch_one(&pool)
    .await
    .expect("mobile_submission_attachment table query");

    assert_eq!(submission_count, 1);
    assert_eq!(attachment_count, 1);
}
```

- [ ] **Step 2: Run test and verify failure**

Run: `cargo test mobile_submission_tables_exist_after_migration`

Expected: fail because the tables do not exist.

- [ ] **Step 3: Add migration**

Create `src-tauri/migrations/0005_mobile_submissions.sql`:

```sql
CREATE TABLE IF NOT EXISTS mobile_submission (
    mobile_submission_id INTEGER PRIMARY KEY AUTOINCREMENT,
    submitted_by INTEGER NOT NULL REFERENCES user(user_id) ON DELETE RESTRICT,
    document_name TEXT NOT NULL,
    category_id INTEGER NOT NULL REFERENCES category(category_id) ON DELETE RESTRICT,
    folder_id INTEGER REFERENCES folder(folder_id) ON DELETE RESTRICT,
    office_id INTEGER REFERENCES office(office_id) ON DELETE RESTRICT,
    date_received TEXT NOT NULL,
    remarks TEXT,
    status TEXT NOT NULL CHECK(status IN ('Filed', 'Archived', 'Confidential', 'Other')),
    review_status TEXT NOT NULL DEFAULT 'Pending' CHECK(review_status IN ('Pending', 'Approved', 'Rejected', 'Removed')),
    rejection_reason TEXT,
    review_notes TEXT,
    reviewed_by INTEGER REFERENCES user(user_id) ON DELETE SET NULL,
    reviewed_at TEXT,
    resulting_document_id INTEGER REFERENCES document(document_id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS mobile_submission_attachment (
    mobile_submission_attachment_id INTEGER PRIMARY KEY AUTOINCREMENT,
    mobile_submission_id INTEGER NOT NULL REFERENCES mobile_submission(mobile_submission_id) ON DELETE CASCADE,
    original_file_name TEXT NOT NULL,
    stored_relative_path TEXT NOT NULL UNIQUE,
    mime_type TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_mobile_submission_review_status ON mobile_submission(review_status);
CREATE INDEX IF NOT EXISTS idx_mobile_submission_submitted_by ON mobile_submission(submitted_by);
CREATE INDEX IF NOT EXISTS idx_mobile_submission_result_document ON mobile_submission(resulting_document_id);
CREATE INDEX IF NOT EXISTS idx_mobile_submission_attachment_submission ON mobile_submission_attachment(mobile_submission_id);
```

Rollback plan for this migration if it has not reached production:

```sql
DROP TABLE IF EXISTS mobile_submission_attachment;
DROP TABLE IF EXISTS mobile_submission;
```

If the migration has reached production, do not drop tables automatically. Add a forward migration that marks Mobile Submissions disabled and preserves records for audit.

- [ ] **Step 4: Run test and verify pass**

Run: `cargo test mobile_submission_tables_exist_after_migration`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/migrations/0005_mobile_submissions.sql src-tauri/tests/mobile_submissions_slice18.rs
git commit -m "feat(db): add mobile submissions schema"
```

## Task 2: Implement Mobile Submissions Domain

**Files:**

- Create: `src-tauri/src/mobile_submissions.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/mobile_submissions_slice18.rs`

- [ ] **Step 1: Write failing domain test for Secretary-only creation**

Append:

```rust
use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    mobile_submissions::{create_mobile_submission, MobileSubmissionAttachmentUpload, MobileSubmissionInput},
    users::{create_user, UserInput},
};
use tempfile::tempdir;

#[tokio::test]
async fn secretary_can_create_mobile_submission_with_metadata_and_file() {
    let pool = create_test_pool().await.expect("test pool");
    create_first_admin(&pool, "Admin", "User", "admin1", "Admin123!").await.unwrap();
    let admin = authenticate_user(&pool, "admin1", "Admin123!").await.unwrap();
    create_user(&pool, &admin.session_id, UserInput {
        role: "Secretary".to_owned(),
        first_name: "Sec".to_owned(),
        middle_name: None,
        last_name: "User".to_owned(),
        username: "sec1".to_owned(),
        email: None,
        contact_number: None,
        address: None,
        password: Some("Secret123!".to_owned()),
        is_active: true,
    }).await.unwrap();
    let secretary = authenticate_user(&pool, "sec1", "Secret123!").await.unwrap();
    let dir = tempdir().unwrap();
    let storage = ovpspee_filing_system::documents::StorageRoot::new(dir.path().join("storage")).unwrap();
    let source = dir.path().join("sample.pdf");
    std::fs::write(&source, b"%PDF-1.4\nmobile").unwrap();

    let id = create_mobile_submission(
        &pool,
        &storage,
        &secretary.session_id,
        MobileSubmissionInput {
            document_name: "Mobile BAC memo".to_owned(),
            category_id: 1,
            folder_id: None,
            office_id: None,
            date_received: "2026-05-20".to_owned(),
            remarks: Some("Captured on Android".to_owned()),
            status: "Filed".to_owned(),
        },
        vec![MobileSubmissionAttachmentUpload {
            source_path: source.to_string_lossy().to_string(),
            original_file_name: "sample.pdf".to_owned(),
        }],
    )
    .await
    .unwrap();

    assert!(id > 0);
}
```

Expected initial failure: `mobile_submissions` module does not exist.

- [ ] **Step 2: Create module API**

Create `src-tauri/src/mobile_submissions.rs` with these public types and functions:

```rust
use std::{ffi::OsStr, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{require_session, write_audit_log},
    db::DbPool,
    documents::{
        mime_for_extension, now_text, require_len, trim_optional, validate_magic, DocumentInput,
        StorageRoot, MAX_ATTACHMENT_BYTES,
    },
    error::{AppError, AppResult},
};

#[derive(Debug, Clone, Deserialize)]
pub struct MobileSubmissionInput {
    pub document_name: String,
    pub category_id: i64,
    pub folder_id: Option<i64>,
    pub office_id: Option<i64>,
    pub date_received: String,
    pub remarks: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct MobileSubmissionAttachmentUpload {
    pub source_path: String,
    pub original_file_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MobileSubmissionItem {
    pub mobile_submission_id: i64,
    pub submitted_by: i64,
    pub submitter_name: String,
    pub document_name: String,
    pub category_id: i64,
    pub category_name: String,
    pub folder_id: Option<i64>,
    pub folder_name: Option<String>,
    pub office_id: Option<i64>,
    pub office_name: Option<String>,
    pub date_received: String,
    pub remarks: Option<String>,
    pub status: String,
    pub review_status: String,
    pub rejection_reason: Option<String>,
    pub review_notes: Option<String>,
    pub reviewed_by: Option<i64>,
    pub reviewed_at: Option<String>,
    pub resulting_document_id: Option<i64>,
    pub attachment_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MobileSubmissionAttachmentItem {
    pub mobile_submission_attachment_id: i64,
    pub mobile_submission_id: i64,
    pub original_file_name: String,
    pub stored_relative_path: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub sort_order: i64,
    pub created_at: String,
}

pub async fn create_mobile_submission(
    pool: &DbPool,
    storage: &StorageRoot,
    session_id: &str,
    input: MobileSubmissionInput,
    uploads: Vec<MobileSubmissionAttachmentUpload>,
) -> AppResult<i64> {
    let session = require_secretary(pool, session_id).await?;
    if uploads.is_empty() {
        return Err(AppError::Validation("At least one attachment is required.".into()));
    }
    let input = validate_mobile_input(pool, input).await?;
    let now = now_text();
    let result = sqlx::query!(
        "INSERT INTO mobile_submission
         (submitted_by, document_name, category_id, folder_id, office_id, date_received, remarks, status, review_status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'Pending', ?, ?)",
        session.user_id,
        input.document_name,
        input.category_id,
        input.folder_id,
        input.office_id,
        input.date_received,
        input.remarks,
        input.status,
        now,
        now
    )
    .execute(pool)
    .await?;
    let submission_id = result.last_insert_rowid();

    for (index, upload) in uploads.into_iter().enumerate() {
        store_mobile_attachment(pool, storage, submission_id, upload, index as i64 + 1).await?;
    }

    write_audit_log(
        pool,
        "MOBILE_UPLOAD",
        Some("mobile_submission"),
        Some(submission_id),
        "Created mobile submission",
        Some(session.user_id),
    )
    .await?;
    Ok(submission_id)
}

async fn require_secretary(pool: &DbPool, session_id: &str) -> AppResult<crate::auth::ValidSession> {
    let session = require_session(pool, session_id).await?;
    if session.role == "Secretary" {
        Ok(session)
    } else {
        Err(AppError::Unauthorized)
    }
}

async fn validate_mobile_input(pool: &DbPool, input: MobileSubmissionInput) -> AppResult<DocumentInput> {
    let normalized = DocumentInput {
        document_name: require_len(&input.document_name, "Document title", 200)?,
        category_id: input.category_id,
        folder_id: input.folder_id,
        office_id: input.office_id,
        date_received: input.date_received,
        remarks: trim_optional(input.remarks, 2000)?,
        status: input.status,
    };
    crate::documents::validate_document_input(pool, normalized).await
}

async fn store_mobile_attachment(
    pool: &DbPool,
    storage: &StorageRoot,
    submission_id: i64,
    upload: MobileSubmissionAttachmentUpload,
    sort_order: i64,
) -> AppResult<i64> {
    let source = crate::documents::validate_source_file(&upload.source_path)?;
    let ext = source
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    validate_magic(&source, &ext)?;
    let file_size = fs::metadata(&source)?.len();
    if file_size > MAX_ATTACHMENT_BYTES {
        return Err(AppError::Validation("Attachment exceeds 1 GB maximum.".into()));
    }
    let relative = format!("mobile-submissions/{submission_id}/{}.{}", Uuid::new_v4(), ext);
    let destination = storage.resolve_checked(&relative)?;
    fs::copy(&source, &destination)?;
    let mime_type = mime_for_extension(&ext).to_owned();
    let original = require_len(&upload.original_file_name, "File name", 255)?;
    let file_size_i64 = file_size as i64;

    let result = sqlx::query!(
        "INSERT INTO mobile_submission_attachment
         (mobile_submission_id, original_file_name, stored_relative_path, mime_type, file_size_bytes, sort_order)
         VALUES (?, ?, ?, ?, ?, ?)",
        submission_id,
        original,
        relative,
        mime_type,
        file_size_i64,
        sort_order
    )
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}
```

Also expose `validate_document_input` from `documents.rs`:

```rust
pub(crate) async fn validate_document_input(
    pool: &DbPool,
    input: DocumentInput,
) -> AppResult<DocumentInput> {
    // existing body unchanged
}
```

- [ ] **Step 3: Register module**

Modify `src-tauri/src/lib.rs`:

```rust
pub mod mobile_submissions;
```

- [ ] **Step 4: Run test and verify pass**

Run: `cargo test secretary_can_create_mobile_submission_with_metadata_and_file`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/mobile_submissions.rs src-tauri/src/lib.rs src-tauri/src/documents.rs src-tauri/tests/mobile_submissions_slice18.rs
git commit -m "feat(mobile): store mobile submissions"
```

## Task 3: Add Review, Approve, Reject Domain Logic

**Files:**

- Modify: `src-tauri/src/mobile_submissions.rs`
- Test: `src-tauri/tests/mobile_submissions_slice18.rs`

- [ ] **Step 1: Write failing approve/reject tests**

Add tests that create a pending submission, approve it, and assert `resulting_document_id` is set. Add a second test that rejects a pending submission and asserts `review_status = 'Rejected'` and no document is created.

Use these expected function names:

```rust
use ovpspee_filing_system::mobile_submissions::{
    approve_mobile_submission, get_mobile_submission, reject_mobile_submission,
};
```

Approve call shape:

```rust
let document_id = approve_mobile_submission(
    &pool,
    &storage,
    &secretary.session_id,
    submission_id,
    Some("Reviewed on desktop".to_owned()),
)
.await
.unwrap();
assert!(document_id > 0);
let approved = get_mobile_submission(&pool, &secretary.session_id, submission_id).await.unwrap();
assert_eq!(approved.submission.review_status, "Approved");
assert_eq!(approved.submission.resulting_document_id, Some(document_id));
```

Reject call shape:

```rust
reject_mobile_submission(
    &pool,
    &secretary.session_id,
    submission_id,
    "Wrong category".to_owned(),
)
.await
.unwrap();
let rejected = get_mobile_submission(&pool, &secretary.session_id, submission_id).await.unwrap();
assert_eq!(rejected.submission.review_status, "Rejected");
assert_eq!(rejected.submission.rejection_reason.as_deref(), Some("Wrong category"));
```

Expected: fail because functions are undefined.

- [ ] **Step 2: Implement detail, list, approve, reject**

Add:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct MobileSubmissionDetail {
    pub submission: MobileSubmissionItem,
    pub attachments: Vec<MobileSubmissionAttachmentItem>,
}

pub async fn list_mobile_submissions(
    pool: &DbPool,
    session_id: &str,
    review_status: Option<String>,
) -> AppResult<Vec<MobileSubmissionItem>> {
    require_secretary(pool, session_id).await?;
    let status_filter = review_status.filter(|value| !value.trim().is_empty());
    let rows = if let Some(status) = status_filter {
        sqlx::query_as!(
            MobileSubmissionItem,
            "SELECT ms.mobile_submission_id AS \"mobile_submission_id!: i64\",
                    ms.submitted_by AS \"submitted_by!: i64\",
                    (u.first_name || ' ' || u.last_name) AS \"submitter_name!: String\",
                    ms.document_name,
                    ms.category_id AS \"category_id!: i64\",
                    c.category_name,
                    ms.folder_id,
                    f.folder_name,
                    ms.office_id,
                    o.office_name,
                    ms.date_received,
                    ms.remarks,
                    ms.status,
                    ms.review_status,
                    ms.rejection_reason,
                    ms.review_notes,
                    ms.reviewed_by,
                    ms.reviewed_at,
                    ms.resulting_document_id,
                    COUNT(a.mobile_submission_attachment_id) AS \"attachment_count!: i64\",
                    ms.created_at,
                    ms.updated_at
             FROM mobile_submission ms
             JOIN user u ON u.user_id = ms.submitted_by
             JOIN category c ON c.category_id = ms.category_id
             LEFT JOIN folder f ON f.folder_id = ms.folder_id
             LEFT JOIN office o ON o.office_id = ms.office_id
             LEFT JOIN mobile_submission_attachment a ON a.mobile_submission_id = ms.mobile_submission_id
             WHERE ms.review_status = ?
             GROUP BY ms.mobile_submission_id
             ORDER BY ms.created_at DESC, ms.mobile_submission_id DESC",
            status
        )
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as!(
            MobileSubmissionItem,
            "SELECT ms.mobile_submission_id AS \"mobile_submission_id!: i64\",
                    ms.submitted_by AS \"submitted_by!: i64\",
                    (u.first_name || ' ' || u.last_name) AS \"submitter_name!: String\",
                    ms.document_name,
                    ms.category_id AS \"category_id!: i64\",
                    c.category_name,
                    ms.folder_id,
                    f.folder_name,
                    ms.office_id,
                    o.office_name,
                    ms.date_received,
                    ms.remarks,
                    ms.status,
                    ms.review_status,
                    ms.rejection_reason,
                    ms.review_notes,
                    ms.reviewed_by,
                    ms.reviewed_at,
                    ms.resulting_document_id,
                    COUNT(a.mobile_submission_attachment_id) AS \"attachment_count!: i64\",
                    ms.created_at,
                    ms.updated_at
             FROM mobile_submission ms
             JOIN user u ON u.user_id = ms.submitted_by
             JOIN category c ON c.category_id = ms.category_id
             LEFT JOIN folder f ON f.folder_id = ms.folder_id
             LEFT JOIN office o ON o.office_id = ms.office_id
             LEFT JOIN mobile_submission_attachment a ON a.mobile_submission_id = ms.mobile_submission_id
             GROUP BY ms.mobile_submission_id
             ORDER BY ms.created_at DESC, ms.mobile_submission_id DESC"
        )
            .fetch_all(pool)
            .await?
    };
    Ok(rows)
}
```

Implement `approve_mobile_submission` by:

1. Requiring Secretary role.
2. Loading pending submission and attachments.
3. Calling `documents::create_document`.
4. Copying each mobile attachment into `documents/{document_id}/mobile/{uuid}.{ext}`.
5. Inserting `attachment` rows.
6. Updating mobile submission to `Approved`.
7. Writing audit action `MOBILE_APPROVE`.

Implement `reject_mobile_submission` by:

1. Requiring Secretary role.
2. Requiring non-empty rejection reason up to 1000 chars.
3. Updating only pending submissions to `Rejected`.
4. Writing audit action `MOBILE_REJECT`.

- [ ] **Step 3: Run targeted tests**

Run: `cargo test mobile_submission`

Expected: pass all mobile-submission domain tests.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/mobile_submissions.rs src-tauri/tests/mobile_submissions_slice18.rs
git commit -m "feat(mobile): review submissions"
```

## Task 4: Expose Desktop Tauri Commands

**Files:**

- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/types.ts`
- Modify: `src/lib/invoke.ts`
- Test: `src-tauri/tests/mobile_submissions_slice18.rs`

- [ ] **Step 1: Write command-level test**

Add Rust tests that call domain functions through the same inputs used by commands. Keep command wrappers thin; domain tests cover behavior.

- [ ] **Step 2: Add command wrappers**

Add to `commands.rs`:

```rust
pub async fn list_mobile_submissions(
    db: State<'_, DbState>,
    session_id: String,
    review_status: Option<String>,
) -> Result<Vec<mobile_submissions::MobileSubmissionItem>, String> {
    mobile_submissions::list_mobile_submissions(&db.pool, &session_id, review_status)
        .await
        .map_err(|err| err.to_string())
}

pub async fn get_mobile_submission(
    db: State<'_, DbState>,
    session_id: String,
    mobile_submission_id: i64,
) -> Result<mobile_submissions::MobileSubmissionDetail, String> {
    mobile_submissions::get_mobile_submission(&db.pool, &session_id, mobile_submission_id)
        .await
        .map_err(|err| err.to_string())
}

pub async fn approve_mobile_submission(
    app: AppHandle,
    db: State<'_, DbState>,
    session_id: String,
    mobile_submission_id: i64,
    review_notes: Option<String>,
) -> Result<i64, String> {
    let storage = storage_root(&app)?;
    mobile_submissions::approve_mobile_submission(
        &db.pool,
        &storage,
        &session_id,
        mobile_submission_id,
        review_notes,
    )
    .await
    .map_err(|err| err.to_string())
}

pub async fn reject_mobile_submission(
    db: State<'_, DbState>,
    session_id: String,
    mobile_submission_id: i64,
    rejection_reason: String,
) -> Result<(), String> {
    mobile_submissions::reject_mobile_submission(
        &db.pool,
        &session_id,
        mobile_submission_id,
        rejection_reason,
    )
    .await
    .map_err(|err| err.to_string())
}
```

Register them in `tauri::generate_handler!`.

- [ ] **Step 3: Add desktop TypeScript types**

Add to `src/types.ts`:

```ts
export type MobileSubmissionReviewStatus = 'Pending' | 'Approved' | 'Rejected' | 'Removed';

export interface MobileSubmissionItem {
  mobile_submission_id: number;
  submitted_by: number;
  submitter_name: string;
  document_name: string;
  category_id: number;
  category_name: string;
  folder_id: number | null;
  folder_name: string | null;
  office_id: number | null;
  office_name: string | null;
  date_received: string;
  remarks: string | null;
  status: DocumentStatus;
  review_status: MobileSubmissionReviewStatus;
  rejection_reason: string | null;
  review_notes: string | null;
  reviewed_by: number | null;
  reviewed_at: string | null;
  resulting_document_id: number | null;
  attachment_count: number;
  created_at: string;
  updated_at: string;
}

export interface MobileSubmissionAttachmentItem {
  mobile_submission_attachment_id: number;
  mobile_submission_id: number;
  original_file_name: string;
  stored_relative_path: string;
  mime_type: string;
  file_size_bytes: number;
  sort_order: number;
  created_at: string;
}

export interface MobileSubmissionDetail {
  submission: MobileSubmissionItem;
  attachments: MobileSubmissionAttachmentItem[];
}
```

- [ ] **Step 4: Add invoke wrappers**

Add to `src/lib/invoke.ts`:

```ts
export const listMobileSubmissions = (params: {
  sessionId: string;
  reviewStatus?: MobileSubmissionReviewStatus | null;
}): Promise<MobileSubmissionItem[]> => invoke('list_mobile_submissions', params);

export const getMobileSubmission = (params: {
  sessionId: string;
  mobileSubmissionId: number;
}): Promise<MobileSubmissionDetail> => invoke('get_mobile_submission', params);

export const approveMobileSubmission = (params: {
  sessionId: string;
  mobileSubmissionId: number;
  reviewNotes: string | null;
}): Promise<number> => invoke('approve_mobile_submission', params);

export const rejectMobileSubmission = (params: {
  sessionId: string;
  mobileSubmissionId: number;
  rejectionReason: string;
}): Promise<void> => invoke('reject_mobile_submission', params);
```

Import `MobileSubmissionDetail`, `MobileSubmissionItem`, and `MobileSubmissionReviewStatus`.

- [ ] **Step 5: Run checks**

Run: `pnpm test -- --run src/test/smoke.test.ts`

Run: `cargo test mobile_submission`

Expected: pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src/types.ts src/lib/invoke.ts src-tauri/tests/mobile_submissions_slice18.rs
git commit -m "feat(mobile): expose review commands"
```

## Task 5: Build Desktop Mobile Submissions Review Page

**Files:**

- Create: `src/pages/secretary/MobileSubmissions.tsx`
- Create: `src/pages/secretary/MobileSubmissions.test.tsx`
- Modify: `src/App.tsx`
- Modify: `src/components/layout/SecretaryLayout.tsx`

- [ ] **Step 1: Write failing UI test**

Create a test that mocks `listMobileSubmissions`, renders the page with one pending submission, and asserts it shows document title, submitter, pending status, Approve, and Reject.

```tsx
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

vi.mock('../../lib/invoke', () => ({
  listMobileSubmissions: vi.fn().mockResolvedValue([
    {
      mobile_submission_id: 1,
      submitted_by: 2,
      submitter_name: 'Sec User',
      document_name: 'Mobile BAC memo',
      category_id: 1,
      category_name: 'BAC',
      folder_id: null,
      folder_name: null,
      office_id: null,
      office_name: null,
      date_received: '2026-05-20',
      remarks: 'Captured on Android',
      status: 'Filed',
      review_status: 'Pending',
      rejection_reason: null,
      review_notes: null,
      reviewed_by: null,
      reviewed_at: null,
      resulting_document_id: null,
      attachment_count: 1,
      created_at: '2026-05-20T08:00:00Z',
      updated_at: '2026-05-20T08:00:00Z'
    }
  ]),
  getMobileSubmission: vi.fn(),
  approveMobileSubmission: vi.fn(),
  rejectMobileSubmission: vi.fn()
}));

vi.mock('../../store/sessionStore', () => ({
  useSessionStore: () => 'session-1'
}));

describe('MobileSubmissions', () => {
  it('shows pending mobile submissions for review', async () => {
    const { MobileSubmissions } = await import('./MobileSubmissions');
    render(<MobileSubmissions />);
    expect(await screen.findByText('Mobile BAC memo')).toBeInTheDocument();
    expect(screen.getByText('Sec User')).toBeInTheDocument();
    expect(screen.getByText('Pending')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /approve/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /reject/i })).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Implement page**

Build `MobileSubmissions.tsx` with:

- Header and refresh button.
- Review-status filter.
- List of submissions.
- Detail panel with metadata.
- Attachment list.
- Approve button with optional notes.
- Reject dialog requiring rejection reason.

Use existing `ConfirmDialog`, `EmptyState`, `getUserErrorMessage`, and button classes.

- [ ] **Step 3: Wire route and nav**

In `src/App.tsx` add:

```tsx
<Route path="mobile-submissions" element={<MobileSubmissions />} />
```

In `SecretaryLayout.tsx`, add a Lucide icon such as `Smartphone`:

```tsx
{ label: 'Mobile Submissions', path: '/s/mobile-submissions', icon: Smartphone }
```

- [ ] **Step 4: Run UI tests**

Run: `pnpm test -- --run src/pages/secretary/MobileSubmissions.test.tsx`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add src/pages/secretary/MobileSubmissions.tsx src/pages/secretary/MobileSubmissions.test.tsx src/App.tsx src/components/layout/SecretaryLayout.tsx
git commit -m "feat(secretary): review mobile submissions"
```

## Task 6: Add LAN Mobile API

**Files:**

- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/mobile_api.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/mobile_api_slice18.rs`

- [ ] **Step 1: Stop for dependency and API approval**

Ask Kenneth to approve:

- Rust dependencies: `axum = "0.8.9"`, `tower-http = "0.6"`, `tokio-util = "0.7"`.
- LAN routes:
  - `GET /api/mobile/health`
  - `POST /api/mobile/login`
  - `POST /api/mobile/logout`
  - `GET /api/mobile/lookups`
  - `POST /api/mobile/submissions`
  - `GET /api/mobile/submissions`
  - `GET /api/mobile/submissions/:id`

Do not edit dependencies or expose the API until approved.

- [ ] **Step 2: Write failing API test**

Create a test that starts the router in-process and asserts health returns OK and unauthenticated submissions return 401.

```rust
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn mobile_api_requires_auth_for_submissions() {
    let pool = ovpspee_filing_system::db::create_test_pool().await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let storage = ovpspee_filing_system::documents::StorageRoot::new(dir.path().join("storage")).unwrap();
    let app = ovpspee_filing_system::mobile_api::router(pool, storage);

    let response = app
        .oneshot(Request::builder().uri("/api/mobile/submissions").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

- [ ] **Step 3: Add dependencies**

In `src-tauri/Cargo.toml` add approved dependencies:

```toml
axum = { version = "0.8.9", features = ["multipart"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
tokio-util = "0.7"
```

- [ ] **Step 4: Implement API router**

Create `mobile_api.rs`:

```rust
use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth,
    db::DbPool,
    documents::StorageRoot,
    error::AppError,
    mobile_submissions::{self, MobileSubmissionAttachmentUpload, MobileSubmissionInput},
};

#[derive(Clone)]
pub struct MobileApiState {
    pub pool: DbPool,
    pub storage: StorageRoot,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub error: String,
}

pub fn router(pool: DbPool, storage: StorageRoot) -> Router {
    Router::new()
        .route("/api/mobile/health", get(health))
        .route("/api/mobile/login", post(login))
        .route("/api/mobile/logout", post(logout))
        .route("/api/mobile/lookups", get(lookups))
        .route("/api/mobile/submissions", get(list_submissions).post(create_submission))
        .route("/api/mobile/submissions/{id}", get(get_submission))
        .with_state(MobileApiState { pool, storage })
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn login(
    State(state): State<MobileApiState>,
    Json(input): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let session = auth::authenticate_user(&state.pool, &input.username, &input.password).await?;
    if session.role != "Secretary" {
        return Err(ApiError(AppError::Unauthorized));
    }
    Ok(Json(session))
}

fn bearer_session(headers: &HeaderMap) -> Result<String, ApiError> {
    let value = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError(AppError::Unauthorized))?;
    value
        .strip_prefix("Bearer ")
        .map(str::to_owned)
        .ok_or(ApiError(AppError::Unauthorized))
}
```

Add the remaining handlers with this shape:

```rust
async fn auth_required(headers: &HeaderMap, state: &MobileApiState) -> Result<String, ApiError> {
    let session_id = bearer_session(headers)?;
    let session = auth::validate_session(&state.pool, &session_id).await?;
    if session.role != "Secretary" {
        return Err(ApiError(AppError::Unauthorized));
    }
    Ok(session_id)
}

async fn list_submissions(
    State(state): State<MobileApiState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = auth_required(&headers, &state).await?;
    let rows = mobile_submissions::list_mobile_submissions(&state.pool, &session_id, None).await?;
    Ok(Json(rows))
}

async fn get_submission(
    State(state): State<MobileApiState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = auth_required(&headers, &state).await?;
    let detail = mobile_submissions::get_mobile_submission(&state.pool, &session_id, id).await?;
    Ok(Json(detail))
}

async fn create_submission(
    State(state): State<MobileApiState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = auth_required(&headers, &state).await?;
    let mut input: Option<MobileSubmissionInput> = None;
    let mut uploads = Vec::new();
    let temp_dir = state.storage.resolve_checked("mobile-submissions/tmp")?;

    while let Some(field) = multipart.next_field().await.map_err(|_| {
        ApiError(AppError::Validation("Invalid mobile upload payload.".into()))
    })? {
        let name = field.name().unwrap_or_default().to_owned();
        if name == "metadata" {
            let text = field.text().await.map_err(|_| {
                ApiError(AppError::Validation("Invalid mobile metadata.".into()))
            })?;
            input = Some(serde_json::from_str::<MobileSubmissionInput>(&text)?);
            continue;
        }
        if name == "files" {
            let file_name = field.file_name().unwrap_or("mobile-upload.bin").to_owned();
            let bytes = field.bytes().await.map_err(|_| {
                ApiError(AppError::Validation("Invalid mobile upload file.".into()))
            })?;
            let temp_path = temp_dir.join(format!("{}-{}", uuid::Uuid::new_v4(), file_name));
            tokio::fs::write(&temp_path, bytes).await?;
            uploads.push(MobileSubmissionAttachmentUpload {
                source_path: temp_path.to_string_lossy().to_string(),
                original_file_name: file_name,
            });
        }
    }

    let id = mobile_submissions::create_mobile_submission(
        &state.pool,
        &state.storage,
        &session_id,
        input.ok_or_else(|| ApiError(AppError::Validation("Metadata is required.".into())))?,
        uploads,
    )
    .await?;
    Ok(Json(serde_json::json!({ "mobile_submission_id": id })))
}

pub struct ApiError(AppError);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.0 {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) | AppError::Conflict(_) | AppError::Duplicate(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let safe = if status == StatusCode::INTERNAL_SERVER_ERROR {
            "Something went wrong. Please try again.".to_owned()
        } else {
            self.0.to_string()
        };
        (status, Json(ApiErrorBody { error: safe })).into_response()
    }
}
```

- [ ] **Step 5: Start API explicitly**

In `lib.rs`, start the API only when enabled. Use an environment variable for v1:

```rust
if std::env::var("OVPSPEE_MOBILE_API_ENABLED").as_deref() == Ok("1") {
    let pool = handle.clone();
    let storage = StorageRoot::new(app_data_dir.join("storage"))?;
    tauri::async_runtime::spawn(async move {
        mobile_api::serve(pool, storage, "0.0.0.0:1421").await;
    });
}
```

The implementation should log only that the mobile API started; do not log secrets or request bodies.

- [ ] **Step 6: Run API tests**

Run: `cargo test mobile_api`

Expected: pass.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/mobile_api.rs src-tauri/src/lib.rs src-tauri/tests/mobile_api_slice18.rs
git commit -m "feat(mobile): add LAN hub API"
```

## Task 7: Scaffold React Native Android App

**Files:**

- Create: `mobile/android/`
- Modify: `.gitignore`

- [ ] **Step 1: Stop for React Native dependency approval**

Ask Kenneth to approve creating the React Native app under `mobile/android` with `react-native = "0.85.3"`.

- [ ] **Step 2: Scaffold app**

Run:

```powershell
npx @react-native-community/cli@20.1.3 init OVPSPEEMobile --version 0.85.3 --directory mobile/android
```

Expected: React Native project is created in `mobile/android`.

- [ ] **Step 3: Add app build outputs to `.gitignore`**

Add:

```gitignore
mobile/android/android/.gradle/
mobile/android/android/app/build/
mobile/android/android/build/
mobile/android/node_modules/
```

- [ ] **Step 4: Run Android unit test baseline**

Run:

```powershell
cd mobile/android
npm test -- --runInBand
```

Expected: generated baseline tests pass.

- [ ] **Step 5: Commit**

```bash
git add .gitignore mobile/android
git commit -m "build(mobile): scaffold Android app"
```

## Task 8: Build Android API Client And Draft Storage

**Files:**

- Create: `mobile/android/src/api/client.ts`
- Create: `mobile/android/src/types.ts`
- Create: `mobile/android/src/storage/drafts.ts`
- Test: `mobile/android/src/__tests__/api-client.test.ts`

- [ ] **Step 1: Write failing client tests**

Test that `login` POSTs to `/api/mobile/login`, stores no password, and `createSubmission` sends `Authorization: Bearer <session>`.

- [ ] **Step 2: Add shared types**

Create types matching the hub API:

```ts
export type DocumentStatus = 'Filed' | 'Archived' | 'Confidential' | 'Other';
export type ReviewStatus = 'Pending' | 'Approved' | 'Rejected' | 'Removed';

export interface SessionPayload {
  session_id: string;
  user_id: number;
  role: 'Secretary';
  display_name: string;
  profile_pic_path: string | null;
}

export interface MobileSubmissionDraft {
  documentName: string;
  categoryId: number | null;
  folderId: number | null;
  officeId: number | null;
  dateReceived: string;
  remarks: string;
  status: DocumentStatus;
  attachments: Array<{ uri: string; name: string; type: string }>;
}
```

- [ ] **Step 3: Implement API client**

Use `fetch`. Centralize safe error handling:

```ts
export class ApiClient {
  constructor(private readonly baseUrl: string) {}

  async login(username: string, password: string): Promise<SessionPayload> {
    const response = await fetch(`${this.baseUrl}/api/mobile/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    });
    return this.parse<SessionPayload>(response, 'Login failed. Check your account and office Wi-Fi.');
  }

  async createSubmission(sessionId: string, draft: MobileSubmissionDraft): Promise<{ mobile_submission_id: number }> {
    const data = new FormData();
    data.append('metadata', JSON.stringify({
      document_name: draft.documentName,
      category_id: draft.categoryId,
      folder_id: draft.folderId,
      office_id: draft.officeId,
      date_received: draft.dateReceived,
      remarks: draft.remarks || null,
      status: draft.status
    }));
    draft.attachments.forEach((file) => {
      data.append('files', { uri: file.uri, name: file.name, type: file.type } as never);
    });
    const response = await fetch(`${this.baseUrl}/api/mobile/submissions`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${sessionId}` },
      body: data
    });
    return this.parse(response, 'Could not submit. Check the office PC connection and try again.');
  }

  private async parse<T>(response: Response, fallback: string): Promise<T> {
    if (!response.ok) {
      throw new Error(fallback);
    }
    return response.json() as Promise<T>;
  }
}
```

- [ ] **Step 4: Stop for local-storage dependency approval**

Ask Kenneth to approve adding `@react-native-async-storage/async-storage` for local draft persistence. Do not add it until approved.

- [ ] **Step 5: Install and implement draft storage**

Run after approval:

```powershell
cd mobile/android
npm install @react-native-async-storage/async-storage
```

Create `mobile/android/src/storage/drafts.ts`:

```ts
import AsyncStorage from '@react-native-async-storage/async-storage';
import type { MobileSubmissionDraft } from '../types';

const DRAFT_KEY = 'ovpspee.mobileSubmissionDraft.v1';

export const saveDraft = async (draft: MobileSubmissionDraft): Promise<void> => {
  await AsyncStorage.setItem(DRAFT_KEY, JSON.stringify(draft));
};

export const loadDraft = async (): Promise<MobileSubmissionDraft | null> => {
  const value = await AsyncStorage.getItem(DRAFT_KEY);
  if (!value) return null;
  return JSON.parse(value) as MobileSubmissionDraft;
};

export const clearDraft = async (): Promise<void> => {
  await AsyncStorage.removeItem(DRAFT_KEY);
};
```

- [ ] **Step 6: Run Android client tests**

Run: `cd mobile/android && npm test -- api-client`

Expected: pass.

- [ ] **Step 7: Commit**

```bash
git add mobile/android/src/api mobile/android/src/types.ts mobile/android/src/storage mobile/android/src/__tests__
git commit -m "feat(mobile): add Android API client"
```

## Task 9: Build Android Screens

**Files:**

- Create/modify Android screen files under `mobile/android/src/screens/`
- Test: `mobile/android/src/__tests__/mobile-flow.test.tsx`

- [ ] **Step 1: Write failing flow test**

Test login, capture home, required metadata validation, attachment review, submit success, and history state.

- [ ] **Step 2: Implement Login screen**

Fields:

- Hub URL.
- Username.
- Password.
- Connection test.
- Login.

Rules:

- Password is never persisted.
- Hub URL can be persisted.
- Errors use safe text.

- [ ] **Step 3: Implement Capture Home**

Actions:

- Primary camera capture.
- Secondary file upload.
- Recent drafts/submissions.

Use large touch targets and bottom action placement.

- [ ] **Step 4: Implement Metadata Wizard**

Fields exactly match v1 required metadata:

- title
- category
- folder
- sender office
- date received
- status
- remarks

Confidential status shows warning text.

- [ ] **Step 5: Implement Attachment Review and Submit Result**

Show selected attachments, retake/remove, and submit button. On success, clear local draft and show pending review reference.

- [ ] **Step 6: Implement Submission History and Settings**

Show pending/approved/rejected states and rejection reason. Settings includes hub URL, connection test, and logout.

- [ ] **Step 7: Run Android tests**

Run: `cd mobile/android && npm test -- mobile-flow`

Expected: pass.

- [ ] **Step 8: Commit**

```bash
git add mobile/android/src/screens mobile/android/src/__tests__
git commit -m "feat(mobile): add Android submission flow"
```

## Task 10: Documentation And End-To-End Verification

**Files:**

- Create: `docs/android-mobile-v1-setup.md`
- Modify: `docs/00_Index.md`

- [ ] **Step 1: Write setup doc**

Include:

- Android APK install steps.
- How to enable the mobile API on the office PC.
- Office Wi-Fi requirement.
- Hub address format: `http://<office-pc-ip>:1421`.
- Connection test.
- Troubleshooting hub offline, invalid login, unsupported file, oversized file.

- [ ] **Step 2: Link docs from index**

Add `docs/android-mobile-v1-setup.md` to `docs/00_Index.md`.

- [ ] **Step 3: Run full checks**

Run:

```powershell
pnpm test
pnpm build
cd src-tauri
cargo test
cd ..\mobile\android
npm test -- --runInBand
```

Expected: all pass.

- [ ] **Step 4: Manual Android office-network checks**

On an Android device connected to office Wi-Fi:

- Install APK.
- Set hub URL.
- Test connection.
- Login as Secretary.
- Capture with camera.
- Fill full metadata.
- Submit.
- Open desktop Mobile Submissions.
- Approve submission.
- Confirm official document appears in Documents.
- Reject a second submission and confirm Android shows rejection reason.

- [ ] **Step 5: Commit**

```bash
git add docs/android-mobile-v1-setup.md docs/00_Index.md
git commit -m "docs: add Android mobile setup guide"
```

## Final Verification

Before claiming complete:

```powershell
git status --short
pnpm test
pnpm build
cd src-tauri
cargo test
cd ..\mobile\android
npm test -- --runInBand
```

Report exact commands run and any skipped checks with reasons.

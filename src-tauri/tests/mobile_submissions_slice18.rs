use std::{
    fs,
    path::{Path, PathBuf},
};

use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    documents::StorageRoot,
    master_data::{
        create_category, create_folder, create_office, CategoryInput, FolderInput, OfficeInput,
    },
    mobile_submissions::{
        approve_mobile_submission, create_mobile_submission, get_mobile_submission,
        get_mobile_submission_attachment_preview_page, reject_mobile_submission,
        MobileSubmissionAttachmentUpload, MobileSubmissionInput,
    },
    users::{create_user, UserInput},
};
use uuid::Uuid;

struct Fixture {
    pool: DbPool,
    secretary: String,
    category_id: i64,
    folder_id: i64,
    office_id: i64,
    storage: StorageRoot,
    source_dir: PathBuf,
}

async fn fixture() -> Fixture {
    let pool = create_test_pool().await.expect("pool");
    create_first_admin(&pool, "Admin", "User", "admin1", "Admin123!")
        .await
        .expect("admin created");
    let admin = authenticate_user(&pool, "admin1", "Admin123!")
        .await
        .expect("admin login")
        .session_id;
    create_user(
        &pool,
        &admin,
        UserInput {
            role: "Secretary".to_owned(),
            first_name: "Sec".to_owned(),
            middle_name: None,
            last_name: "User".to_owned(),
            username: "sec1".to_owned(),
            email: None,
            contact_number: None,
            address: None,
            password: "Secret123!".to_owned(),
        },
    )
    .await
    .expect("secretary created");
    let secretary = authenticate_user(&pool, "sec1", "Secret123!")
        .await
        .expect("secretary login")
        .session_id;
    let category_id = create_category(
        &pool,
        &admin,
        CategoryInput {
            category_name: "Mobile Incoming".to_owned(),
            description: None,
            color_code: "#2563EB".to_owned(),
            icon: Some("Folder".to_owned()),
        },
    )
    .await
    .expect("category");
    let folder_id = create_folder(
        &pool,
        &admin,
        FolderInput {
            category_id,
            folder_name: "Android".to_owned(),
            description: None,
            folder_color: "#0F766E".to_owned(),
        },
    )
    .await
    .expect("folder");
    let office_id = create_office(
        &pool,
        &admin,
        OfficeInput {
            office_name: "OVPSPEE".to_owned(),
            description: None,
        },
    )
    .await
    .expect("office");
    let root = std::env::temp_dir().join(format!("ovpspee-mobile-{}", Uuid::new_v4()));
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).expect("source dir");
    let storage = StorageRoot::new(root.join("storage")).expect("storage");

    Fixture {
        pool,
        secretary,
        category_id,
        folder_id,
        office_id,
        storage,
        source_dir,
    }
}

fn input(fx: &Fixture) -> MobileSubmissionInput {
    MobileSubmissionInput {
        client_submission_id: Some("mobile-client-1".to_owned()),
        device_id: Some("device-1".to_owned()),
        device_name: Some("Records phone".to_owned()),
        document_name: "Mobile BAC memo".to_owned(),
        category_id: fx.category_id,
        folder_id: Some(fx.folder_id),
        office_id: Some(fx.office_id),
        date_received: "2026-05-20".to_owned(),
        remarks: Some("Captured on Android".to_owned()),
        status: "Filed".to_owned(),
    }
}

fn write_file(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, bytes).expect("source file");
    path
}

async fn create_pending(fx: &Fixture) -> i64 {
    let source = write_file(&fx.source_dir, "sample.pdf", b"%PDF-1.4\nmobile");
    create_mobile_submission(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        input(fx),
        vec![MobileSubmissionAttachmentUpload {
            source_path: source.to_string_lossy().into_owned(),
            original_file_name: "sample.pdf".to_owned(),
        }],
    )
    .await
    .expect("submission")
}

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

#[tokio::test]
async fn secretary_can_create_mobile_submission_with_metadata_and_file() {
    let fx = fixture().await;

    let id = create_pending(&fx).await;
    let detail = get_mobile_submission(&fx.pool, &fx.secretary, id)
        .await
        .expect("detail");

    assert_eq!(detail.submission.review_status, "Pending");
    assert_eq!(
        detail.submission.client_submission_id.as_deref(),
        Some("mobile-client-1")
    );
    assert_eq!(
        detail.submission.submitted_device_name.as_deref(),
        Some("Records phone")
    );
    assert_eq!(detail.attachments.len(), 1);
    assert!(fx
        .storage
        .resolve_relative(&detail.attachments[0].stored_relative_path)
        .exists());
}

#[tokio::test]
async fn duplicate_client_submission_id_returns_existing_pending_submission() {
    let fx = fixture().await;
    let source = write_file(&fx.source_dir, "duplicate.pdf", b"%PDF-1.4\nmobile");
    let upload = || MobileSubmissionAttachmentUpload {
        source_path: source.to_string_lossy().into_owned(),
        original_file_name: "duplicate.pdf".to_owned(),
    };

    let first_id = create_mobile_submission(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        input(&fx),
        vec![upload()],
    )
    .await
    .expect("first submission");
    let second_id = create_mobile_submission(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        input(&fx),
        vec![upload()],
    )
    .await
    .expect("idempotent duplicate");

    assert_eq!(first_id, second_id);
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mobile_submission WHERE client_submission_id = 'mobile-client-1'",
    )
    .fetch_one(&fx.pool)
    .await
    .expect("count duplicate client submission rows");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn approve_mobile_submission_creates_document_and_marks_approved() {
    let fx = fixture().await;
    let submission_id = create_pending(&fx).await;

    let document_id = approve_mobile_submission(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        submission_id,
        Some("Reviewed on desktop".to_owned()),
    )
    .await
    .expect("approved");
    let approved = get_mobile_submission(&fx.pool, &fx.secretary, submission_id)
        .await
        .expect("approved detail");

    assert!(document_id > 0);
    assert_eq!(approved.submission.review_status, "Approved");
    assert_eq!(approved.submission.resulting_document_id, Some(document_id));
}

#[tokio::test]
async fn reject_mobile_submission_marks_rejected_without_document() {
    let fx = fixture().await;
    let submission_id = create_pending(&fx).await;

    reject_mobile_submission(
        &fx.pool,
        &fx.secretary,
        submission_id,
        "Wrong category".to_owned(),
    )
    .await
    .expect("rejected");
    let rejected = get_mobile_submission(&fx.pool, &fx.secretary, submission_id)
        .await
        .expect("rejected detail");

    assert_eq!(rejected.submission.review_status, "Rejected");
    assert_eq!(
        rejected.submission.rejection_reason.as_deref(),
        Some("Wrong category")
    );
    assert_eq!(rejected.submission.resulting_document_id, None);
}

#[tokio::test]
async fn mobile_submission_attachment_preview_returns_file_path() {
    let fx = fixture().await;
    let submission_id = create_pending(&fx).await;
    let detail = get_mobile_submission(&fx.pool, &fx.secretary, submission_id)
        .await
        .expect("detail");

    let preview = get_mobile_submission_attachment_preview_page(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        detail.attachments[0].mobile_submission_attachment_id,
        Some(1),
    )
    .await
    .expect("preview");

    assert_eq!(preview.info.preview_kind, "Pdf");
    assert!(preview.info.file_exists);
    assert!(preview.file_path.is_some());
}

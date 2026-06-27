use std::{fs, path::PathBuf};

use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    documents::{
        add_attachment, create_document, get_public_document, list_documents, move_document,
        set_document_status, trash_document, AttachmentInput, DocumentInput, DocumentListFilter,
        StorageRoot,
    },
    master_data::{
        create_category, create_folder, create_office, CategoryInput, FolderInput, OfficeInput,
    },
    users::{create_user, UserInput},
};
use uuid::Uuid;

struct Fixture {
    pool: DbPool,
    admin: String,
    secretary: String,
    category_id: i64,
    folder_id: i64,
    other_category_id: i64,
    other_folder_id: i64,
    office_id: i64,
    storage: StorageRoot,
    source_dir: PathBuf,
}

async fn fixture() -> Fixture {
    let pool = create_test_pool().await.expect("pool");
    create_first_admin(&pool, "Kenneth", "Laron", "admin", "Valid123!")
        .await
        .expect("admin created");
    let admin = authenticate_user(&pool, "admin", "Valid123!")
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
            username: "secretary".to_owned(),
            email: None,
            contact_number: None,
            address: None,
            password: "Valid123!".to_owned(),
        },
    )
    .await
    .expect("secretary created");
    let secretary = authenticate_user(&pool, "secretary", "Valid123!")
        .await
        .expect("secretary login")
        .session_id;
    let category_id = create_category(&pool, &admin, category("Incoming"))
        .await
        .expect("category");
    let folder_id = create_folder(&pool, &admin, folder(category_id, "Memos"))
        .await
        .expect("folder");
    let other_category_id = create_category(&pool, &admin, category("Archive"))
        .await
        .expect("other category");
    let other_folder_id = create_folder(&pool, &admin, folder(other_category_id, "Closed"))
        .await
        .expect("other folder");
    let office_id = create_office(&pool, &admin, office("OVPSPEE"))
        .await
        .expect("office");
    let root = std::env::temp_dir().join(format!("ovpspee-slice6-{}", Uuid::new_v4()));
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).expect("source dir");
    let storage = StorageRoot::new(root.join("storage")).expect("storage");
    Fixture {
        pool,
        admin,
        secretary,
        category_id,
        folder_id,
        other_category_id,
        other_folder_id,
        office_id,
        storage,
        source_dir,
    }
}

fn category(name: &str) -> CategoryInput {
    CategoryInput {
        category_name: name.to_owned(),
        description: None,
        color_code: "#2563EB".to_owned(),
        icon: Some("Folder".to_owned()),
    }
}

fn folder(category_id: i64, name: &str) -> FolderInput {
    FolderInput {
        category_id,
        folder_name: name.to_owned(),
        description: None,
        folder_color: "#0F766E".to_owned(),
    }
}

fn office(name: &str) -> OfficeInput {
    OfficeInput {
        office_name: name.to_owned(),
        description: None,
    }
}

fn doc(fx: &Fixture, title: &str) -> DocumentInput {
    DocumentInput {
        document_name: title.to_owned(),
        category_id: fx.category_id,
        folder_id: Some(fx.folder_id),
        office_id: Some(fx.office_id),
        date_received: "2026-05-14".to_owned(),
        remarks: Some("Slice 6".to_owned()),
        status: "Filed".to_owned(),
    }
}

async fn create_doc(fx: &Fixture, title: &str) -> i64 {
    create_document(&fx.pool, &fx.secretary, doc(fx, title))
        .await
        .expect("document")
}

async fn fetch_doc(fx: &Fixture, id: i64) -> ovpspee_filing_system::documents::DocumentItem {
    list_documents(&fx.pool, &fx.secretary, DocumentListFilter::default())
        .await
        .expect("docs")
        .documents
        .into_iter()
        .find(|row| row.document_id == id)
        .expect("document row")
}

#[tokio::test]
async fn secretary_can_move_document_to_another_category() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Move Category").await;

    move_document(&fx.pool, &fx.secretary, id, fx.other_category_id, None)
        .await
        .expect("move");

    let moved = fetch_doc(&fx, id).await;
    assert_eq!(moved.category_id, fx.other_category_id);
    assert_eq!(moved.folder_id, None);
}

#[tokio::test]
async fn secretary_can_move_document_to_folder_under_selected_category() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Move Folder").await;

    move_document(
        &fx.pool,
        &fx.secretary,
        id,
        fx.other_category_id,
        Some(fx.other_folder_id),
    )
    .await
    .expect("move");

    let moved = fetch_doc(&fx, id).await;
    assert_eq!(moved.category_id, fx.other_category_id);
    assert_eq!(moved.folder_id, Some(fx.other_folder_id));
}

#[tokio::test]
async fn secretary_can_move_document_to_category_root() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Move Root").await;

    move_document(&fx.pool, &fx.secretary, id, fx.category_id, None)
        .await
        .expect("move root");

    assert_eq!(fetch_doc(&fx, id).await.folder_id, None);
}

#[tokio::test]
async fn move_rejects_folder_category_mismatch() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Mismatch").await;

    assert!(move_document(
        &fx.pool,
        &fx.secretary,
        id,
        fx.category_id,
        Some(fx.other_folder_id)
    )
    .await
    .is_err());
}

#[tokio::test]
async fn move_rejects_inactive_or_missing_category() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Bad Category").await;
    sqlx::query!(
        "UPDATE category SET is_active = 0 WHERE category_id = ?",
        fx.other_category_id
    )
    .execute(&fx.pool)
    .await
    .expect("deactivate category");

    assert!(
        move_document(&fx.pool, &fx.secretary, id, fx.other_category_id, None)
            .await
            .is_err()
    );
    assert!(move_document(&fx.pool, &fx.secretary, id, 99_999, None)
        .await
        .is_err());
}

#[tokio::test]
async fn move_rejects_inactive_or_missing_folder() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Bad Folder").await;
    sqlx::query!(
        "UPDATE folder SET is_active = 0 WHERE folder_id = ?",
        fx.other_folder_id
    )
    .execute(&fx.pool)
    .await
    .expect("deactivate folder");

    assert!(move_document(
        &fx.pool,
        &fx.secretary,
        id,
        fx.other_category_id,
        Some(fx.other_folder_id)
    )
    .await
    .is_err());
    assert!(move_document(
        &fx.pool,
        &fx.secretary,
        id,
        fx.other_category_id,
        Some(99_999)
    )
    .await
    .is_err());
}

#[tokio::test]
async fn move_rejects_direct_trash_category_and_trashed_documents() {
    let fx = fixture().await;
    let id = create_doc(&fx, "No Trash Move").await;
    let trash_id = sqlx::query!(
        "SELECT category_id AS \"category_id!: i64\" FROM category WHERE category_name = 'TRASH'"
    )
    .fetch_one(&fx.pool)
    .await
    .expect("trash")
    .category_id;

    assert!(move_document(&fx.pool, &fx.secretary, id, trash_id, None)
        .await
        .is_err());

    trash_document(&fx.pool, &fx.secretary, id)
        .await
        .expect("trash");
    assert!(
        move_document(&fx.pool, &fx.secretary, id, fx.other_category_id, None)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn move_does_not_move_attachment_files_on_disk() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Attachment Stays").await;
    let source = fx.source_dir.join("stay.pdf");
    fs::write(&source, b"%PDF-1.4\n").expect("source");
    add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        id,
        AttachmentInput {
            source_path: source.to_string_lossy().into_owned(),
            sort_order: Some(1),
        },
    )
    .await
    .expect("attach");
    let before = sqlx::query!(
        "SELECT stored_relative_path FROM attachment WHERE document_id = ?",
        id
    )
    .fetch_one(&fx.pool)
    .await
    .expect("attachment")
    .stored_relative_path;

    move_document(
        &fx.pool,
        &fx.secretary,
        id,
        fx.other_category_id,
        Some(fx.other_folder_id),
    )
    .await
    .expect("move");

    let after = sqlx::query!(
        "SELECT stored_relative_path FROM attachment WHERE document_id = ?",
        id
    )
    .fetch_one(&fx.pool)
    .await
    .expect("attachment")
    .stored_relative_path;
    assert_eq!(before, after);
    assert!(fx.storage.resolve_relative(&after).exists());
}

#[tokio::test]
async fn viewer_and_admin_cannot_call_normal_move_command() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Blocked Move").await;

    assert!(move_document(&fx.pool, "", id, fx.other_category_id, None)
        .await
        .is_err());
    assert!(
        move_document(&fx.pool, &fx.admin, id, fx.other_category_id, None)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn secretary_can_set_valid_statuses() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Statuses").await;

    set_document_status(&fx.pool, &fx.secretary, id, "Filed".to_owned())
        .await
        .expect("filed");
    assert_eq!(fetch_doc(&fx, id).await.status, "Filed");

    set_document_status(&fx.pool, &fx.secretary, id, "Archived".to_owned())
        .await
        .expect("archived");
    assert_eq!(fetch_doc(&fx, id).await.status, "Archived");
}

#[tokio::test]
async fn confidential_status_hides_document_from_viewer() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Confidential Status").await;

    set_document_status(&fx.pool, &fx.secretary, id, "Confidential".to_owned())
        .await
        .expect("confidential");

    let row = fetch_doc(&fx, id).await;
    assert!(row.is_hidden);
    assert!(get_public_document(&fx.pool, id).await.is_err());
}

#[tokio::test]
async fn invalid_status_is_rejected() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Bad Status").await;

    assert!(
        set_document_status(&fx.pool, &fx.secretary, id, "Pending".to_owned())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn viewer_and_admin_cannot_call_normal_status_command() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Blocked Status").await;

    assert!(set_document_status(&fx.pool, "", id, "Archived".to_owned())
        .await
        .is_err());
    assert!(
        set_document_status(&fx.pool, &fx.admin, id, "Archived".to_owned())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn status_and_move_write_audit_logs() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Audit Move Status").await;

    move_document(&fx.pool, &fx.secretary, id, fx.other_category_id, None)
        .await
        .expect("move");
    set_document_status(&fx.pool, &fx.secretary, id, "Archived".to_owned())
        .await
        .expect("status");

    let count = sqlx::query!(
        "SELECT COUNT(*) AS \"count!: i64\" FROM audit_log
         WHERE table_affected = 'document' AND record_id = ?
           AND (log_action = 'MOVE' OR description LIKE 'Changed document status%')",
        id
    )
    .fetch_one(&fx.pool)
    .await
    .expect("audit count")
    .count;
    assert_eq!(count, 2);
}

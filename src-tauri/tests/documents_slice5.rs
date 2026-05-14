use std::{fs, path::PathBuf};

use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    documents::{
        add_attachment, create_document, get_public_document, list_documents, list_public_documents,
        list_trash_documents, purge_document, restore_document, set_document_hidden, trash_document,
        update_document, AttachmentInput, DocumentInput, DocumentListFilter, StorageRoot,
        empty_trash,
    },
    master_data::{create_category, create_folder, create_office, CategoryInput, FolderInput, OfficeInput},
    users::{create_user, UserInput},
};
use uuid::Uuid;

struct Fixture {
    pool: DbPool,
    admin: String,
    secretary: String,
    category_id: i64,
    folder_id: i64,
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
    let folder_id = create_folder(
        &pool,
        &admin,
        FolderInput {
            category_id,
            folder_name: "Memos".to_owned(),
            description: None,
            folder_color: "#0F766E".to_owned(),
        },
    )
    .await
    .expect("folder");
    let office_id = create_office(&pool, &admin, office("OVPSPEE"))
        .await
        .expect("office");
    let root = std::env::temp_dir().join(format!("ovpspee-slice5-{}", Uuid::new_v4()));
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).expect("source dir");
    let storage = StorageRoot::new(root.join("storage")).expect("storage");
    Fixture { pool, admin, secretary, category_id, folder_id, office_id, storage, source_dir }
}

fn category(name: &str) -> CategoryInput {
    CategoryInput {
        category_name: name.to_owned(),
        description: None,
        color_code: "#2563EB".to_owned(),
        icon: Some("Folder".to_owned()),
    }
}

fn office(name: &str) -> OfficeInput {
    OfficeInput { office_name: name.to_owned(), description: None }
}

fn doc(fx: &Fixture, title: &str) -> DocumentInput {
    DocumentInput {
        document_name: title.to_owned(),
        category_id: fx.category_id,
        folder_id: Some(fx.folder_id),
        office_id: Some(fx.office_id),
        date_received: "2026-05-14".to_owned(),
        remarks: Some("Slice 5".to_owned()),
        status: "Filed".to_owned(),
    }
}

async fn create_doc(fx: &Fixture, title: &str) -> i64 {
    create_document(&fx.pool, &fx.secretary, doc(fx, title))
        .await
        .expect("document")
}

#[tokio::test]
async fn hide_and_unhide_document_controls_viewer_visibility() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Hide Me").await;

    set_document_hidden(&fx.pool, &fx.secretary, id, true).await.expect("hide");
    assert!(get_public_document(&fx.pool, id).await.is_err());

    set_document_hidden(&fx.pool, &fx.secretary, id, false).await.expect("unhide");
    assert!(get_public_document(&fx.pool, id).await.is_ok());
}

#[tokio::test]
async fn confidential_documents_can_be_explicitly_unhidden() {
    let fx = fixture().await;
    let mut confidential = doc(&fx, "Confidential");
    confidential.status = "Confidential".to_owned();
    let id = create_document(&fx.pool, &fx.secretary, confidential).await.expect("document");

    assert!(get_public_document(&fx.pool, id).await.is_err());
    set_document_hidden(&fx.pool, &fx.secretary, id, false).await.expect("unhide");
    assert!(get_public_document(&fx.pool, id).await.is_ok());
}

#[tokio::test]
async fn trash_document_sets_flags_and_hides_from_viewer() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Trash Me").await;

    trash_document(&fx.pool, &fx.secretary, id).await.expect("trash");
    let trash = list_trash_documents(&fx.pool, &fx.secretary).await.expect("trash list");
    let item = trash.iter().find(|row| row.document_id == id).expect("trashed row");

    assert!(item.is_trashed);
    assert_eq!(item.category_name, "TRASH");
    assert!(get_public_document(&fx.pool, id).await.is_err());
}

#[tokio::test]
async fn secretary_can_restore_to_original_folder_or_category_root() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Restore Folder").await;
    trash_document(&fx.pool, &fx.secretary, id).await.expect("trash");
    restore_document(&fx.pool, &fx.secretary, id).await.expect("restore");
    let restored = list_documents(&fx.pool, &fx.secretary, DocumentListFilter::default())
        .await
        .expect("docs")
        .into_iter()
        .find(|row| row.document_id == id)
        .expect("restored");
    assert_eq!(restored.folder_id, Some(fx.folder_id));

    let root_id = create_doc(&fx, "Restore Root").await;
    trash_document(&fx.pool, &fx.secretary, root_id).await.expect("trash");
    sqlx::query!("UPDATE folder SET is_active = 0 WHERE folder_id = ?", fx.folder_id)
        .execute(&fx.pool)
        .await
        .expect("deactivate folder");
    restore_document(&fx.pool, &fx.secretary, root_id).await.expect("restore to root");
    let restored = list_documents(&fx.pool, &fx.secretary, DocumentListFilter::default())
        .await
        .expect("docs")
        .into_iter()
        .find(|row| row.document_id == root_id)
        .expect("restored");
    assert_eq!(restored.folder_id, None);
}

#[tokio::test]
async fn restore_fails_when_original_category_inactive() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Restore Conflict").await;
    trash_document(&fx.pool, &fx.secretary, id).await.expect("trash");
    sqlx::query!("UPDATE category SET is_active = 0 WHERE category_id = ?", fx.category_id)
        .execute(&fx.pool)
        .await
        .expect("deactivate category");

    assert!(restore_document(&fx.pool, &fx.secretary, id).await.is_err());
}

#[tokio::test]
async fn secretary_cannot_purge_but_admin_can_purge_with_files() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Purge Me").await;
    let source = fx.source_dir.join("purge.pdf");
    fs::write(&source, b"%PDF-1.4\n").expect("source");
    add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        id,
        AttachmentInput { source_path: source.to_string_lossy().into_owned(), sort_order: Some(1) },
    )
    .await
    .expect("attach");
    let stored = sqlx::query!(
        "SELECT stored_relative_path FROM attachment WHERE document_id = ?",
        id
    )
    .fetch_one(&fx.pool)
    .await
    .expect("attachment")
    .stored_relative_path;
    let stored_path = fx.storage.resolve_relative(&stored);
    assert!(stored_path.exists());
    trash_document(&fx.pool, &fx.secretary, id).await.expect("trash");

    assert!(purge_document(&fx.pool, &fx.storage, &fx.secretary, id).await.is_err());
    purge_document(&fx.pool, &fx.storage, &fx.admin, id).await.expect("purge");

    assert!(!stored_path.exists());
    let attachments = sqlx::query!("SELECT COUNT(*) AS \"count!: i64\" FROM attachment WHERE document_id = ?", id)
        .fetch_one(&fx.pool)
        .await
        .expect("count");
    assert_eq!(attachments.count, 0);
}

#[tokio::test]
async fn empty_trash_purges_all_trashed_documents() {
    let fx = fixture().await;
    let first = create_doc(&fx, "Trash One").await;
    let second = create_doc(&fx, "Trash Two").await;
    trash_document(&fx.pool, &fx.secretary, first).await.expect("trash first");
    trash_document(&fx.pool, &fx.secretary, second).await.expect("trash second");

    let count = empty_trash(&fx.pool, &fx.storage, &fx.admin).await.expect("empty");
    assert_eq!(count, 2);
    assert!(list_trash_documents(&fx.pool, &fx.admin).await.expect("trash").is_empty());
}

#[tokio::test]
async fn normal_update_cannot_move_document_to_trash_category() {
    let fx = fixture().await;
    let id = create_doc(&fx, "No Direct Trash").await;
    let trash = sqlx::query!("SELECT category_id AS \"category_id!: i64\" FROM category WHERE category_name = 'TRASH'")
        .fetch_one(&fx.pool)
        .await
        .expect("trash")
        .category_id;
    let mut input = doc(&fx, "No Direct Trash");
    input.category_id = trash;
    input.folder_id = None;

    assert!(update_document(&fx.pool, &fx.secretary, id, input).await.is_err());
}

#[tokio::test]
async fn unauthenticated_viewer_cannot_call_lifecycle_commands() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Blocked").await;

    assert!(set_document_hidden(&fx.pool, "", id, true).await.is_err());
    assert!(trash_document(&fx.pool, "", id).await.is_err());
    assert!(restore_document(&fx.pool, "", id).await.is_err());
}

#[tokio::test]
async fn audit_logs_are_written_for_lifecycle_actions() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Audited").await;

    set_document_hidden(&fx.pool, &fx.secretary, id, true).await.expect("hide");
    trash_document(&fx.pool, &fx.secretary, id).await.expect("trash");
    restore_document(&fx.pool, &fx.secretary, id).await.expect("restore");
    trash_document(&fx.pool, &fx.secretary, id).await.expect("trash again");
    purge_document(&fx.pool, &fx.storage, &fx.admin, id).await.expect("purge");

    let count = sqlx::query!(
        "SELECT COUNT(*) AS \"count!: i64\" FROM audit_log
         WHERE table_affected = 'document' AND record_id = ?
           AND description IN ('Hid document', 'Moved document to trash', 'Restored document', 'Purged document')",
        id
    )
    .fetch_one(&fx.pool)
    .await
    .expect("audit count")
    .count;
    assert_eq!(count, 5);
}

#[tokio::test]
async fn viewer_lists_exclude_hidden_and_trashed_documents() {
    let fx = fixture().await;
    let visible = create_doc(&fx, "Visible").await;
    let hidden = create_doc(&fx, "Hidden").await;
    let trashed = create_doc(&fx, "Trashed").await;
    set_document_hidden(&fx.pool, &fx.secretary, hidden, true).await.expect("hide");
    trash_document(&fx.pool, &fx.secretary, trashed).await.expect("trash");

    let public = list_public_documents(&fx.pool, DocumentListFilter::default())
        .await
        .expect("public");

    assert!(public.iter().any(|row| row.document_id == visible));
    assert!(!public.iter().any(|row| row.document_id == hidden));
    assert!(!public.iter().any(|row| row.document_id == trashed));
}

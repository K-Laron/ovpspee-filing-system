use std::{
    fs,
    path::{Path, PathBuf},
};

use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    documents::{
        add_attachment, create_document, get_document, get_public_document, list_documents,
        list_public_documents, remove_attachment, reorder_attachments, update_document,
        AttachmentInput, DocumentInput, DocumentListFilter, StorageRoot,
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
    other_category_id: i64,
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
    let other_category_id = create_category(&pool, &admin, category("Outgoing"))
        .await
        .expect("other category");
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
    let root = std::env::temp_dir().join(format!("ovpspee-slice4-{}", Uuid::new_v4()));
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).expect("source dir");
    let storage = StorageRoot::new(root.join("storage")).expect("storage");

    Fixture {
        pool,
        admin,
        secretary,
        category_id,
        other_category_id,
        folder_id,
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
        remarks: Some("Filed during Slice 4".to_owned()),
        status: "Filed".to_owned(),
    }
}

fn write_file(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, bytes).expect("source file");
    path
}

#[tokio::test]
async fn create_document_success() {
    let fx = fixture().await;

    let id = create_document(&fx.pool, &fx.secretary, doc(&fx, "Board Memo"))
        .await
        .expect("document created");
    let item = get_document(&fx.pool, &fx.secretary, id).await.expect("document");

    assert_eq!(item.document.document_name, "Board Memo");
    assert_eq!(item.attachments.len(), 0);
}

#[tokio::test]
async fn create_document_with_attachment_copies_file_and_stores_relative_path() {
    let fx = fixture().await;
    let source = write_file(&fx.source_dir, "memo.pdf", b"%PDF-1.4\n");
    let id = create_document(&fx.pool, &fx.secretary, doc(&fx, "Memo With File"))
        .await
        .expect("document");

    let attachment_id = add_attachment(
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
    .expect("attachment");
    let item = get_document(&fx.pool, &fx.secretary, id).await.expect("document");
    let attachment = item
        .attachments
        .iter()
        .find(|row| row.attachment_id == attachment_id)
        .expect("attachment listed");

    assert!(fx.storage.resolve_relative(&attachment.stored_relative_path).exists());
    assert!(!Path::new(&attachment.stored_relative_path).is_absolute());
}

#[tokio::test]
async fn list_documents_for_secretary_includes_normal_documents() {
    let fx = fixture().await;
    create_document(&fx.pool, &fx.secretary, doc(&fx, "Visible Doc"))
        .await
        .expect("document");

    let rows = list_documents(&fx.pool, &fx.secretary, DocumentListFilter::default())
        .await
        .expect("documents");

    assert!(rows.iter().any(|row| row.document_name == "Visible Doc"));
}

#[tokio::test]
async fn viewer_list_excludes_hidden_and_trashed_documents() {
    let fx = fixture().await;
    let visible = create_document(&fx.pool, &fx.secretary, doc(&fx, "Visible"))
        .await
        .expect("visible");
    let hidden = create_document(&fx.pool, &fx.secretary, confidential_doc(&fx, "Hidden"))
        .await
        .expect("hidden");
    let trashed = create_document(&fx.pool, &fx.secretary, doc(&fx, "Trashed"))
        .await
        .expect("trashed");
    sqlx::query!("UPDATE document SET is_trashed = 1 WHERE document_id = ?", trashed)
        .execute(&fx.pool)
        .await
        .expect("mark trashed");

    let rows = list_public_documents(&fx.pool, DocumentListFilter::default())
        .await
        .expect("public documents");

    assert!(rows.iter().any(|row| row.document_id == visible));
    assert!(!rows.iter().any(|row| row.document_id == hidden));
    assert!(!rows.iter().any(|row| row.document_id == trashed));
}

#[tokio::test]
async fn viewer_cannot_call_create_or_update_commands() {
    let fx = fixture().await;
    let id = create_document(&fx.pool, &fx.secretary, doc(&fx, "Visible"))
        .await
        .expect("document");

    assert!(create_document(&fx.pool, "", doc(&fx, "Blocked")).await.is_err());
    assert!(update_document(&fx.pool, "", id, doc(&fx, "Blocked")).await.is_err());
    assert!(create_document(&fx.pool, &fx.admin, doc(&fx, "Admin Blocked")).await.is_err());
    assert!(update_document(&fx.pool, &fx.admin, id, doc(&fx, "Admin Blocked")).await.is_err());
}

#[tokio::test]
async fn update_document_success_and_folder_must_belong_to_category() {
    let fx = fixture().await;
    let id = create_document(&fx.pool, &fx.secretary, doc(&fx, "Original"))
        .await
        .expect("document");
    let mut updated = doc(&fx, "Updated");
    updated.folder_id = None;
    update_document(&fx.pool, &fx.secretary, id, updated)
        .await
        .expect("updated");
    let item = get_document(&fx.pool, &fx.secretary, id).await.expect("document");
    assert_eq!(item.document.document_name, "Updated");

    let mut invalid = doc(&fx, "Invalid");
    invalid.category_id = fx.other_category_id;
    invalid.folder_id = Some(fx.folder_id);
    assert!(update_document(&fx.pool, &fx.secretary, id, invalid).await.is_err());
}

#[tokio::test]
async fn remove_attachment_deletes_file_and_reorder_updates_sort_order() {
    let fx = fixture().await;
    let id = create_document(&fx.pool, &fx.secretary, doc(&fx, "Files"))
        .await
        .expect("document");
    let first = add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        id,
        AttachmentInput {
            source_path: write_file(&fx.source_dir, "a.pdf", b"%PDF-a").to_string_lossy().into_owned(),
            sort_order: Some(1),
        },
    )
    .await
    .expect("first");
    let second = add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        id,
        AttachmentInput {
            source_path: write_file(&fx.source_dir, "b.pdf", b"%PDF-b").to_string_lossy().into_owned(),
            sort_order: Some(2),
        },
    )
    .await
    .expect("second");
    reorder_attachments(&fx.pool, &fx.secretary, id, vec![second, first])
        .await
        .expect("reordered");
    let rows = get_document(&fx.pool, &fx.secretary, id)
        .await
        .expect("document")
        .attachments;
    assert_eq!(rows[0].attachment_id, second);

    let removed_path = rows
        .iter()
        .find(|row| row.attachment_id == first)
        .expect("first path")
        .stored_relative_path
        .clone();
    remove_attachment(&fx.pool, &fx.storage, &fx.secretary, first)
        .await
        .expect("removed");
    assert!(!fx.storage.resolve_relative(&removed_path).exists());
}

#[tokio::test]
async fn invalid_file_type_and_path_traversal_rejected() {
    let fx = fixture().await;
    let id = create_document(&fx.pool, &fx.secretary, doc(&fx, "Files"))
        .await
        .expect("document");
    let exe = write_file(&fx.source_dir, "bad.exe", b"MZ");

    assert!(add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        id,
        AttachmentInput {
            source_path: exe.to_string_lossy().into_owned(),
            sort_order: None,
        },
    )
    .await
    .is_err());
    assert!(fx.storage.resolve_checked("../escape.pdf").is_err());
}

#[tokio::test]
async fn file_over_one_gb_rejected_without_copying() {
    let fx = fixture().await;
    let id = create_document(&fx.pool, &fx.secretary, doc(&fx, "Large File"))
        .await
        .expect("document");
    let large = fx.source_dir.join("large.pdf");
    let file = fs::File::create(&large).expect("large file");
    #[cfg(windows)]
    {
        std::process::Command::new("fsutil")
            .args(["sparse", "setflag", &large.to_string_lossy()])
            .status()
            .expect("mark sparse");
    }
    file.set_len(1_073_741_825).expect("sparse large file");

    assert!(add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        id,
        AttachmentInput {
            source_path: large.to_string_lossy().into_owned(),
            sort_order: None,
        },
    )
    .await
    .is_err());
}

#[tokio::test]
async fn confidential_status_auto_hides_document_from_public_detail() {
    let fx = fixture().await;
    let id = create_document(&fx.pool, &fx.secretary, confidential_doc(&fx, "Confidential Memo"))
        .await
        .expect("document");
    let item = get_document(&fx.pool, &fx.secretary, id).await.expect("document");

    assert!(item.document.is_hidden);
    assert!(get_public_document(&fx.pool, id).await.is_err());
}

fn confidential_doc(fx: &Fixture, title: &str) -> DocumentInput {
    let mut input = doc(fx, title);
    input.status = "Confidential".to_owned();
    input
}

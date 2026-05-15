use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
};

use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    documents::{
        get_attachment_file_path, get_document, list_documents, DocumentInput, DocumentListFilter,
        StorageRoot,
    },
    master_data::{
        create_category, create_folder, create_office, CategoryInput, FolderInput, OfficeInput,
    },
    scan_intake::{
        attach_scan_to_document, file_scan_as_document, import_scan_files, list_scan_intake,
        remove_scan_intake,
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
            folder_name: "Scans".to_owned(),
            description: None,
            folder_color: "#0F766E".to_owned(),
        },
    )
    .await
    .expect("folder");
    let office_id = create_office(&pool, &admin, office("OVPSPEE"))
        .await
        .expect("office");
    let root = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(format!("ovpspee-slice7-{}", Uuid::new_v4()));
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).expect("source dir");
    let storage = StorageRoot::new(root.join("storage")).expect("storage");
    Fixture {
        pool,
        admin,
        secretary,
        category_id,
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
        remarks: Some("Filed from scan intake".to_owned()),
        status: "Filed".to_owned(),
    }
}

fn write_pdf(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, b"%PDF-1.4\n").expect("pdf");
    path
}

fn write_png(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, b"\x89PNG\r\n\x1A\nscan").expect("png");
    path
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn create_sparse_large_file(path: &Path) {
    File::create(path).expect("huge file");
    #[cfg(windows)]
    {
        let status = Command::new("fsutil")
            .args(["sparse", "setflag", &path_string(path)])
            .status()
            .expect("fsutil sparse");
        assert!(status.success());
    }
    File::options()
        .write(true)
        .open(path)
        .expect("open huge")
        .set_len(1_073_741_825)
        .expect("set len");
}

async fn import_one(fx: &Fixture, name: &str) -> i64 {
    let source = write_pdf(&fx.source_dir, name);
    import_scan_files(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        vec![path_string(&source)],
    )
    .await
    .expect("import")[0]
}

async fn audit_count(pool: &DbPool, description: &str) -> i64 {
    sqlx::query!(
        "SELECT COUNT(*) AS \"count!: i64\" FROM audit_log WHERE description = ?",
        description
    )
    .fetch_one(pool)
    .await
    .expect("audit")
    .count
}

#[tokio::test]
async fn secretary_can_import_one_or_multiple_scan_files() {
    let fx = fixture().await;
    let first = write_pdf(&fx.source_dir, "scan-one.pdf");
    let ids = import_scan_files(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        vec![path_string(&first)],
    )
    .await
    .expect("single import");
    assert_eq!(ids.len(), 1);

    let second = write_pdf(&fx.source_dir, "scan-two.pdf");
    let third = write_png(&fx.source_dir, "scan-three.png");
    let ids = import_scan_files(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        vec![path_string(&second), path_string(&third)],
    )
    .await
    .expect("multi import");
    assert_eq!(ids.len(), 2);
    assert_eq!(
        list_scan_intake(&fx.pool, &fx.secretary)
            .await
            .expect("pending")
            .len(),
        3
    );
}

#[tokio::test]
async fn imported_scan_is_copied_and_stores_relative_path_only() {
    let fx = fixture().await;
    import_one(&fx, "relative.pdf").await;
    let row = list_scan_intake(&fx.pool, &fx.secretary)
        .await
        .expect("pending")
        .remove(0);

    assert!(!Path::new(&row.stored_relative_path).is_absolute());
    assert!(fx
        .storage
        .resolve_relative(&row.stored_relative_path)
        .exists());
}

#[tokio::test]
async fn scan_import_rejects_invalid_type_oversized_file_and_path_traversal() {
    let fx = fixture().await;
    let exe = fx.source_dir.join("bad.exe");
    fs::write(&exe, b"MZ").expect("exe");
    assert!(import_scan_files(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        vec![path_string(&exe)]
    )
    .await
    .is_err());

    let huge = fx.source_dir.join("huge.pdf");
    create_sparse_large_file(&huge);
    assert!(import_scan_files(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        vec![path_string(&huge)]
    )
    .await
    .is_err());

    assert!(import_scan_files(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        vec!["..\\escape.pdf".to_owned()]
    )
    .await
    .is_err());
}

#[tokio::test]
async fn secretary_can_file_scan_as_new_document_with_attachment() {
    let fx = fixture().await;
    let scan_id = import_one(&fx, "file-me.pdf").await;

    let document_id = file_scan_as_document(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        vec![scan_id],
        doc(&fx, "Filed Scan"),
    )
    .await
    .expect("file scan");
    let detail = get_document(&fx.pool, &fx.secretary, document_id)
        .await
        .expect("document detail");
    let attachment = detail.attachments.first().expect("attachment");
    let attachment_path = get_attachment_file_path(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        attachment.attachment_id,
    )
    .await
    .expect("attachment path");

    assert_eq!(detail.document.document_name, "Filed Scan");
    assert_eq!(detail.attachments.len(), 1);
    assert!(Path::new(&attachment_path).exists());
    assert!(list_scan_intake(&fx.pool, &fx.secretary)
        .await
        .expect("pending")
        .is_empty());
}

#[tokio::test]
async fn filed_scan_creates_document_and_attachment_metadata() {
    let fx = fixture().await;
    let scan_id = import_one(&fx, "metadata.pdf").await;

    let document_id = file_scan_as_document(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        vec![scan_id],
        doc(&fx, "Metadata Scan"),
    )
    .await
    .expect("file");
    let documents = list_documents(&fx.pool, &fx.secretary, DocumentListFilter::default())
        .await
        .expect("documents");
    let attachment = sqlx::query!(
        "SELECT original_file_name, stored_relative_path FROM attachment WHERE document_id = ?",
        document_id
    )
    .fetch_one(&fx.pool)
    .await
    .expect("attachment");
    let scan = sqlx::query!(
        "SELECT status, filed_document_id FROM scan_intake WHERE scan_intake_id = ?",
        scan_id
    )
    .fetch_one(&fx.pool)
    .await
    .expect("scan");

    assert!(documents.iter().any(|row| row.document_id == document_id));
    assert_eq!(attachment.original_file_name, "metadata.pdf");
    assert!(!Path::new(&attachment.stored_relative_path).is_absolute());
    assert_eq!(scan.status, "Filed");
    assert_eq!(scan.filed_document_id, Some(document_id));
}

#[tokio::test]
async fn secretary_can_attach_scan_to_existing_document() {
    let fx = fixture().await;
    let scan_id = import_one(&fx, "attach-me.pdf").await;
    let document_id = file_scan_as_document(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        vec![import_one(&fx, "base.pdf").await],
        doc(&fx, "Base"),
    )
    .await
    .expect("base");

    let attachment_ids = attach_scan_to_document(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        vec![scan_id],
        document_id,
    )
    .await
    .expect("attach existing");

    assert_eq!(attachment_ids.len(), 1);
    assert_eq!(
        get_document(&fx.pool, &fx.secretary, document_id)
            .await
            .expect("document")
            .attachments
            .len(),
        2
    );
}

#[tokio::test]
async fn viewer_admin_and_unauthenticated_users_cannot_use_scan_intake() {
    let fx = fixture().await;
    let source = write_pdf(&fx.source_dir, "denied.pdf");
    let source_path = path_string(&source);
    assert!(
        import_scan_files(&fx.pool, &fx.storage, "", vec![source_path.clone()])
            .await
            .is_err()
    );
    assert!(list_scan_intake(&fx.pool, "").await.is_err());
    assert!(
        file_scan_as_document(&fx.pool, &fx.storage, "", vec![1], doc(&fx, "Denied"))
            .await
            .is_err()
    );

    assert!(
        import_scan_files(&fx.pool, &fx.storage, &fx.admin, vec![source_path.clone()])
            .await
            .is_err()
    );
    assert!(list_scan_intake(&fx.pool, &fx.admin).await.is_err());
    assert!(file_scan_as_document(
        &fx.pool,
        &fx.storage,
        &fx.admin,
        vec![1],
        doc(&fx, "Admin Denied")
    )
    .await
    .is_err());
}

#[tokio::test]
async fn scan_import_and_file_write_audit_logs() {
    let fx = fixture().await;
    let scan_id = import_one(&fx, "audit.pdf").await;

    assert_eq!(audit_count(&fx.pool, "Imported scan intake file").await, 1);

    file_scan_as_document(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        vec![scan_id],
        doc(&fx, "Audit Filed"),
    )
    .await
    .expect("file");

    assert_eq!(
        audit_count(&fx.pool, "Filed scan intake as new document").await,
        1
    );
}

#[tokio::test]
async fn removing_pending_scan_hides_it_and_preserves_recoverable_file() {
    let fx = fixture().await;
    let scan_id = import_one(&fx, "remove-me.pdf").await;
    let row = list_scan_intake(&fx.pool, &fx.secretary)
        .await
        .expect("pending")
        .remove(0);
    let stored_path = fx.storage.resolve_relative(&row.stored_relative_path);

    remove_scan_intake(&fx.pool, &fx.secretary, scan_id)
        .await
        .expect("remove");

    assert!(list_scan_intake(&fx.pool, &fx.secretary)
        .await
        .expect("pending")
        .is_empty());
    assert!(stored_path.exists());
    assert_eq!(audit_count(&fx.pool, "Removed scan intake file").await, 1);
}

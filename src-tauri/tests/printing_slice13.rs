use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    devices::PrinterDevice,
    documents::{create_document, set_document_hidden, trash_document, DocumentInput, StorageRoot},
    master_data::{
        create_category, create_folder, create_office, CategoryInput, FolderInput, OfficeInput,
    },
    printing::{mock::MockPrintProvider, print_document_pdf_with_provider, PrintOptions},
    users::{create_user, UserInput},
};
use sqlx::Row;
use tempfile::TempDir;

struct Fixture {
    pool: DbPool,
    admin: String,
    secretary: String,
    category_id: i64,
    folder_id: i64,
    office_id: i64,
    storage: StorageRoot,
    _root: TempDir,
    provider: MockPrintProvider,
}

async fn fixture() -> Fixture {
    let pool = create_test_pool().await.expect("pool");
    create_first_admin(&pool, "Kenneth", "Laron", "admin", "Valid123!")
        .await
        .expect("admin");
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
    .expect("secretary");
    let secretary = authenticate_user(&pool, "secretary", "Valid123!")
        .await
        .expect("secretary login")
        .session_id;
    let category_id = create_category(
        &pool,
        &admin,
        CategoryInput {
            category_name: "Print".to_owned(),
            description: None,
            color_code: "#2563EB".to_owned(),
            icon: Some("FileText".to_owned()),
        },
    )
    .await
    .expect("category");
    let folder_id = create_folder(
        &pool,
        &admin,
        FolderInput {
            category_id,
            folder_name: "Printables".to_owned(),
            description: None,
            folder_color: "#2563EB".to_owned(),
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
    let root = TempDir::new().expect("tempdir");
    let storage = StorageRoot::new(root.path().join("storage")).expect("storage");
    Fixture {
        pool,
        admin,
        secretary,
        category_id,
        folder_id,
        office_id,
        storage,
        _root: root,
        provider: MockPrintProvider {
            printers: vec![PrinterDevice {
                printer_id: "printer-main".to_owned(),
                name: "Main Printer".to_owned(),
                is_default: true,
                status: "Idle".to_owned(),
                is_available: true,
                is_network: false,
            }],
            fail_print: false,
        },
    }
}

fn doc(fx: &Fixture, name: &str, status: &str) -> DocumentInput {
    DocumentInput {
        document_name: name.to_owned(),
        category_id: fx.category_id,
        folder_id: Some(fx.folder_id),
        office_id: Some(fx.office_id),
        date_received: "2026-05-15".to_owned(),
        remarks: Some("Print remarks".to_owned()),
        status: status.to_owned(),
    }
}

async fn create_doc(fx: &Fixture, name: &str, status: &str) -> i64 {
    create_document(&fx.pool, &fx.secretary, doc(fx, name, status))
        .await
        .expect("document")
}

fn options(copies: i64) -> PrintOptions {
    PrintOptions { copies }
}

#[tokio::test]
async fn secretary_can_print_accessible_normal_document() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Secretary Print", "Filed").await;

    let result = print_document_pdf_with_provider(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        id,
        "printer-main",
        options(1),
        &fx.provider,
    )
    .await
    .expect("print");

    assert_eq!(result.document_id, id);
    assert_eq!(result.printer_name, "Main Printer");
}

#[tokio::test]
async fn secretary_can_print_hidden_and_confidential_document() {
    let fx = fixture().await;
    let hidden = create_doc(&fx, "Hidden Print", "Filed").await;
    set_document_hidden(&fx.pool, &fx.secretary, hidden, true)
        .await
        .expect("hide");
    let confidential = create_doc(&fx, "Confidential Print", "Confidential").await;

    assert!(print_document_pdf_with_provider(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        hidden,
        "printer-main",
        options(1),
        &fx.provider,
    )
    .await
    .is_ok());
    assert!(print_document_pdf_with_provider(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        confidential,
        "printer-main",
        options(1),
        &fx.provider,
    )
    .await
    .is_ok());
}

#[tokio::test]
async fn public_viewer_can_print_visible_public_document() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Public Print", "Filed").await;

    assert!(print_document_pdf_with_provider(
        &fx.pool,
        &fx.storage,
        None,
        id,
        "printer-main",
        options(2),
        &fx.provider,
    )
    .await
    .is_ok());
}

#[tokio::test]
async fn public_viewer_cannot_print_hidden_confidential_or_trashed_documents() {
    let fx = fixture().await;
    let hidden = create_doc(&fx, "Hidden Public Print", "Filed").await;
    set_document_hidden(&fx.pool, &fx.secretary, hidden, true)
        .await
        .expect("hide");
    let confidential = create_doc(&fx, "Confidential Public Print", "Confidential").await;
    let trashed = create_doc(&fx, "Trashed Public Print", "Filed").await;
    trash_document(&fx.pool, &fx.secretary, trashed)
        .await
        .expect("trash");

    for id in [hidden, confidential, trashed] {
        assert!(print_document_pdf_with_provider(
            &fx.pool,
            &fx.storage,
            None,
            id,
            "printer-main",
            options(1),
            &fx.provider,
        )
        .await
        .is_err());
    }
}

#[tokio::test]
async fn unauthorized_and_admin_users_cannot_print_restricted_document() {
    let fx = fixture().await;
    let id = create_doc(&fx, "No Admin Print", "Filed").await;

    assert!(print_document_pdf_with_provider(
        &fx.pool,
        &fx.storage,
        Some(&fx.admin),
        id,
        "printer-main",
        options(1),
        &fx.provider,
    )
    .await
    .is_err());
    assert!(print_document_pdf_with_provider(
        &fx.pool,
        &fx.storage,
        Some("bad-session"),
        id,
        "printer-main",
        options(1),
        &fx.provider,
    )
    .await
    .is_err());
}

#[tokio::test]
async fn invalid_printer_and_copies_are_rejected() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Invalid Print", "Filed").await;

    assert!(print_document_pdf_with_provider(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        id,
        "C:\\printer",
        options(1),
        &fx.provider,
    )
    .await
    .is_err());
    assert!(print_document_pdf_with_provider(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        id,
        "printer-main",
        options(0),
        &fx.provider,
    )
    .await
    .is_err());
    assert!(print_document_pdf_with_provider(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        id,
        "printer-main",
        options(21),
        &fx.provider,
    )
    .await
    .is_err());
}

#[tokio::test]
async fn missing_document_returns_safe_not_found_error() {
    let fx = fixture().await;

    let err = print_document_pdf_with_provider(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        99_999,
        "printer-main",
        options(1),
        &fx.provider,
    )
    .await
    .expect_err("not found");

    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn print_action_writes_safe_audit_event() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Audit Print", "Filed").await;

    print_document_pdf_with_provider(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        id,
        "printer-main",
        options(1),
        &fx.provider,
    )
    .await
    .expect("print");

    let row = sqlx::query(
        "SELECT description FROM audit_log WHERE description LIKE 'Printed document PDF%'",
    )
    .fetch_one(&fx.pool)
    .await
    .expect("audit");
    let description = row.get::<String, _>("description");
    assert!(description.contains("Main Printer"));
    assert!(!description.contains(":\\"));
    assert!(!description.contains("print-tmp"));
}

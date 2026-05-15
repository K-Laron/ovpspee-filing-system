use std::{fs, path::PathBuf};

use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    documents::{
        add_attachment, create_document, export_document_pdf, get_attachment_preview_info,
        get_attachment_preview_page, set_document_hidden, trash_document, AttachmentInput,
        DocumentInput, StorageRoot,
    },
    master_data::{
        create_category, create_folder, create_office, CategoryInput, FolderInput, OfficeInput,
    },
    users::{create_user, UserInput},
};
use sqlx::Row;
use tempfile::TempDir;

struct Fixture {
    pool: DbPool,
    secretary: String,
    category_id: i64,
    folder_id: i64,
    office_id: i64,
    storage: StorageRoot,
    root: TempDir,
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
            category_name: "PDF Export".to_owned(),
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
            folder_name: "Exports".to_owned(),
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
        secretary,
        category_id,
        folder_id,
        office_id,
        storage,
        root,
    }
}

fn doc(fx: &Fixture, name: &str, status: &str) -> DocumentInput {
    DocumentInput {
        document_name: name.to_owned(),
        category_id: fx.category_id,
        folder_id: Some(fx.folder_id),
        office_id: Some(fx.office_id),
        date_received: "2026-05-15".to_owned(),
        remarks: Some("PDF export remarks".to_owned()),
        status: status.to_owned(),
    }
}

async fn create_doc(fx: &Fixture, name: &str, status: &str) -> i64 {
    create_document(&fx.pool, &fx.secretary, doc(fx, name, status))
        .await
        .expect("document")
}

fn out(fx: &Fixture, name: &str) -> String {
    fx.root.path().join(name).to_string_lossy().into_owned()
}

fn source(fx: &Fixture, name: &str, bytes: &[u8]) -> PathBuf {
    let path = fx.root.path().join("source").join(name);
    fs::create_dir_all(path.parent().expect("parent")).expect("source parent");
    fs::write(&path, bytes).expect("source");
    path
}

#[tokio::test]
async fn public_viewer_can_export_visible_document_with_metadata() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Public Export", "Filed").await;
    let output = out(&fx, "public-export.pdf");

    export_document_pdf(&fx.pool, &fx.storage, None, id, &output)
        .await
        .expect("export");

    let bytes = fs::read(&output).expect("pdf");
    let text = String::from_utf8_lossy(&bytes);
    assert!(bytes.len() > 100);
    assert!(text.contains("Public Export"));
    assert!(text.contains("UNIVERSITY OF EASTERN PHILIPPINES"));
    assert!(text.contains("PAGE 1 of"));
    assert!(text.contains("system copy"));
}

#[tokio::test]
async fn public_viewer_cannot_export_hidden_confidential_or_trashed_documents() {
    let fx = fixture().await;
    let hidden = create_doc(&fx, "Hidden Export", "Filed").await;
    set_document_hidden(&fx.pool, &fx.secretary, hidden, true)
        .await
        .expect("hide");
    let confidential = create_doc(&fx, "Confidential Export", "Confidential").await;
    let trashed = create_doc(&fx, "Trashed Export", "Filed").await;
    trash_document(&fx.pool, &fx.secretary, trashed)
        .await
        .expect("trash");

    assert!(
        export_document_pdf(&fx.pool, &fx.storage, None, hidden, &out(&fx, "hidden.pdf"))
            .await
            .is_err()
    );
    assert!(export_document_pdf(
        &fx.pool,
        &fx.storage,
        None,
        confidential,
        &out(&fx, "confidential.pdf")
    )
    .await
    .is_err());
    assert!(export_document_pdf(
        &fx.pool,
        &fx.storage,
        None,
        trashed,
        &out(&fx, "trashed.pdf")
    )
    .await
    .is_err());
}

#[tokio::test]
async fn secretary_can_export_normal_hidden_and_confidential_documents() {
    let fx = fixture().await;
    let normal = create_doc(&fx, "Secretary Normal Export", "Filed").await;
    let hidden = create_doc(&fx, "Secretary Hidden Export", "Filed").await;
    set_document_hidden(&fx.pool, &fx.secretary, hidden, true)
        .await
        .expect("hide");
    let confidential = create_doc(&fx, "Secretary Confidential Export", "Confidential").await;

    for (id, file_name) in [
        (normal, "secretary-normal.pdf"),
        (hidden, "secretary-hidden.pdf"),
        (confidential, "secretary-confidential.pdf"),
    ] {
        let output = out(&fx, file_name);
        export_document_pdf(&fx.pool, &fx.storage, Some(&fx.secretary), id, &output)
            .await
            .expect("secretary export");
        assert!(fs::metadata(output).expect("pdf").len() > 100);
    }
}

#[tokio::test]
async fn export_handles_pdf_image_unsupported_and_missing_attachments() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Attachment Export", "Filed").await;
    let pdf = source(
        &fx,
        "two-page.pdf",
        b"%PDF-1.4\n1 0 obj << /Type /Page >> endobj\n2 0 obj << /Type /Page >> endobj\n",
    );
    let png = source(&fx, "image.png", b"\x89PNG\r\n\x1a\nslice9");
    let txt = source(&fx, "notes.txt", b"plain text");
    add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        id,
        AttachmentInput {
            source_path: pdf.to_string_lossy().into_owned(),
            sort_order: Some(1),
        },
    )
    .await
    .expect("pdf attachment");
    add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        id,
        AttachmentInput {
            source_path: png.to_string_lossy().into_owned(),
            sort_order: Some(2),
        },
    )
    .await
    .expect("image attachment");
    let txt_id = add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        id,
        AttachmentInput {
            source_path: txt.to_string_lossy().into_owned(),
            sort_order: Some(3),
        },
    )
    .await
    .expect("txt attachment");
    let stored = sqlx::query("SELECT stored_relative_path FROM attachment WHERE attachment_id = ?")
        .bind(txt_id)
        .fetch_one(&fx.pool)
        .await
        .expect("stored")
        .get::<String, _>("stored_relative_path");
    fs::remove_file(fx.storage.resolve_relative(&stored)).expect("remove stored");

    let output = out(&fx, "attachments.pdf");
    export_document_pdf(&fx.pool, &fx.storage, Some(&fx.secretary), id, &output)
        .await
        .expect("export");
    let text = String::from_utf8_lossy(&fs::read(output).expect("pdf")).to_string();

    assert!(text.contains("Attachment Manifest"));
    assert!(text.contains("two-page.pdf"));
    assert!(text.contains("Image attachment detected"));
    assert!(text.contains("file unavailable"));
}

#[tokio::test]
async fn export_rejects_bad_document_and_bad_output_path_and_writes_audit() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Audited Export", "Filed").await;
    let bad_path = fx
        .root
        .path()
        .join("nested")
        .join("..")
        .join("bad.pdf")
        .to_string_lossy()
        .into_owned();
    assert!(export_document_pdf(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        999_999,
        &out(&fx, "missing.pdf")
    )
    .await
    .is_err());
    assert!(
        export_document_pdf(&fx.pool, &fx.storage, Some(&fx.secretary), id, &bad_path)
            .await
            .is_err()
    );

    export_document_pdf(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        id,
        &out(&fx, "audited.pdf"),
    )
    .await
    .expect("export");
    let count = sqlx::query(
        "SELECT COUNT(*) AS count FROM audit_log WHERE log_action = 'EXPORT' AND table_affected = 'document' AND record_id = ?",
    )
    .bind(id)
    .fetch_one(&fx.pool)
    .await
    .expect("audit")
    .get::<i64, _>("count");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn attachment_preview_enforces_visibility_and_supports_pdf_pagination() {
    let fx = fixture().await;
    let visible = create_doc(&fx, "Preview Visible", "Filed").await;
    let hidden = create_doc(&fx, "Preview Hidden", "Filed").await;
    set_document_hidden(&fx.pool, &fx.secretary, hidden, true)
        .await
        .expect("hide");
    let pdf = source(
        &fx,
        "preview.pdf",
        b"%PDF-1.4\n1 0 obj << /Type /Page >> endobj\n2 0 obj << /Type /Page >> endobj\n",
    );
    let visible_attachment = add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        visible,
        AttachmentInput {
            source_path: pdf.to_string_lossy().into_owned(),
            sort_order: Some(1),
        },
    )
    .await
    .expect("visible attachment");
    let hidden_attachment = add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        hidden,
        AttachmentInput {
            source_path: pdf.to_string_lossy().into_owned(),
            sort_order: Some(1),
        },
    )
    .await
    .expect("hidden attachment");

    let info = get_attachment_preview_info(&fx.pool, &fx.storage, None, visible_attachment)
        .await
        .expect("public preview info");
    assert_eq!(info.preview_kind, "Pdf");
    assert_eq!(info.page_count, Some(2));
    let page =
        get_attachment_preview_page(&fx.pool, &fx.storage, None, visible_attachment, Some(2))
            .await
            .expect("page two");
    assert_eq!(page.page_number, 2);
    assert!(page.file_path.is_some());
    assert!(
        get_attachment_preview_info(&fx.pool, &fx.storage, None, hidden_attachment)
            .await
            .is_err()
    );
    assert!(get_attachment_preview_info(
        &fx.pool,
        &fx.storage,
        Some(&fx.secretary),
        hidden_attachment
    )
    .await
    .is_ok());
}

#[tokio::test]
async fn attachment_preview_rejects_path_traversal() {
    let fx = fixture().await;
    let id = create_doc(&fx, "Traversal Preview", "Filed").await;
    let txt = source(&fx, "safe.txt", b"safe");
    let attachment_id = add_attachment(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        id,
        AttachmentInput {
            source_path: txt.to_string_lossy().into_owned(),
            sort_order: Some(1),
        },
    )
    .await
    .expect("attachment");
    sqlx::query(
        "UPDATE attachment SET stored_relative_path = '../escape.txt' WHERE attachment_id = ?",
    )
    .bind(attachment_id)
    .execute(&fx.pool)
    .await
    .expect("tamper");

    assert!(
        get_attachment_preview_info(&fx.pool, &fx.storage, Some(&fx.secretary), attachment_id)
            .await
            .is_err()
    );
}

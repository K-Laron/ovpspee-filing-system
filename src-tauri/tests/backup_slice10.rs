use std::{fs, path::PathBuf};

use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    backup::{
        create_backup, export_backup_archive, get_backup_settings, list_backup_history,
        restore_from_backup, update_backup_settings, validate_backup_archive, BackupRuntime,
        BackupSettingsInput,
    },
    db::{connect_database, DbPool},
    documents::{add_attachment, create_document, AttachmentInput, DocumentInput, StorageRoot},
    master_data::{
        create_category, create_folder, create_office, CategoryInput, FolderInput, OfficeInput,
    },
    users::{create_user, UserInput},
};
use sqlx::Row;
use tempfile::TempDir;

struct Fixture {
    pool: DbPool,
    root: TempDir,
    runtime: BackupRuntime,
    admin: String,
    secretary: String,
    attachment_path: PathBuf,
}

async fn fixture() -> Fixture {
    let root = TempDir::new().expect("tempdir");
    let app_data = root.path().join("app-data");
    fs::create_dir_all(&app_data).expect("app data");
    let db_path = app_data.join("filing_system.db");
    let storage = StorageRoot::new(app_data.join("storage")).expect("storage");
    let pool = connect_database(&db_path).await.expect("db");

    create_first_admin(&pool, "Ada", "Admin", "admin", "Valid123!")
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
            category_name: "Backup Docs".to_owned(),
            description: None,
            color_code: "#2563EB".to_owned(),
            icon: None,
        },
    )
    .await
    .expect("category");
    let folder_id = create_folder(
        &pool,
        &admin,
        FolderInput {
            category_id,
            folder_name: "Folder".to_owned(),
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
    let document_id = create_document(
        &pool,
        &secretary,
        DocumentInput {
            document_name: "Backup Source".to_owned(),
            category_id,
            folder_id: Some(folder_id),
            office_id: Some(office_id),
            date_received: "2026-05-15".to_owned(),
            remarks: Some("backup test".to_owned()),
            status: "Filed".to_owned(),
        },
    )
    .await
    .expect("document");
    let source = root.path().join("source.pdf");
    fs::write(&source, b"%PDF-1.4 backup attachment").expect("source");
    let attachment_id = add_attachment(
        &pool,
        &storage,
        &secretary,
        document_id,
        AttachmentInput {
            source_path: source.to_string_lossy().into_owned(),
            sort_order: Some(1),
        },
    )
    .await
    .expect("attachment");
    let stored: String =
        sqlx::query("SELECT stored_relative_path FROM attachment WHERE attachment_id = ?")
            .bind(attachment_id)
            .fetch_one(&pool)
            .await
            .expect("stored")
            .get("stored_relative_path");
    let attachment_path = storage.resolve_relative(&stored);

    let runtime = BackupRuntime::new(app_data.clone(), db_path, storage);

    Fixture {
        pool,
        root,
        runtime,
        admin,
        secretary,
        attachment_path,
    }
}

#[tokio::test]
async fn admin_can_get_and_update_backup_settings() {
    let fx = fixture().await;
    let default = get_backup_settings(&fx.pool, &fx.runtime, &fx.admin)
        .await
        .expect("settings");
    assert_eq!(default.schedule_time, "02:00");
    assert_eq!(default.retention_count, 10);
    assert!(default.is_local_app_data);

    let custom = fx.root.path().join("custom-backups");
    let updated = update_backup_settings(
        &fx.pool,
        &fx.runtime,
        &fx.admin,
        BackupSettingsInput {
            destination_path: Some(custom.to_string_lossy().into_owned()),
            schedule_enabled: true,
            schedule_time: "03:30".to_owned(),
            retention_count: 3,
        },
    )
    .await
    .expect("update");

    assert_eq!(updated.destination_path, custom.to_string_lossy());
    assert!(updated.schedule_enabled);
    assert_eq!(updated.retention_count, 3);
}

#[tokio::test]
async fn non_admin_cannot_access_backup_settings_or_create_backup() {
    let fx = fixture().await;

    assert!(get_backup_settings(&fx.pool, &fx.runtime, &fx.secretary)
        .await
        .is_err());
    assert!(create_backup(&fx.pool, &fx.runtime, &fx.secretary, false)
        .await
        .is_err());
    assert!(create_backup(&fx.pool, &fx.runtime, "", false)
        .await
        .is_err());
}

#[tokio::test]
async fn admin_can_create_backup_with_manifest_checksums_and_storage_files() {
    let fx = fixture().await;
    let backup = create_backup(&fx.pool, &fx.runtime, &fx.admin, false)
        .await
        .expect("backup");

    assert!(backup.backup_path.ends_with(&backup.backup_name));
    assert!(PathBuf::from(&backup.manifest_path).exists());
    assert!(PathBuf::from(&backup.database_path).exists());
    assert!(PathBuf::from(&backup.storage_path).exists());
    assert!(backup.total_bytes > 0);
    assert!(backup.file_count >= 3);

    let manifest_text = fs::read_to_string(&backup.manifest_path).expect("manifest");
    assert!(manifest_text.contains("\"app_version\""));
    assert!(manifest_text.contains("\"schema_version\""));
    assert!(manifest_text.contains("\"checksums\""));
    assert!(manifest_text.contains("storage/documents/"));
    assert!(!manifest_text.contains(&fx.root.path().to_string_lossy().to_string()));

    let history = list_backup_history(&fx.pool, &fx.runtime, &fx.admin)
        .await
        .expect("history");
    assert_eq!(history.len(), 1);
    assert!(history[0].is_valid);
}

#[tokio::test]
async fn backup_destination_rejects_path_traversal() {
    let fx = fixture().await;
    let err = update_backup_settings(
        &fx.pool,
        &fx.runtime,
        &fx.admin,
        BackupSettingsInput {
            destination_path: Some("..\\bad".to_owned()),
            schedule_enabled: false,
            schedule_time: "02:00".to_owned(),
            retention_count: 10,
        },
    )
    .await
    .expect_err("traversal rejected")
    .to_string();
    assert!(err.contains("ERR_VALIDATION"));
}

#[tokio::test]
async fn export_archive_validates_and_corrupt_archive_is_rejected() {
    let fx = fixture().await;
    let backup = create_backup(&fx.pool, &fx.runtime, &fx.admin, false)
        .await
        .expect("backup");
    let archive = fx.root.path().join("portable.ovpspee-backup");

    export_backup_archive(
        &fx.pool,
        &fx.runtime,
        &fx.admin,
        backup.backup_name.clone(),
        archive.to_string_lossy().into_owned(),
    )
    .await
    .expect("export");

    let validation = validate_backup_archive(
        &fx.pool,
        &fx.runtime,
        &fx.admin,
        archive.to_string_lossy().into_owned(),
    )
    .await
    .expect("validate");
    assert!(validation.is_valid);
    assert!(validation.file_count >= 2);

    let corrupt = fx.root.path().join("bad.ovpspee-backup");
    fs::write(&corrupt, b"not a zip").expect("corrupt");
    assert!(validate_backup_archive(
        &fx.pool,
        &fx.runtime,
        &fx.admin,
        corrupt.to_string_lossy().into_owned()
    )
    .await
    .is_err());
}

#[tokio::test]
async fn restore_creates_safety_backup_and_restores_database_and_storage() {
    let fx = fixture().await;
    let backup = create_backup(&fx.pool, &fx.runtime, &fx.admin, false)
        .await
        .expect("backup");

    fs::remove_file(&fx.attachment_path).expect("remove attachment");
    sqlx::query("DELETE FROM document")
        .execute(&fx.pool)
        .await
        .expect("delete docs");

    let result = restore_from_backup(&fx.pool, &fx.runtime, &fx.admin, backup.backup_name)
        .await
        .expect("restore");
    assert!(result.pre_restore_backup_name.starts_with("pre_restore_"));
    assert!(result.restart_required);
    assert!(fx.attachment_path.exists());

    let count: i64 = sqlx::query("SELECT COUNT(*) AS count FROM document")
        .fetch_one(&fx.pool)
        .await
        .expect("count")
        .get("count");
    assert!(count >= 1);
}

#[tokio::test]
async fn retention_cleanup_keeps_newest_backups_and_audit_logs_are_safe() {
    let fx = fixture().await;
    update_backup_settings(
        &fx.pool,
        &fx.runtime,
        &fx.admin,
        BackupSettingsInput {
            destination_path: None,
            schedule_enabled: false,
            schedule_time: "02:00".to_owned(),
            retention_count: 2,
        },
    )
    .await
    .expect("settings");

    create_backup(&fx.pool, &fx.runtime, &fx.admin, false)
        .await
        .expect("backup 1");
    create_backup(&fx.pool, &fx.runtime, &fx.admin, false)
        .await
        .expect("backup 2");
    create_backup(&fx.pool, &fx.runtime, &fx.admin, false)
        .await
        .expect("backup 3");

    let history = list_backup_history(&fx.pool, &fx.runtime, &fx.admin)
        .await
        .expect("history");
    assert_eq!(history.len(), 2);

    let audit_text: String = sqlx::query(
        "SELECT group_concat(description, ' ') AS description FROM audit_log WHERE table_affected = 'backup'",
    )
    .fetch_one(&fx.pool)
    .await
    .expect("audit")
    .get("description");
    assert!(audit_text.contains("backup"));
    assert!(!audit_text.contains("Valid123!"));
    assert!(!audit_text.contains("password_hash"));
    assert!(!audit_text.contains("%PDF-1.4 backup attachment"));
}

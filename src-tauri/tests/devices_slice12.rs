use std::{fs, path::PathBuf};

use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    devices::{mock::MockDeviceProvider, scan_to_intake_with_provider, ScanOptions, ScannerDevice},
    documents::StorageRoot,
    scan_intake::list_scan_intake,
    users::{create_user, UserInput},
};
use sqlx::Row;
use uuid::Uuid;

struct Fixture {
    pool: DbPool,
    admin: String,
    secretary: String,
    storage: StorageRoot,
    storage_dir: PathBuf,
    provider: MockDeviceProvider,
}

async fn fixture() -> Fixture {
    let pool = create_test_pool().await.expect("pool");
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
    let root = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(format!("ovpspee-slice12-{}", Uuid::new_v4()));
    let storage_dir = root.join("storage");
    let storage = StorageRoot::new(storage_dir.clone()).expect("storage");
    Fixture {
        pool,
        admin,
        secretary,
        storage,
        storage_dir,
        provider: MockDeviceProvider {
            scanners: vec![ScannerDevice {
                device_id: "scanner-main".to_owned(),
                name: "Main Scanner".to_owned(),
                manufacturer: Some("UEP".to_owned()),
                connection_type: Some("USB".to_owned()),
                is_available: true,
                status: Some("Ready".to_owned()),
            }],
            printers: vec![],
            scan_fails: false,
            captured_bytes: vec![],
        },
    }
}

fn options() -> ScanOptions {
    ScanOptions {
        dpi: 300,
        color_mode: "color".to_owned(),
        output_format: "png".to_owned(),
        source: "flatbed".to_owned(),
    }
}

#[tokio::test]
async fn secretary_can_scan_mocked_one_page_result_into_intake() {
    let fx = fixture().await;

    let item = scan_to_intake_with_provider(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        "scanner-main",
        options(),
        &fx.provider,
    )
    .await
    .expect("scan");

    assert_eq!(item.status, "Pending");
    assert!(item.stored_relative_path.starts_with("intake/"));
    assert!(!PathBuf::from(&item.stored_relative_path).is_absolute());
    assert!(fx
        .storage
        .resolve_checked(&item.stored_relative_path)
        .expect("path")
        .exists());
}

#[tokio::test]
async fn captured_scan_appears_as_pending_intake_item() {
    let fx = fixture().await;
    let item = scan_to_intake_with_provider(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        "scanner-main",
        options(),
        &fx.provider,
    )
    .await
    .expect("scan");

    let rows = list_scan_intake(&fx.pool, &fx.secretary)
        .await
        .expect("list");

    assert!(rows
        .iter()
        .any(|row| row.scan_intake_id == item.scan_intake_id));
}

#[tokio::test]
async fn invalid_scanner_id_and_invalid_options_are_rejected() {
    let fx = fixture().await;

    assert!(scan_to_intake_with_provider(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        "C:\\scanner",
        options(),
        &fx.provider,
    )
    .await
    .is_err());

    let mut bad = options();
    bad.dpi = 150;
    assert!(scan_to_intake_with_provider(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        "scanner-main",
        bad,
        &fx.provider,
    )
    .await
    .is_err());

    let mut bad = options();
    bad.color_mode = "sepia".to_owned();
    assert!(scan_to_intake_with_provider(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        "scanner-main",
        bad,
        &fx.provider,
    )
    .await
    .is_err());

    let mut bad = options();
    bad.output_format = "pdf".to_owned();
    assert!(scan_to_intake_with_provider(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        "scanner-main",
        bad,
        &fx.provider,
    )
    .await
    .is_err());
}

#[tokio::test]
async fn admin_viewer_and_unauthenticated_users_cannot_scan_into_intake() {
    let fx = fixture().await;

    assert!(scan_to_intake_with_provider(
        &fx.pool,
        &fx.storage,
        &fx.admin,
        "scanner-main",
        options(),
        &fx.provider,
    )
    .await
    .is_err());
    assert!(scan_to_intake_with_provider(
        &fx.pool,
        &fx.storage,
        "",
        "scanner-main",
        options(),
        &fx.provider,
    )
    .await
    .is_err());
}

#[tokio::test]
async fn scanner_unavailable_returns_safe_error() {
    let mut fx = fixture().await;
    fx.provider.scanners[0].is_available = false;

    let err = scan_to_intake_with_provider(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        "scanner-main",
        options(),
        &fx.provider,
    )
    .await
    .expect_err("unavailable");

    assert!(err
        .to_string()
        .contains("Selected scanner is not available"));
}

#[tokio::test]
async fn failed_scan_does_not_leave_db_record_or_file() {
    let mut fx = fixture().await;
    fx.provider.scan_fails = true;

    assert!(scan_to_intake_with_provider(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        "scanner-main",
        options(),
        &fx.provider,
    )
    .await
    .is_err());

    assert!(list_scan_intake(&fx.pool, &fx.secretary)
        .await
        .expect("list")
        .is_empty());
    let intake_dir = fx.storage_dir.join("intake");
    let file_count = if intake_dir.exists() {
        fs::read_dir(intake_dir).expect("read dir").count()
    } else {
        0
    };
    assert_eq!(file_count, 0);
}

#[tokio::test]
async fn scan_capture_audit_log_is_safe() {
    let fx = fixture().await;
    scan_to_intake_with_provider(
        &fx.pool,
        &fx.storage,
        &fx.secretary,
        "scanner-main",
        options(),
        &fx.provider,
    )
    .await
    .expect("scan");

    let rows = sqlx::query("SELECT description FROM audit_log WHERE description = ?")
        .bind("Captured scan into intake")
        .fetch_all(&fx.pool)
        .await
        .expect("audit");

    assert_eq!(rows.len(), 1);
    let text = rows[0].get::<String, _>("description");
    assert!(!text.contains(":\\"));
    assert!(!text.contains("password"));
}

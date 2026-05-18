use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    devices::{
        get_device_settings, list_printers_with_provider, list_scanners_with_provider,
        mock::MockDeviceProvider, update_device_settings_with_provider, DeviceSettingsInput,
        PrinterDevice, ScannerDevice,
    },
    users::{create_user, UserInput},
};
use sqlx::Row;

struct Fixture {
    pool: DbPool,
    admin: String,
    secretary: String,
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
    Fixture {
        pool,
        admin,
        secretary,
        provider: MockDeviceProvider {
            scanners: vec![ScannerDevice {
                device_id: "scanner-main".to_owned(),
                name: "Main Scanner".to_owned(),
                manufacturer: Some("UEP".to_owned()),
                connection_type: Some("USB".to_owned()),
                is_available: true,
                status: Some("Detected".to_owned()),
            }],
            printers: vec![PrinterDevice {
                printer_id: "printer-main".to_owned(),
                name: "Main Printer".to_owned(),
                is_default: true,
                status: "Idle".to_owned(),
                is_available: true,
                is_network: false,
            }],
            ..Default::default()
        },
    }
}

fn input() -> DeviceSettingsInput {
    DeviceSettingsInput {
        default_scanner_id: Some("scanner-main".to_owned()),
        default_printer_id: Some("printer-main".to_owned()),
        scan_default_dpi: 300,
        scan_default_color_mode: "color".to_owned(),
        scan_default_output_format: "png".to_owned(),
    }
}

#[tokio::test]
async fn admin_can_list_scanners_and_printers() {
    let fx = fixture().await;
    let scanners = list_scanners_with_provider(&fx.pool, &fx.admin, &fx.provider)
        .await
        .expect("scanners");
    let printers = list_printers_with_provider(&fx.pool, &fx.admin, &fx.provider)
        .await
        .expect("printers");

    assert_eq!(scanners.len(), 1);
    assert_eq!(scanners[0].device_id, "scanner-main");
    assert_eq!(printers.len(), 1);
    assert!(printers[0].is_default);
}

#[tokio::test]
async fn secretary_can_list_and_read_device_settings() {
    let fx = fixture().await;

    assert!(
        list_scanners_with_provider(&fx.pool, &fx.secretary, &fx.provider)
            .await
            .expect("secretary scanners")
            .iter()
            .any(|device| device.device_id == "scanner-main")
    );
    assert!(
        list_printers_with_provider(&fx.pool, &fx.secretary, &fx.provider)
            .await
            .expect("secretary printers")
            .iter()
            .any(|device| device.printer_id == "printer-main")
    );

    let settings = get_device_settings(&fx.pool, &fx.secretary)
        .await
        .expect("settings");
    assert_eq!(settings.scan_default_dpi, 300);
}

#[tokio::test]
async fn admin_can_update_and_persist_device_settings() {
    let fx = fixture().await;

    let updated = update_device_settings_with_provider(&fx.pool, &fx.admin, input(), &fx.provider)
        .await
        .expect("update");

    assert_eq!(updated.default_scanner_id.as_deref(), Some("scanner-main"));
    assert_eq!(updated.default_printer_id.as_deref(), Some("printer-main"));
    assert_eq!(updated.scan_default_color_mode, "color");

    let reloaded = get_device_settings(&fx.pool, &fx.admin)
        .await
        .expect("reload");
    assert_eq!(reloaded.default_scanner_id.as_deref(), Some("scanner-main"));
    assert_eq!(reloaded.default_printer_id.as_deref(), Some("printer-main"));
}

#[tokio::test]
async fn secretary_and_unauthenticated_users_cannot_update_global_defaults() {
    let fx = fixture().await;

    assert!(
        update_device_settings_with_provider(&fx.pool, &fx.secretary, input(), &fx.provider)
            .await
            .is_err()
    );
    assert!(
        update_device_settings_with_provider(&fx.pool, "", input(), &fx.provider)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn unauthenticated_calls_are_rejected() {
    let fx = fixture().await;

    assert!(list_scanners_with_provider(&fx.pool, "", &fx.provider)
        .await
        .is_err());
    assert!(list_printers_with_provider(&fx.pool, "", &fx.provider)
        .await
        .is_err());
    assert!(get_device_settings(&fx.pool, "").await.is_err());
}

#[tokio::test]
async fn empty_device_lists_are_handled_gracefully() {
    let fx = fixture().await;
    let provider = MockDeviceProvider::default();

    assert!(list_scanners_with_provider(&fx.pool, &fx.admin, &provider)
        .await
        .expect("empty scanners")
        .is_empty());
    assert!(list_printers_with_provider(&fx.pool, &fx.admin, &provider)
        .await
        .expect("empty printers")
        .is_empty());
}

#[tokio::test]
async fn invalid_dpi_color_output_and_device_ids_are_rejected() {
    let fx = fixture().await;
    let mut bad = input();
    bad.scan_default_dpi = 150;
    assert!(
        update_device_settings_with_provider(&fx.pool, &fx.admin, bad, &fx.provider)
            .await
            .is_err()
    );

    let mut bad = input();
    bad.scan_default_color_mode = "sepia".to_owned();
    assert!(
        update_device_settings_with_provider(&fx.pool, &fx.admin, bad, &fx.provider)
            .await
            .is_err()
    );

    let mut bad = input();
    bad.scan_default_output_format = "pdf".to_owned();
    assert!(
        update_device_settings_with_provider(&fx.pool, &fx.admin, bad, &fx.provider)
            .await
            .is_err()
    );

    let mut bad = input();
    bad.default_scanner_id = Some("C:\\driver\\scanner".to_owned());
    assert!(
        update_device_settings_with_provider(&fx.pool, &fx.admin, bad, &fx.provider)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn legacy_pdf_scan_default_reads_as_png() {
    let fx = fixture().await;
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES ('scan_default_output_format', 'pdf', '2026-05-18T00:00:00Z')",
    )
    .execute(&fx.pool)
    .await
    .expect("legacy setting");

    let settings = get_device_settings(&fx.pool, &fx.admin)
        .await
        .expect("settings");

    assert_eq!(settings.scan_default_output_format, "png");

    let stored: String =
        sqlx::query("SELECT value FROM settings WHERE key = 'scan_default_output_format'")
            .fetch_one(&fx.pool)
            .await
            .expect("stored setting")
            .get("value");
    assert_eq!(stored, "png");
}

#[tokio::test]
async fn missing_or_invalid_scan_default_is_written_as_png() {
    let fx = fixture().await;
    sqlx::query("DELETE FROM settings WHERE key = 'scan_default_output_format'")
        .execute(&fx.pool)
        .await
        .expect("remove setting");

    let settings = get_device_settings(&fx.pool, &fx.admin)
        .await
        .expect("missing setting");
    assert_eq!(settings.scan_default_output_format, "png");

    sqlx::query(
        "UPDATE settings SET value = 'gif' WHERE key = 'scan_default_output_format'",
    )
    .execute(&fx.pool)
    .await
    .expect("invalid setting");

    let settings = get_device_settings(&fx.pool, &fx.admin)
        .await
        .expect("invalid setting");
    assert_eq!(settings.scan_default_output_format, "png");

    let stored: String =
        sqlx::query("SELECT value FROM settings WHERE key = 'scan_default_output_format'")
            .fetch_one(&fx.pool)
            .await
            .expect("stored setting")
            .get("value");
    assert_eq!(stored, "png");
}

#[tokio::test]
async fn device_dtos_do_not_expose_absolute_paths() {
    let fx = fixture().await;
    let provider = MockDeviceProvider {
        scanners: vec![
            ScannerDevice {
                device_id: "scanner-ok".to_owned(),
                name: "Scanner OK".to_owned(),
                manufacturer: None,
                connection_type: None,
                is_available: true,
                status: None,
            },
            ScannerDevice {
                device_id: "C:\\unsafe\\scanner".to_owned(),
                name: "Unsafe Scanner".to_owned(),
                manufacturer: None,
                connection_type: None,
                is_available: true,
                status: None,
            },
        ],
        printers: vec![
            PrinterDevice {
                printer_id: "printer-ok".to_owned(),
                name: "Printer OK".to_owned(),
                is_default: false,
                status: "Idle".to_owned(),
                is_available: true,
                is_network: false,
            },
            PrinterDevice {
                printer_id: "D:\\unsafe\\printer".to_owned(),
                name: "Unsafe Printer".to_owned(),
                is_default: false,
                status: "Idle".to_owned(),
                is_available: true,
                is_network: false,
            },
        ],
        ..Default::default()
    };

    let scanners = list_scanners_with_provider(&fx.pool, &fx.admin, &provider)
        .await
        .expect("scanners");
    let printers = list_printers_with_provider(&fx.pool, &fx.admin, &provider)
        .await
        .expect("printers");

    assert_eq!(scanners.len(), 1);
    assert_eq!(scanners[0].device_id, "scanner-ok");
    assert_eq!(printers.len(), 1);
    assert_eq!(printers[0].printer_id, "printer-ok");
}

#[tokio::test]
async fn device_settings_update_writes_safe_audit_log() {
    let fx = fixture().await;

    update_device_settings_with_provider(&fx.pool, &fx.admin, input(), &fx.provider)
        .await
        .expect("update");

    let text: String = sqlx::query(
        "SELECT COALESCE(group_concat(description, ' '), '') AS text FROM audit_log WHERE table_affected = 'settings'",
    )
    .fetch_one(&fx.pool)
    .await
    .expect("audit")
    .get("text");
    assert!(text.contains("Updated device detection defaults"));
    assert!(!text.contains("scanner-main"));
    assert!(!text.contains("printer-main"));
}

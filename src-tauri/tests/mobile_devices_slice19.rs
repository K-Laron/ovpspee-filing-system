use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    documents::StorageRoot,
    mobile_api,
    mobile_devices::{
        create_mobile_device, list_mobile_devices, revoke_mobile_device, validate_mobile_device,
    },
    users::{create_user, UserInput},
};
use sqlx::Row;
use tower::ServiceExt;

struct Fixture {
    pool: DbPool,
    admin: String,
    secretary: String,
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
    Fixture {
        pool,
        admin,
        secretary,
    }
}

#[tokio::test]
async fn mobile_device_table_exists_after_migration() {
    let pool = create_test_pool().await.expect("pool");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'mobile_device'",
    )
    .fetch_one(&pool)
    .await
    .expect("mobile_device table query");

    assert_eq!(count, 1);
}

#[tokio::test]
async fn admin_can_create_list_and_validate_mobile_device_token() {
    let fx = fixture().await;

    let created = create_mobile_device(&fx.pool, &fx.admin, "Records Android")
        .await
        .expect("device");
    assert!(created.device_id.starts_with("device-"));
    assert!(created.device_token.starts_with("ovpspee-"));

    let devices = list_mobile_devices(&fx.pool, &fx.admin)
        .await
        .expect("devices");
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].device_name, "Records Android");
    assert!(devices[0].is_active);

    let stored_hash: String =
        sqlx::query("SELECT token_hash FROM mobile_device WHERE device_id = ?")
            .bind(&created.device_id)
            .fetch_one(&fx.pool)
            .await
            .expect("stored hash")
            .get("token_hash");
    assert_ne!(stored_hash, created.device_token);

    validate_mobile_device(&fx.pool, &created.device_id, &created.device_token)
        .await
        .expect("valid token");
}

#[tokio::test]
async fn secretary_cannot_create_mobile_device_token() {
    let fx = fixture().await;

    assert!(
        create_mobile_device(&fx.pool, &fx.secretary, "Blocked phone")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn revoked_token_cannot_use_mobile_api() {
    let fx = fixture().await;
    let created = create_mobile_device(&fx.pool, &fx.admin, "Review phone")
        .await
        .expect("device");
    let dir = tempfile::tempdir().expect("temp dir");
    let storage = StorageRoot::new(dir.path().join("storage")).expect("storage");
    let app = mobile_api::router(fx.pool.clone(), storage);

    let authorized = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/mobile/health")
                .header("x-ovpspee-device-id", &created.device_id)
                .header("x-ovpspee-device-token", &created.device_token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(authorized.status(), StatusCode::OK);

    revoke_mobile_device(&fx.pool, &fx.admin, &created.device_id)
        .await
        .expect("revoke");

    let revoked = app
        .oneshot(
            Request::builder()
                .uri("/api/mobile/health")
                .header("x-ovpspee-device-id", &created.device_id)
                .header("x-ovpspee-device-token", &created.device_token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(revoked.status(), StatusCode::UNAUTHORIZED);
}

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    mobile_devices::{create_mobile_device, CreatedMobileDevice},
};
use tower::ServiceExt;

async fn pool_with_device() -> (DbPool, CreatedMobileDevice) {
    let pool = create_test_pool().await.expect("pool");
    create_first_admin(&pool, "Admin", "User", "admin1", "Admin123!")
        .await
        .expect("admin");
    let admin = authenticate_user(&pool, "admin1", "Admin123!")
        .await
        .expect("admin login")
        .session_id;
    let device = create_mobile_device(&pool, &admin, "Records Android")
        .await
        .expect("device");
    (pool, device)
}

#[tokio::test]
async fn mobile_api_requires_auth_for_submissions() {
    let (pool, device) = pool_with_device().await;
    let dir = tempfile::tempdir().expect("temp dir");
    let storage = ovpspee_filing_system::documents::StorageRoot::new(dir.path().join("storage"))
        .expect("storage");
    let app = ovpspee_filing_system::mobile_api::router(pool, storage);

    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/mobile/health")
                .header("x-ovpspee-device-id", &device.device_id)
                .header("x-ovpspee-device-token", &device.device_token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/mobile/submissions")
                .header("x-ovpspee-device-id", &device.device_id)
                .header("x-ovpspee-device-token", &device.device_token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn mobile_api_requires_registered_device_token() {
    let (pool, device) = pool_with_device().await;
    let dir = tempfile::tempdir().expect("temp dir");
    let storage = ovpspee_filing_system::documents::StorageRoot::new(dir.path().join("storage"))
        .expect("storage");
    let app = ovpspee_filing_system::mobile_api::router(pool, storage);

    let missing_token = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/mobile/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_token.status(), StatusCode::UNAUTHORIZED);

    let with_token = app
        .oneshot(
            Request::builder()
                .uri("/api/mobile/health")
                .header("x-ovpspee-device-id", &device.device_id)
                .header("x-ovpspee-device-token", &device.device_token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(with_token.status(), StatusCode::OK);
}

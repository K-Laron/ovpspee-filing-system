use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn mobile_api_requires_auth_for_submissions() {
    let pool = ovpspee_filing_system::db::create_test_pool()
        .await
        .expect("pool");
    let dir = tempfile::tempdir().expect("temp dir");
    let storage = ovpspee_filing_system::documents::StorageRoot::new(dir.path().join("storage"))
        .expect("storage");
    let app = ovpspee_filing_system::mobile_api::router(pool, storage);

    let health = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/mobile/health")
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
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn mobile_api_requires_device_token_when_configured() {
    let pool = ovpspee_filing_system::db::create_test_pool()
        .await
        .expect("pool");
    let dir = tempfile::tempdir().expect("temp dir");
    let storage = ovpspee_filing_system::documents::StorageRoot::new(dir.path().join("storage"))
        .expect("storage");
    let app = ovpspee_filing_system::mobile_api::router_with_config(
        pool,
        storage,
        ovpspee_filing_system::mobile_api::MobileApiConfig {
            device_token: Some("office-token".to_owned()),
        },
    );

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
                .header("x-ovpspee-device-token", "office-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(with_token.status(), StatusCode::OK);
}

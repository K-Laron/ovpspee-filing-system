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

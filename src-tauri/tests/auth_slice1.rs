use ovpspee_filing_system::auth::{
    authenticate_user, create_first_admin, first_run_required, hash_password, logout_session,
    validate_password, validate_session, verify_password,
};
use ovpspee_filing_system::db::create_test_pool;

async fn setup_pool() -> sqlx::SqlitePool {
    create_test_pool().await.expect("test db should initialize")
}

#[tokio::test]
async fn password_hash_verifies_and_rejects_wrong_password() {
    let hash = hash_password("Valid123!").expect("password hashes");

    assert!(verify_password("Valid123!", &hash).is_ok());
    assert!(verify_password("Wrong123!", &hash).is_err());
}

#[tokio::test]
async fn first_run_setup_creates_initial_admin() {
    let pool = setup_pool().await;

    assert!(first_run_required(&pool).await.expect("first run checks"));

    create_first_admin(&pool, "Kenneth", "Laron", "kenneth_admin", "Valid123!")
        .await
        .expect("admin created");

    assert!(!first_run_required(&pool).await.expect("first run checks"));
}

#[tokio::test]
async fn weak_password_rejected() {
    assert!(validate_password("short1!").is_err());
    assert!(validate_password("NoDigits!").is_err());
    assert!(validate_password("NoSpecial1").is_err());
}

#[tokio::test]
async fn login_validate_logout_flow() {
    let pool = setup_pool().await;
    create_first_admin(&pool, "Kenneth", "Laron", "kenneth_admin", "Valid123!")
        .await
        .expect("admin created");

    let session = authenticate_user(&pool, "kenneth_admin", "Valid123!")
        .await
        .expect("login succeeds");

    assert_eq!(session.role, "Admin");
    assert_eq!(session.display_name, "Kenneth Laron");

    let validated = validate_session(&pool, &session.session_id)
        .await
        .expect("session valid");
    assert_eq!(validated.user_id, session.user_id);

    logout_session(&pool, &session.session_id)
        .await
        .expect("logout succeeds");

    assert!(validate_session(&pool, &session.session_id).await.is_err());
}

#[tokio::test]
async fn expired_session_rejected() {
    let pool = setup_pool().await;
    create_first_admin(&pool, "Kenneth", "Laron", "kenneth_admin", "Valid123!")
        .await
        .expect("admin created");
    let session = authenticate_user(&pool, "kenneth_admin", "Valid123!")
        .await
        .expect("login succeeds");

    sqlx::query("UPDATE session SET expires_at = '2000-01-01T00:00:00Z' WHERE session_id = ?")
        .bind(&session.session_id)
        .execute(&pool)
        .await
        .expect("expire session");

    assert!(validate_session(&pool, &session.session_id).await.is_err());
}

#[tokio::test]
async fn deactivated_user_cannot_login() {
    let pool = setup_pool().await;
    create_first_admin(&pool, "Kenneth", "Laron", "kenneth_admin", "Valid123!")
        .await
        .expect("admin created");

    sqlx::query("UPDATE user SET is_active = 0 WHERE username = ?")
        .bind("kenneth_admin")
        .execute(&pool)
        .await
        .expect("deactivate user");

    assert!(authenticate_user(&pool, "kenneth_admin", "Valid123!")
        .await
        .is_err());
}

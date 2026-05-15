use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    users::{
        admin_reset_password, change_my_password, create_user, get_my_profile, list_users,
        update_my_profile, update_user, ProfileInput, UserInput, UserUpdateInput,
    },
};

async fn setup_admin(pool: &DbPool) -> String {
    create_first_admin(pool, "Kenneth", "Laron", "kenneth_admin", "Valid123!")
        .await
        .expect("admin created");
    authenticate_user(pool, "kenneth_admin", "Valid123!")
        .await
        .expect("admin login")
        .session_id
}

async fn setup_secretary(pool: &DbPool, admin_session: &str) -> String {
    create_user(
        pool,
        admin_session,
        user("Secretary", "secretary", Some("sec@example.edu.ph")),
    )
    .await
    .expect("secretary created");
    authenticate_user(pool, "secretary", "Valid123!")
        .await
        .expect("secretary login")
        .session_id
}

fn user(role: &str, username: &str, email: Option<&str>) -> UserInput {
    UserInput {
        role: role.to_owned(),
        first_name: "Test".to_owned(),
        middle_name: Some("M".to_owned()),
        last_name: "User".to_owned(),
        username: username.to_owned(),
        email: email.map(str::to_owned),
        contact_number: Some("09171234567".to_owned()),
        address: Some("Campus".to_owned()),
        password: "Valid123!".to_owned(),
    }
}

fn update(username: &str, email: Option<&str>, is_active: bool) -> UserUpdateInput {
    UserUpdateInput {
        role: "Secretary".to_owned(),
        first_name: "Updated".to_owned(),
        middle_name: None,
        last_name: "User".to_owned(),
        username: username.to_owned(),
        email: email.map(str::to_owned),
        contact_number: None,
        address: None,
        is_active,
    }
}

fn profile(email: Option<&str>) -> ProfileInput {
    ProfileInput {
        first_name: "Self".to_owned(),
        middle_name: Some("P".to_owned()),
        last_name: "Updated".to_owned(),
        email: email.map(str::to_owned),
        contact_number: Some("09998887777".to_owned()),
        address: Some("Office".to_owned()),
    }
}

#[tokio::test]
async fn create_user_success() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;

    let user_id = create_user(
        &pool,
        &admin,
        user("Secretary", "secretary", Some("sec@example.edu.ph")),
    )
    .await
    .expect("user created");
    let users = list_users(&pool, &admin, None).await.expect("users listed");

    assert!(users
        .iter()
        .any(|row| row.user_id == user_id && row.username == "secretary"));
}

#[tokio::test]
async fn duplicate_username_rejected() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;

    create_user(&pool, &admin, user("Secretary", "secretary", None))
        .await
        .expect("user created");

    assert!(
        create_user(&pool, &admin, user("Secretary", "SECRETARY", None))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn duplicate_email_rejected_when_provided() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;

    create_user(
        &pool,
        &admin,
        user("Secretary", "one", Some("same@example.edu.ph")),
    )
    .await
    .expect("user created");

    assert!(create_user(
        &pool,
        &admin,
        user("Secretary", "two", Some("SAME@example.edu.ph"))
    )
    .await
    .is_err());
}

#[tokio::test]
async fn weak_password_rejected_for_user_create() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;
    let mut input = user("Secretary", "secretary", None);
    input.password = "weak".to_owned();

    assert!(create_user(&pool, &admin, input).await.is_err());
}

#[tokio::test]
async fn update_user_deactivate_blocks_login() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;
    let user_id = create_user(&pool, &admin, user("Secretary", "secretary", None))
        .await
        .expect("user created");

    update_user(&pool, &admin, user_id, update("secretary", None, false))
        .await
        .expect("user deactivated");

    assert!(authenticate_user(&pool, "secretary", "Valid123!")
        .await
        .is_err());
}

#[tokio::test]
async fn admin_reset_password_changes_login_credential() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;
    let user_id = create_user(&pool, &admin, user("Secretary", "secretary", None))
        .await
        .expect("user created");

    admin_reset_password(&pool, &admin, user_id, "Reset123!")
        .await
        .expect("password reset");

    assert!(authenticate_user(&pool, "secretary", "Valid123!")
        .await
        .is_err());
    assert!(authenticate_user(&pool, "secretary", "Reset123!")
        .await
        .is_ok());
}

#[tokio::test]
async fn non_admin_cannot_manage_users() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;
    let secretary = setup_secretary(&pool, &admin).await;

    assert!(
        create_user(&pool, &secretary, user("Secretary", "blocked", None))
            .await
            .is_err()
    );
    assert!(list_users(&pool, &secretary, None).await.is_err());
}

#[tokio::test]
async fn get_and_update_own_profile() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;
    let secretary = setup_secretary(&pool, &admin).await;

    let before = get_my_profile(&pool, &secretary).await.expect("profile");
    update_my_profile(&pool, &secretary, profile(Some("self@example.edu.ph")))
        .await
        .expect("profile updated");
    let after = get_my_profile(&pool, &secretary).await.expect("profile");

    assert_eq!(before.username, "secretary");
    assert_eq!(after.first_name, "Self");
    assert_eq!(after.email.as_deref(), Some("self@example.edu.ph"));
}

#[tokio::test]
async fn change_password_success_and_wrong_current_rejected() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;
    let secretary = setup_secretary(&pool, &admin).await;

    assert!(
        change_my_password(&pool, &secretary, "Wrong123!", "Newpass123!")
            .await
            .is_err()
    );

    change_my_password(&pool, &secretary, "Valid123!", "Newpass123!")
        .await
        .expect("password changed");

    assert!(authenticate_user(&pool, "secretary", "Valid123!")
        .await
        .is_err());
    assert!(authenticate_user(&pool, "secretary", "Newpass123!")
        .await
        .is_ok());
}

#[tokio::test]
async fn plain_text_password_not_present_in_audit_log() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;
    let user_id = create_user(&pool, &admin, user("Secretary", "secretary", None))
        .await
        .expect("user created");

    admin_reset_password(&pool, &admin, user_id, "Secret987!")
        .await
        .expect("password reset");
    change_my_password(&pool, &admin, "Valid123!", "Ownpass123!")
        .await
        .expect("own password changed");

    let rows = sqlx::query!(
        "SELECT description FROM audit_log WHERE description LIKE '%Secret987%' OR description LIKE '%Ownpass123%'"
    )
    .fetch_all(&pool)
    .await
    .expect("audit checked");

    assert!(rows.is_empty());
}

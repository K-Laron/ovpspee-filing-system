use ovpspee_filing_system::{
    audit_log::{
        get_audit_retention_settings, list_audit_logs, list_my_activity,
        update_audit_retention_settings, AuditLogFilter,
    },
    auth::{authenticate_user, create_first_admin},
    db::{create_test_pool, DbPool},
    documents::{create_document, DocumentInput},
    master_data::{create_category, CategoryInput},
    users::{
        admin_reset_password, change_my_password, create_user, update_user, UserInput,
        UserUpdateInput,
    },
};

struct Fixture {
    pool: DbPool,
    admin: String,
    secretary: String,
    other_secretary: String,
    secretary_user_id: i64,
    category_id: i64,
}

async fn fixture() -> Fixture {
    let pool = create_test_pool().await.expect("pool");
    create_first_admin(&pool, "Kenneth", "Laron", "admin", "Valid123!")
        .await
        .expect("admin created");
    let admin_payload = authenticate_user(&pool, "admin", "Valid123!")
        .await
        .expect("admin login");
    let admin = admin_payload.session_id;
    let secretary_user_id =
        create_user(&pool, &admin, user("secretary", Some("sec@example.edu.ph")))
            .await
            .expect("secretary");
    create_user(
        &pool,
        &admin,
        user("other_secretary", Some("other@example.edu.ph")),
    )
    .await
    .expect("other secretary");
    let secretary = authenticate_user(&pool, "secretary", "Valid123!")
        .await
        .expect("secretary login")
        .session_id;
    let other_secretary = authenticate_user(&pool, "other_secretary", "Valid123!")
        .await
        .expect("other secretary login")
        .session_id;
    let category_id = create_category(
        &pool,
        &admin,
        CategoryInput {
            category_name: "Audit Docs".to_owned(),
            description: None,
            color_code: "#2563EB".to_owned(),
            icon: Some("Folder".to_owned()),
        },
    )
    .await
    .expect("category");
    Fixture {
        pool,
        admin,
        secretary,
        other_secretary,
        secretary_user_id,
        category_id,
    }
}

fn user(username: &str, email: Option<&str>) -> UserInput {
    UserInput {
        role: "Secretary".to_owned(),
        first_name: "Sec".to_owned(),
        middle_name: None,
        last_name: "User".to_owned(),
        username: username.to_owned(),
        email: email.map(str::to_owned),
        contact_number: None,
        address: None,
        password: "Valid123!".to_owned(),
    }
}

fn update(username: &str, email: Option<&str>, active: bool) -> UserUpdateInput {
    UserUpdateInput {
        role: "Secretary".to_owned(),
        first_name: "Sec".to_owned(),
        middle_name: None,
        last_name: "Updated".to_owned(),
        username: username.to_owned(),
        email: email.map(str::to_owned),
        contact_number: None,
        address: None,
        is_active: active,
    }
}

fn doc(fx: &Fixture, name: &str) -> DocumentInput {
    DocumentInput {
        document_name: name.to_owned(),
        category_id: fx.category_id,
        folder_id: None,
        office_id: None,
        date_received: "2026-05-15".to_owned(),
        remarks: Some("Audit slice".to_owned()),
        status: "Filed".to_owned(),
    }
}

#[tokio::test]
async fn admin_can_list_audit_logs() {
    let fx = fixture().await;
    let page = list_audit_logs(&fx.pool, &fx.admin, AuditLogFilter::default())
        .await
        .expect("audit logs");

    assert!(!page.entries.is_empty());
}

#[tokio::test]
async fn non_admin_cannot_list_all_audit_logs() {
    let fx = fixture().await;

    assert!(
        list_audit_logs(&fx.pool, &fx.secretary, AuditLogFilter::default())
            .await
            .is_err()
    );
    assert!(list_audit_logs(&fx.pool, "", AuditLogFilter::default())
        .await
        .is_err());
}

#[tokio::test]
async fn secretary_can_list_only_own_activity() {
    let fx = fixture().await;
    create_document(&fx.pool, &fx.secretary, doc(&fx, "Own Document"))
        .await
        .expect("own doc");
    create_document(&fx.pool, &fx.other_secretary, doc(&fx, "Other Document"))
        .await
        .expect("other doc");

    let page = list_my_activity(&fx.pool, &fx.secretary, AuditLogFilter::default())
        .await
        .expect("my activity");

    assert!(!page.entries.is_empty());
    assert!(page
        .entries
        .iter()
        .all(|entry| entry.actor_user_id == Some(fx.secretary_user_id)));
    assert!(page
        .entries
        .iter()
        .any(|entry| entry.summary == "Created document"));
}

#[tokio::test]
async fn admin_audit_list_includes_user_and_document_events() {
    let fx = fixture().await;
    let doc_id = create_document(&fx.pool, &fx.secretary, doc(&fx, "Audited Doc"))
        .await
        .expect("document");
    update_user(
        &fx.pool,
        &fx.admin,
        fx.secretary_user_id,
        update("secretary", Some("updated@example.edu.ph"), true),
    )
    .await
    .expect("user update");

    let page = list_audit_logs(&fx.pool, &fx.admin, AuditLogFilter::default())
        .await
        .expect("audit logs");

    assert!(page
        .entries
        .iter()
        .any(|entry| entry.entity_type.as_deref() == Some("user")
            && entry.summary == "Updated user account"));
    assert!(page
        .entries
        .iter()
        .any(|entry| entry.entity_type.as_deref() == Some("document")
            && entry.entity_id == Some(doc_id)));
}

#[tokio::test]
async fn audit_list_filters_by_action_actor_and_paginates() {
    let fx = fixture().await;
    create_document(&fx.pool, &fx.secretary, doc(&fx, "Filter Doc"))
        .await
        .expect("document");

    let action_page = list_audit_logs(
        &fx.pool,
        &fx.admin,
        AuditLogFilter {
            action: Some("INSERT".to_owned()),
            limit: Some(5),
            offset: Some(0),
            ..AuditLogFilter::default()
        },
    )
    .await
    .expect("action filter");
    assert!(!action_page.entries.is_empty());
    assert!(action_page
        .entries
        .iter()
        .all(|entry| entry.action == "INSERT"));
    assert_eq!(action_page.limit, 5);
    assert_eq!(action_page.offset, 0);

    let actor_page = list_audit_logs(
        &fx.pool,
        &fx.admin,
        AuditLogFilter {
            actor_user_id: Some(fx.secretary_user_id),
            search: Some("document".to_owned()),
            limit: Some(20),
            ..AuditLogFilter::default()
        },
    )
    .await
    .expect("actor filter");
    assert!(actor_page
        .entries
        .iter()
        .all(|entry| entry.actor_user_id == Some(fx.secretary_user_id)));
    assert!(actor_page
        .entries
        .iter()
        .any(|entry| entry.summary == "Created document"));
}

#[tokio::test]
async fn audit_list_filters_by_date_range() {
    let fx = fixture().await;
    let page = list_audit_logs(
        &fx.pool,
        &fx.admin,
        AuditLogFilter {
            date_from: Some("2000-01-01T00:00:00Z".to_owned()),
            date_to: Some("2999-01-01T00:00:00Z".to_owned()),
            ..AuditLogFilter::default()
        },
    )
    .await
    .expect("date filter");

    assert!(!page.entries.is_empty());
}

#[tokio::test]
async fn password_audit_logs_do_not_expose_plain_password_or_hash() {
    let fx = fixture().await;
    admin_reset_password(&fx.pool, &fx.admin, fx.secretary_user_id, "Secret987!")
        .await
        .expect("reset");
    let secretary = authenticate_user(&fx.pool, "secretary", "Secret987!")
        .await
        .expect("secretary login after reset")
        .session_id;
    change_my_password(&fx.pool, &secretary, "Secret987!", "Changed987!")
        .await
        .expect("change");
    sqlx::query(
        "INSERT INTO audit_log (log_action, table_affected, record_id, description, user_id)
         VALUES ('UPDATE', 'user', ?, 'password=Secret987! password_hash=$argon2id$v=19$m=65536', ?)"
    )
    .bind(fx.secretary_user_id)
    .bind(fx.secretary_user_id)
    .execute(&fx.pool)
    .await
    .expect("legacy unsafe audit row");

    let page = list_audit_logs(&fx.pool, &fx.admin, AuditLogFilter::default())
        .await
        .expect("audit logs");
    let text = page
        .entries
        .iter()
        .map(|entry| entry.summary.clone())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(!text.contains("Secret987!"));
    assert!(!text.contains("Changed987!"));
    assert!(!text.contains("$argon2id"));
    assert!(!text.contains("password_hash"));
}

#[tokio::test]
async fn missing_optional_audit_fields_do_not_crash_viewer() {
    let fx = fixture().await;
    sqlx::query(
        "INSERT INTO audit_log (log_action, description)
         VALUES ('CLEANUP', 'System cleanup without optional fields')",
    )
    .execute(&fx.pool)
    .await
    .expect("audit row");

    let page = list_audit_logs(&fx.pool, &fx.admin, AuditLogFilter::default())
        .await
        .expect("audit logs");

    assert!(page
        .entries
        .iter()
        .any(|entry| entry.actor_user_id.is_none()
            && entry.summary == "System cleanup without optional fields"));
}

#[tokio::test]
async fn retention_setting_defaults_and_validates_range() {
    let fx = fixture().await;
    let settings = get_audit_retention_settings(&fx.pool, &fx.admin)
        .await
        .expect("settings");
    assert_eq!(settings.retention_months, 36);
    assert_eq!(settings.min_months, 24);
    assert_eq!(settings.max_months, 36);
    assert!(settings.cleanup_deferred);

    assert!(update_audit_retention_settings(&fx.pool, &fx.admin, 23)
        .await
        .is_err());
    assert!(update_audit_retention_settings(&fx.pool, &fx.admin, 37)
        .await
        .is_err());
    let updated = update_audit_retention_settings(&fx.pool, &fx.admin, 24)
        .await
        .expect("updated");
    assert_eq!(updated.retention_months, 24);
}

#[tokio::test]
async fn non_admin_cannot_change_retention() {
    let fx = fixture().await;

    assert!(update_audit_retention_settings(&fx.pool, &fx.secretary, 24)
        .await
        .is_err());
    assert!(update_audit_retention_settings(&fx.pool, "", 24)
        .await
        .is_err());
}

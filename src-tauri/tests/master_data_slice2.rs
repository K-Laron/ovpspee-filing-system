use ovpspee_filing_system::{
    auth::{authenticate_user, create_first_admin, hash_password},
    db::{create_test_pool, DbPool},
    master_data::{
        create_category, create_folder, create_office, list_categories, list_offices,
        update_category, CategoryInput, FolderInput, OfficeInput,
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

async fn create_secretary(pool: &DbPool) -> String {
    let role_id = sqlx::query!("SELECT role_id FROM role WHERE role_name = 'Secretary'")
        .fetch_one(pool)
        .await
        .expect("secretary role")
        .role_id;
    let hash = hash_password("Valid123!").expect("hash");
    sqlx::query!(
        "INSERT INTO user (role_id, first_name, last_name, username, password_hash) VALUES (?, ?, ?, ?, ?)",
        role_id,
        "Sec",
        "User",
        "secretary",
        hash
    )
    .execute(pool)
    .await
    .expect("secretary inserted");
    authenticate_user(pool, "secretary", "Valid123!")
        .await
        .expect("secretary login")
        .session_id
}

fn category(name: &str) -> CategoryInput {
    CategoryInput {
        category_name: name.to_owned(),
        description: Some(format!("{name} description")),
        color_code: "#2563EB".to_owned(),
        icon: Some("Folder".to_owned()),
    }
}

fn office(name: &str) -> OfficeInput {
    OfficeInput {
        office_name: name.to_owned(),
        description: Some(format!("{name} description")),
    }
}

#[tokio::test]
async fn create_category_success() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;

    let id = create_category(&pool, &admin, category("BAC"))
        .await
        .expect("category created");

    let rows = list_categories(&pool, &admin, Some(false))
        .await
        .expect("categories listed");
    assert!(rows
        .iter()
        .any(|row| row.category_id == id && row.category_name == "BAC"));
}

#[tokio::test]
async fn duplicate_category_rejected_case_insensitive() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;

    create_category(&pool, &admin, category("BAC"))
        .await
        .expect("category");

    assert!(create_category(&pool, &admin, category("bac"))
        .await
        .is_err());
}

#[tokio::test]
async fn trash_seeded_and_sorted_last() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;
    create_category(&pool, &admin, category("ZZZ Category"))
        .await
        .expect("category");

    let rows = list_categories(&pool, &admin, Some(false))
        .await
        .expect("categories listed");
    let trash = rows.last().expect("last category");

    assert_eq!(trash.category_name, "TRASH");
    assert!(trash.is_system);
}

#[tokio::test]
async fn cannot_edit_system_category() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;
    let trash = list_categories(&pool, &admin, Some(false))
        .await
        .expect("categories listed")
        .into_iter()
        .find(|row| row.category_name == "TRASH")
        .expect("trash exists");

    assert!(update_category(
        &pool,
        &admin,
        trash.category_id,
        category("Trash Edit"),
        true
    )
    .await
    .is_err());
}

#[tokio::test]
async fn cannot_create_folder_under_trash() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;
    let trash = list_categories(&pool, &admin, Some(false))
        .await
        .expect("categories listed")
        .into_iter()
        .find(|row| row.category_name == "TRASH")
        .expect("trash exists");

    let folder = FolderInput {
        category_id: trash.category_id,
        folder_name: "Invalid".to_owned(),
        description: None,
        folder_color: "#64748B".to_owned(),
    };

    assert!(create_folder(&pool, &admin, folder).await.is_err());
}

#[tokio::test]
async fn folder_name_unique_within_category_but_allowed_across_categories() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;
    let bac = create_category(&pool, &admin, category("BAC"))
        .await
        .expect("bac");
    let bor = create_category(&pool, &admin, category("BOR"))
        .await
        .expect("bor");

    let folder = |category_id| FolderInput {
        category_id,
        folder_name: "Minutes".to_owned(),
        description: None,
        folder_color: "#64748B".to_owned(),
    };

    create_folder(&pool, &admin, folder(bac))
        .await
        .expect("folder");
    assert!(create_folder(&pool, &admin, folder(bac)).await.is_err());
    assert!(create_folder(&pool, &admin, folder(bor)).await.is_ok());
}

#[tokio::test]
async fn create_office_success_and_duplicate_rejected() {
    let pool = create_test_pool().await.expect("pool");
    let admin = setup_admin(&pool).await;

    let id = create_office(&pool, &admin, office("OVPSPEE"))
        .await
        .expect("office created");
    let rows = list_offices(&pool, &admin, Some(false))
        .await
        .expect("offices listed");

    assert!(rows
        .iter()
        .any(|row| row.office_id == id && row.office_name == "OVPSPEE"));
    assert!(create_office(&pool, &admin, office("ovpspee"))
        .await
        .is_err());
}

#[tokio::test]
async fn secretary_cannot_manage_master_data() {
    let pool = create_test_pool().await.expect("pool");
    let _admin = setup_admin(&pool).await;
    let secretary = create_secretary(&pool).await;

    assert!(create_category(&pool, &secretary, category("BAC"))
        .await
        .is_err());
}

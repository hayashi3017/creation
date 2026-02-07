use creation_adapter::{model::user::UserTable, repository::RepositoryImpl};
use creation_service::{
    model::user::{LoginUserSchema, RegisterUserSchema},
    repository::user::{UserLoginRepositoryError, UsesUserRepository},
};
use sqlx::PgPool;

#[sqlx::test(fixtures("user_repository"))]
async fn confirm_user_exist_returns_true_for_existing_user(db: PgPool) {
    let repo = RepositoryImpl::<UserTable>::new_test(db).await;
    let body = RegisterUserSchema {
        name: "test_user".to_string(),
        email: "login@example.com".to_string(),
        password: "test_password".to_string(),
    };

    let exists = repo.confirm_user_exist(&body).await.unwrap();
    assert!(exists);
}

#[sqlx::test(fixtures("user_repository"))]
async fn confirm_user_exist_returns_false_for_missing_user(db: PgPool) {
    let repo = RepositoryImpl::<UserTable>::new_test(db).await;
    let body = RegisterUserSchema {
        name: "missing_user".to_string(),
        email: "missing@example.com".to_string(),
        password: "test_password".to_string(),
    };

    let exists = repo.confirm_user_exist(&body).await.unwrap();
    assert!(!exists);
}

#[sqlx::test(fixtures("user_repository"))]
async fn regist_user_hashes_password_and_persists(db: PgPool) {
    let repo = RepositoryImpl::<UserTable>::new_test(db.clone()).await;
    let body = RegisterUserSchema {
        name: "new_user".to_string(),
        email: "new@example.com".to_string(),
        password: "plain_password".to_string(),
    };

    repo.regist_user(body).await.unwrap();

    let stored_password: String = sqlx::query_scalar(
        r#"
            SELECT password FROM users WHERE email = $1
        "#,
    )
    .bind("new@example.com")
    .fetch_one(&db)
    .await
    .unwrap();

    assert_ne!(stored_password, "plain_password");
    assert!(stored_password.starts_with("$argon2"));
}

#[sqlx::test(fixtures("user_repository"))]
async fn login_user_returns_filtered_user(db: PgPool) {
    let repo = RepositoryImpl::<UserTable>::new_test(db).await;
    let body = LoginUserSchema {
        email: "login@example.com".to_string(),
        password: "test_password".to_string(),
    };

    let user = repo.login_user(body).await.unwrap();
    assert_eq!(user.email, "login@example.com");
    assert_eq!(user.name, "login_user");
}

#[sqlx::test(fixtures("user_repository"))]
async fn login_user_returns_wrong_password(db: PgPool) {
    let repo = RepositoryImpl::<UserTable>::new_test(db).await;
    let body = LoginUserSchema {
        email: "login@example.com".to_string(),
        password: "wrong_password".to_string(),
    };

    let err = repo.login_user(body).await.unwrap_err();
    assert!(matches!(err, UserLoginRepositoryError::WrongPassword));
}

#[sqlx::test(fixtures("user_repository"))]
async fn login_user_returns_wrong_user(db: PgPool) {
    let repo = RepositoryImpl::<UserTable>::new_test(db).await;
    let body = LoginUserSchema {
        email: "missing@example.com".to_string(),
        password: "test_password".to_string(),
    };

    let err = repo.login_user(body).await.unwrap_err();
    assert!(matches!(err, UserLoginRepositoryError::WrongUser));
}

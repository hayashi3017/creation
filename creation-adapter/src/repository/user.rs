use anyhow::Result;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use async_trait::async_trait;
use creation_service::{
    repository::user::{ProvidesUserRepository, UserRepository, UsesUserRepository},
    service::user::{ProvidesUserService, UserService},
};
use creation_usecase::usecase::user::{ProvidesUserUsecase, UserUsecase};
use rand_core::OsRng;

use crate::{errors::AppError, model::user::UserTable};

use super::RepositoryImpl;

// to avoid orphan rule, impl trait for struct type.
#[async_trait]
impl UsesUserRepository for RepositoryImpl<UserTable> {
    async fn regist_user(
        &self,
        body: creation_service::model::user::RegisterUserSchema,
    ) -> Result<()> {
        let user_exists: Option<bool> =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = ?)")
                .bind(body.email.to_owned().to_ascii_lowercase())
                .fetch_one(&self.pool.0)
                .await
                .map_err(|e| AppError::Db(e))?;

        if let Some(exists) = user_exists {
            if exists {
                return Err(AppError::DubpicateUser.into());
            }
        }

        let salt = SaltString::generate(&mut OsRng);
        let hashed_password = Argon2::default()
            .hash_password(body.password.as_bytes(), &salt)
            .map_err(|e| AppError::HashingPassword(e))
            .map(|hash| hash.to_string())?;

        // TODO: transaction
        let _user = sqlx::query!(
            r#"
            INSERT INTO users
                (name,email,password)
                VALUES (?, ?, ?)
        "#,
            body.name.to_string(),
            body.email.to_string().to_ascii_lowercase(),
            hashed_password
        )
        .execute(&self.pool.0)
        .await
        .map_err(|e| AppError::Db(e))?;

        Ok(())
    }
}

impl UserRepository for RepositoryImpl<UserTable> {}
impl UserService for RepositoryImpl<UserTable> {}
impl UserUsecase for RepositoryImpl<UserTable> {}

impl ProvidesUserRepository for RepositoryImpl<UserTable> {
    type T = Self;
    fn user_repository(&self) -> &Self::T {
        self
    }
}
impl ProvidesUserService for RepositoryImpl<UserTable> {
    type T = Self;
    fn user_service(&self) -> &Self::T {
        self
    }
}
impl ProvidesUserUsecase for RepositoryImpl<UserTable> {
    type T = Self;
    fn user_usecase(&self) -> &Self::T {
        self
    }
}

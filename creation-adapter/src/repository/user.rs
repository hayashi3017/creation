use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use async_trait::async_trait;
use creation_service::{
    model::user::{FilteredUser, LoginUserSchema, RegisterUserSchema},
    repository::user::{
        ProvidesUserRepository, UserConfirmRepositoryError, UserLoginRepositoryError,
        UserRepository, UserResistRepositoryError, UsesUserRepository,
    },
    service::user::{ProvidesUserService, UserService},
};
use creation_usecase::usecase::user::{ProvidesUserUsecase, UserUsecase};
use rand_core::OsRng;

use crate::model::user::UserTable;

use super::RepositoryImpl;

fn filter_user_record(user: &UserTable) -> FilteredUser {
    FilteredUser {
        id: user.id.to_string(),
        email: user.email.to_owned(),
        name: user.name.to_owned(),
        photo: user.photo.to_owned(),
        role: user.role.to_owned(),
        createdAt: user.created_at.unwrap(),
        updatedAt: user.updated_at.unwrap(),
    }
}

// to avoid orphan rule, impl trait for struct type.
#[async_trait]
impl UsesUserRepository for RepositoryImpl<UserTable> {
    async fn confirm_user_exist(
        &self,
        body: &RegisterUserSchema,
    ) -> Result<bool, UserConfirmRepositoryError> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = ?)")
            .bind(body.email.to_owned().to_ascii_lowercase())
            .fetch_one(&self.pool.0)
            .await
            .map_err(|e| UserConfirmRepositoryError::Db(e))
    }
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserResistRepositoryError> {
        let salt = SaltString::generate(&mut OsRng);
        let hashed_password = Argon2::default()
            .hash_password(body.password.as_bytes(), &salt)
            .map_err(|e| UserResistRepositoryError::HashingPassword(e))
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
        .map_err(|e| UserResistRepositoryError::Db(e))?;

        Ok(())
    }

    async fn login_user(
        &self,
        body: LoginUserSchema,
    ) -> Result<FilteredUser, UserLoginRepositoryError> {
        let user = sqlx::query_as!(
            UserTable,
            r#"
                SELECT
                    id as `id: _`, 
                    name, email, photo,
                    password,
                    role,
                    created_at,
                    tz_created_at as `tz_created_at: _`,
                    updated_at,
                    tz_updated_at as `tz_updated_at: _`
                FROM users WHERE email = ?
            "#,
            body.email.to_ascii_lowercase()
        )
        .fetch_optional(&self.pool.0)
        .await
        .map_err(|e| UserLoginRepositoryError::Db(e))?
        .ok_or_else(|| UserLoginRepositoryError::WrongUser)?;

        let is_valid = match PasswordHash::new(&user.password) {
            Ok(parsed_hash) => Argon2::default()
                .verify_password(body.password.as_bytes(), &parsed_hash)
                .map_or(false, |_| true),
            Err(_) => false,
        };

        if !is_valid {
            return Err(UserLoginRepositoryError::WrongPassword);
        }

        let user = self::filter_user_record(&user);

        Ok(user)
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

use async_trait::async_trait;
use thiserror::Error;

use crate::model::user::{FilteredUser, LoginUserSchema, RegisterUserSchema};

pub trait UserRepository: Send + Sync + 'static {}

#[derive(Debug, Error)]
pub enum UserRepositoryError {
    #[error(transparent)]
    UserConfirmRepositoryError(#[from] UserConfirmRepositoryError),
    #[error(transparent)]
    UserResistRepositoryError(#[from] UserResistRepositoryError),
    #[error(transparent)]
    UserLoginRepositoryError(#[from] UserLoginRepositoryError),
}

#[derive(Debug, Error)]
pub enum UserConfirmRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum UserResistRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("hashing password")]
    HashingPassword(argon2::password_hash::Error),
}

#[derive(Debug, Error)]
pub enum UserLoginRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("wrong password")]
    WrongPassword,
    #[error("wrong user")]
    WrongUser,
}

// TODO: 検証
// ref: https://github.com/http-rs/surf/issues/335#issuecomment-1025118151
impl From<argon2::password_hash::Error> for UserRepositoryError {
    fn from(value: argon2::password_hash::Error) -> Self {
        UserRepositoryError::UserResistRepositoryError(UserResistRepositoryError::HashingPassword(
            value,
        ))
    }
}

#[async_trait]
pub trait UsesUserRepository: Send + Sync + 'static {
    async fn confirm_user_exist(
        &self,
        body: &RegisterUserSchema,
    ) -> Result<bool, UserConfirmRepositoryError>;
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserResistRepositoryError>;
    async fn login_user(
        &self,
        body: LoginUserSchema,
    ) -> Result<FilteredUser, UserLoginRepositoryError>;
}

// why this is not error?
// #[async_trait]
// impl<T: UserRepository> UsesUserRepository for T {
//     async fn regist_user(&self, body: RegisterUserSchema) -> Result<()> {
//         self.regist_user(body).await
//     }
// }

pub trait ProvidesUserRepository: Send + Sync + 'static {
    type T: UsesUserRepository;
    fn user_repository(&self) -> &Self::T;
}

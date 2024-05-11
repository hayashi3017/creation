use async_trait::async_trait;
use thiserror::Error;

use crate::{
    model::user::RegisterUserSchema,
    repository::user::{ProvidesUserRepository, UserRepositoryError, UsesUserRepository},
};

#[async_trait]
pub trait UserService: ProvidesUserRepository {}

#[derive(Debug, Error)]
pub enum UserServiceError {
    #[error(transparent)]
    UserRepositoryError(#[from] UserRepositoryError),
}

#[async_trait]
pub trait UsesUserService {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserServiceError>;
}

#[async_trait]
impl<T: UserService> UsesUserService for T {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserServiceError> {
        match self.user_repository().regist_user(body).await {
            Err(err) => Err(UserServiceError::UserRepositoryError(
                UserRepositoryError::UserResistError(err),
            )),
            Ok(()) => Ok(()),
        }
    }
}

pub trait ProvidesUserService: Send + Sync + 'static {
    type T: UsesUserService;
    fn user_service(&self) -> &Self::T;
}

use async_trait::async_trait;
use thiserror::Error;

use super::{map_service_result, map_service_result_unit};

use crate::{
    model::user::{FilteredUser, LoginUserSchema, RegisterUserSchema},
    repository::user::{
        ProvidesUserRepository, UserConfirmRepositoryError, UserLoginRepositoryError,
        UserResistRepositoryError, UsesUserRepository,
    },
};

#[async_trait]
pub trait UserService: ProvidesUserRepository {}

#[derive(Debug, Error)]
pub enum UserServiceError {
    #[error(transparent)]
    UserRegistServiceError(#[from] UserRegistServiceError),
    #[error(transparent)]
    UserLoginServiceError(#[from] UserLoginServiceError),
}

#[derive(Debug, Error)]
pub enum UserRegistServiceError {
    #[error(transparent)]
    UserConfirmRepositoryError(#[from] UserConfirmRepositoryError),
    #[error(transparent)]
    UserResistRepositoryError(#[from] UserResistRepositoryError),
    #[error("duplicate user")]
    DubpicateUser,
}

#[derive(Debug, Error)]
pub enum UserLoginServiceError {
    #[error(transparent)]
    UserLoginRepositoryError(#[from] UserLoginRepositoryError),
}

#[async_trait]
pub trait UsesUserService {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserRegistServiceError>;
    async fn login_user(
        &self,
        body: LoginUserSchema,
    ) -> Result<FilteredUser, UserLoginServiceError>;
}

#[async_trait]
impl<T: UserService> UsesUserService for T {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserRegistServiceError> {
        let user_exists = self.user_repository().confirm_user_exist(&body).await?;
        if user_exists {
            return Err(UserRegistServiceError::DubpicateUser);
        }

        map_service_result_unit!(
            self.user_repository().regist_user(body),
            UserRegistServiceError::UserResistRepositoryError
        )
    }
    async fn login_user(
        &self,
        body: LoginUserSchema,
    ) -> Result<FilteredUser, UserLoginServiceError> {
        map_service_result!(
            self.user_repository().login_user(body),
            UserLoginServiceError::UserLoginRepositoryError
        )
    }
}

pub trait ProvidesUserService: Send + Sync + 'static {
    type T: UsesUserService;
    fn user_service(&self) -> &Self::T;
}

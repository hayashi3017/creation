use async_trait::async_trait;
use creation_service::{
    model::user::{FilteredUser, LoginUserSchema, RegisterUserSchema},
    service::user::{
        ProvidesUserService, UserLoginServiceError, UserRegistServiceError, UsesUserService,
    },
};
use thiserror::Error;

use super::{map_usecase_result, map_usecase_result_unit};

#[async_trait]
pub trait UserUsecase: ProvidesUserService {}

#[derive(Debug, Error)]
pub enum UserUsecaseError {
    #[error(transparent)]
    UserRegistUsecaseError(#[from] UserRegistUsecaseError),
    #[error(transparent)]
    UserLoginUsecaseError(#[from] UserLoginUsecaseError),
}

#[derive(Debug, Error)]
pub enum UserRegistUsecaseError {
    #[error(transparent)]
    UserRegistServiceError(#[from] UserRegistServiceError),
}

#[derive(Debug, Error)]
pub enum UserLoginUsecaseError {
    #[error(transparent)]
    UserLoginServiceError(#[from] UserLoginServiceError),
}

#[async_trait]
pub trait UsesUserUsecase {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserRegistUsecaseError>;
    async fn login_user(
        &self,
        body: LoginUserSchema,
    ) -> Result<FilteredUser, UserLoginUsecaseError>;
}

#[async_trait]
impl<T: UserUsecase> UsesUserUsecase for T {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserRegistUsecaseError> {
        map_usecase_result_unit!(
            self.user_service().regist_user(body),
            UserRegistUsecaseError::UserRegistServiceError
        )
    }
    async fn login_user(
        &self,
        body: LoginUserSchema,
    ) -> Result<FilteredUser, UserLoginUsecaseError> {
        map_usecase_result!(
            self.user_service().login_user(body),
            UserLoginUsecaseError::UserLoginServiceError
        )
    }
}

pub trait ProvidesUserUsecase: Send + Sync + 'static {
    type T: UsesUserUsecase + Sized;
    fn user_usecase(&self) -> &Self::T;
}

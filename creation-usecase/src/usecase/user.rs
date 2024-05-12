use async_trait::async_trait;
use creation_service::{
    model::user::{FilteredUser, LoginUserSchema, RegisterUserSchema},
    service::user::{
        ProvidesUserService, UserLoginServiceError, UserRegistServiceError, UsesUserService,
    },
};
use thiserror::Error;

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
        match self.user_service().regist_user(body).await {
            Err(err) => Err(UserRegistUsecaseError::UserRegistServiceError(err)),
            Ok(()) => Ok(()),
        }
    }
    async fn login_user(
        &self,
        body: LoginUserSchema,
    ) -> Result<FilteredUser, UserLoginUsecaseError> {
        match self.user_service().login_user(body).await {
            Err(err) => Err(UserLoginUsecaseError::UserLoginServiceError(err)),
            Ok(val) => Ok(val),
        }
    }
}

pub trait ProvidesUserUsecase: Send + Sync + 'static {
    type T: UsesUserUsecase + Sized;
    fn user_usecase(&self) -> &Self::T;
}

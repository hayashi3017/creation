use async_trait::async_trait;
use creation_service::{
    model::user::{FilteredUser, LoginUserSchema, RegisterUserSchema},
    service::user::{ProvidesUserService, UserServiceError, UsesUserService},
};
use thiserror::Error;

#[async_trait]
pub trait UserUsecase: ProvidesUserService {}

#[derive(Debug, Error)]
pub enum UserUsecaseError {
    #[error(transparent)]
    UserServiceError(#[from] UserServiceError),
}

#[async_trait]
pub trait UsesUserUsecase {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserUsecaseError>;
    async fn login_user(&self, body: LoginUserSchema) -> Result<FilteredUser, UserUsecaseError>;
}

#[async_trait]
impl<T: UserUsecase> UsesUserUsecase for T {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserUsecaseError> {
        match self.user_service().regist_user(body).await {
            Err(err) => Err(UserUsecaseError::UserServiceError(err)),
            Ok(()) => Ok(()),
        }
    }
    async fn login_user(&self, body: LoginUserSchema) -> Result<FilteredUser, UserUsecaseError> {
        match self.user_service().login_user(body).await {
            Err(err) => Err(UserUsecaseError::UserServiceError(err)),
            Ok(val) => Ok(val),
        }
    }
}

pub trait ProvidesUserUsecase: Send + Sync + 'static {
    type T: UsesUserUsecase + Sized;
    fn user_usecase(&self) -> &Self::T;
}

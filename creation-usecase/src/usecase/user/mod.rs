pub mod login_user;
pub mod regist_user;

use async_trait::async_trait;
use creation_service::service::user::ProvidesUserService;
use thiserror::Error;

use super::map_usecase_result;
use super::map_usecase_result_unit;

pub use login_user::{UserLoginUsecaseError, UsesLoginUserUsecase};
pub use regist_user::{UserRegistUsecaseError, UsesRegistUserUsecase};

#[async_trait]
pub trait UserUsecase: ProvidesUserService {}

#[derive(Debug, Error)]
pub enum UserUsecaseError {
    #[error(transparent)]
    UserRegistUsecaseError(#[from] UserRegistUsecaseError),
    #[error(transparent)]
    UserLoginUsecaseError(#[from] UserLoginUsecaseError),
}

#[async_trait]
pub trait UsesUserUsecase: UsesRegistUserUsecase + UsesLoginUserUsecase {
    async fn regist_user(
        &self,
        body: creation_service::model::user::RegisterUserSchema,
    ) -> Result<(), UserRegistUsecaseError> {
        UsesRegistUserUsecase::regist_user(self, body).await
    }

    async fn login_user(
        &self,
        body: creation_service::model::user::LoginUserSchema,
    ) -> Result<creation_service::model::user::FilteredUser, UserLoginUsecaseError> {
        UsesLoginUserUsecase::login_user(self, body).await
    }
}

impl<T> UsesUserUsecase for T where T: UsesRegistUserUsecase + UsesLoginUserUsecase {}

pub trait ProvidesUserUsecase: Send + Sync + 'static {
    type T: UsesUserUsecase + Sized;
    fn user_usecase(&self) -> &Self::T;
}

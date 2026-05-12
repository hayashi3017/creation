use async_trait::async_trait;
use creation_service::{
    model::user::RegisterUserSchema,
    service::user::{UserRegistServiceError, UsesUserService},
};
use thiserror::Error;

use super::map_usecase_result_unit;
use super::UserUsecase;

#[derive(Debug, Error)]
pub enum UserRegistUsecaseError {
    #[error(transparent)]
    UserRegistServiceError(#[from] UserRegistServiceError),
}

#[async_trait]
pub trait UsesRegistUserUsecase {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserRegistUsecaseError>;
}

#[async_trait]
impl<T: UserUsecase> UsesRegistUserUsecase for T {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<(), UserRegistUsecaseError> {
        map_usecase_result_unit!(
            self.user_service().regist_user(body),
            UserRegistUsecaseError::UserRegistServiceError
        )
    }
}

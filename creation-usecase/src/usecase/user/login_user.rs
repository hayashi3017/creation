use async_trait::async_trait;
use creation_service::{
    model::user::{FilteredUser, LoginUserSchema},
    service::user::{UserLoginServiceError, UsesUserService},
};
use thiserror::Error;

use super::map_usecase_result;
use super::UserUsecase;

#[derive(Debug, Error)]
pub enum UserLoginUsecaseError {
    #[error(transparent)]
    UserLoginServiceError(#[from] UserLoginServiceError),
}

#[async_trait]
pub trait UsesLoginUserUsecase {
    async fn login_user(
        &self,
        body: LoginUserSchema,
    ) -> Result<FilteredUser, UserLoginUsecaseError>;
}

#[async_trait]
impl<T: UserUsecase> UsesLoginUserUsecase for T {
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

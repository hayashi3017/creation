use anyhow::Result;
use async_trait::async_trait;
use creation_service::{
    model::user::RegisterUserSchema,
    service::user::{ProvidesUserService, UsesUserService},
};

#[async_trait]
pub trait UserUsecase: ProvidesUserService {}

#[async_trait]
pub trait UsesUserUsecase {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<()>;
}

#[async_trait]
impl<T: UserUsecase> UsesUserUsecase for T {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<()> {
        self.user_service().regist_user(body).await
    }
}

pub trait ProvidesUserUsecase: Send + Sync + 'static {
    type T: UsesUserUsecase + Sized;
    fn user_usecase(&self) -> &Self::T;
}

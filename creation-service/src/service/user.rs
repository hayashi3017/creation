use anyhow::Result;
use async_trait::async_trait;

use crate::{
    model::user::RegisterUserSchema,
    repository::user::{ProvidesUserRepository, UsesUserRepository},
};

#[async_trait]
pub trait UserService: ProvidesUserRepository {}

#[async_trait]
pub trait UsesUserService {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<()>;
}

#[async_trait]
impl<T: UserService> UsesUserService for T {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<()> {
        self.user_repository().regist_user(body).await
    }
}

pub trait ProvidesUserService: Send + Sync + 'static {
    type T: UsesUserService;
    fn user_service(&self) -> &Self::T;
}

use anyhow::Result;
use async_trait::async_trait;

use crate::model::user::RegisterUserSchema;

pub trait UserRepository: Send + Sync + 'static {}

#[async_trait]
pub trait UsesUserRepository: Send + Sync + 'static {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<()>;
}

// why this is not error?
// #[async_trait]
// impl<T: UserRepository> UsesUserRepository for T {
//     async fn regist_user(&self, body: RegisterUserSchema) -> Result<()> {
//         self.regist_user(body).await
//     }
// }

pub trait ProvidesUserRepository: Send + Sync + 'static {
    type T: UsesUserRepository;
    fn user_repository(&self) -> &Self::T;
}

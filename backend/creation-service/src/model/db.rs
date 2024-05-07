use anyhow::Result;
use async_trait::async_trait;

use super::user::RegisterUserSchema;

pub trait Database: Send + Sync + 'static {}

#[async_trait]
pub trait UsesDatabase: Send + Sync + 'static {
    async fn regist_user(&self, body: RegisterUserSchema) -> Result<()>;
}

pub trait ProvidesDatabase: Send + Sync + 'static {
    type T: UsesDatabase;
    fn database(&self) -> &Self::T;
}

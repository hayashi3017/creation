use async_trait::async_trait;
use thiserror::Error;

use crate::model::world::{
    CreateWorldSchema, DeleteWorldSchema, GetWorldSchema, GetWorldsSchema, UpdateWorldSchema, World,
};

pub trait WorldRepository: Send + Sync + 'static {}

#[derive(Debug, Error)]
pub enum GetWorldsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum GetWorldRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum CreateWorldRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum UpdateWorldRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteWorldRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesWorldRepository: Send + Sync + 'static {
    async fn get_worlds(
        &self,
        body: GetWorldsSchema,
    ) -> Result<Vec<World>, GetWorldsRepositoryError>;
    async fn get_world(
        &self,
        body: GetWorldSchema,
    ) -> Result<Option<World>, GetWorldRepositoryError>;
    async fn create_world(
        &self,
        body: CreateWorldSchema,
    ) -> Result<World, CreateWorldRepositoryError>;
    async fn update_world(
        &self,
        body: UpdateWorldSchema,
    ) -> Result<World, UpdateWorldRepositoryError>;
    async fn delete_world(&self, body: DeleteWorldSchema)
        -> Result<(), DeleteWorldRepositoryError>;
}

pub trait ProvidesWorldRepository: Send + Sync + 'static {
    type T: UsesWorldRepository;
    fn world_repository(&self) -> &Self::T;
}

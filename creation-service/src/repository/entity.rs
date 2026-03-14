use async_trait::async_trait;
use thiserror::Error;

use crate::model::entity::{
    CreateEntitySchema, DeleteEntitySchema, Entity, GetEntitiesSchema, UpdateEntitySchema,
};

pub trait EntityRepository: Send + Sync + 'static {}

#[derive(Debug, Error)]
pub enum EntityRepositoryError {
    #[error(transparent)]
    GetEntitiesRepositoryError(#[from] GetEntitiesRepositoryError),
    #[error(transparent)]
    CreateEntityRepositoryError(#[from] CreateEntityRepositoryError),
    #[error(transparent)]
    UpdateEntityRepositoryError(#[from] UpdateEntityRepositoryError),
    #[error(transparent)]
    DeleteEntityRepositoryError(#[from] DeleteEntityRepositoryError),
}

#[derive(Debug, Error)]
pub enum GetEntitiesRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum CreateEntityRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum UpdateEntityRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum DeleteEntityRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[async_trait]
pub trait UsesEntityRepository: Send + Sync + 'static {
    async fn get_entities(
        &self,
        body: GetEntitiesSchema,
    ) -> Result<Vec<Entity>, GetEntitiesRepositoryError>;
    async fn create_entity(
        &self,
        body: CreateEntitySchema,
    ) -> Result<(), CreateEntityRepositoryError>;
    async fn update_entity(
        &self,
        body: UpdateEntitySchema,
    ) -> Result<(), UpdateEntityRepositoryError>;
    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<(), DeleteEntityRepositoryError>;
}

pub trait ProvidesEntityRepository: Send + Sync + 'static {
    type T: UsesEntityRepository;
    fn entity_repository(&self) -> &Self::T;
}

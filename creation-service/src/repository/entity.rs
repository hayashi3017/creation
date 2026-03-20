use async_trait::async_trait;
use thiserror::Error;

use crate::model::entity::{
    CreateEntitySchema, DeleteEntitySchema, Entity, GetEntitiesSchema,
    LoadActiveEntitiesByDiagramIdsSchema, LoadActiveEntityIdsSchema, LoadSeedEntitiesSchema,
    SeedEntity, UpdateEntitySchema,
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
    #[error(transparent)]
    LoadSeedEntitiesRepositoryError(#[from] LoadSeedEntitiesRepositoryError),
    #[error(transparent)]
    LoadActiveEntityIdsRepositoryError(#[from] LoadActiveEntityIdsRepositoryError),
    #[error(transparent)]
    LoadActiveEntitiesByDiagramIdsRepositoryError(
        #[from] LoadActiveEntitiesByDiagramIdsRepositoryError,
    ),
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
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteEntityRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum LoadSeedEntitiesRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum LoadActiveEntityIdsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum LoadActiveEntitiesByDiagramIdsRepositoryError {
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
    ) -> Result<usize, CreateEntityRepositoryError>;
    async fn update_entity(
        &self,
        body: UpdateEntitySchema,
    ) -> Result<(), UpdateEntityRepositoryError>;
    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<usize, DeleteEntityRepositoryError>;
    async fn load_seed_entities(
        &self,
        body: LoadSeedEntitiesSchema,
    ) -> Result<Vec<SeedEntity>, LoadSeedEntitiesRepositoryError>;
    async fn load_active_entity_ids(
        &self,
        body: LoadActiveEntityIdsSchema,
    ) -> Result<Vec<usize>, LoadActiveEntityIdsRepositoryError>;
    async fn load_active_entities_by_diagram_ids(
        &self,
        body: LoadActiveEntitiesByDiagramIdsSchema,
    ) -> Result<Vec<SeedEntity>, LoadActiveEntitiesByDiagramIdsRepositoryError>;
}

pub trait ProvidesEntityRepository: Send + Sync + 'static {
    type T: UsesEntityRepository;
    fn entity_repository(&self) -> &Self::T;
}

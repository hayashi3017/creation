use async_trait::async_trait;
use thiserror::Error;

use crate::model::entity::{
    CreateDiagramEntityMembershipSchema, CreateEntitySchema, DeleteDiagramEntityMembershipsSchema,
    DeleteEntitySchema, Entity, GetEntitiesSchema, LoadActiveEntitiesByDiagramIdsSchema,
    LoadActiveEntityIdsSchema, LoadEntitiesByDiagramIdsSchema, LoadEntitiesByWorldSchema,
    LoadSeedEntitiesSchema, SeedEntity, UpdateEntitySchema,
};

pub trait EntityRepository: Send + Sync + 'static {}

#[derive(Debug, Error)]
pub enum EntityRepositoryError {
    #[error(transparent)]
    GetEntitiesRepositoryError(#[from] GetEntitiesRepositoryError),
    #[error(transparent)]
    CreateEntityRepositoryError(#[from] CreateEntityRepositoryError),
    #[error(transparent)]
    CreateDiagramEntityMembershipRepositoryError(
        #[from] CreateDiagramEntityMembershipRepositoryError,
    ),
    #[error(transparent)]
    UpdateEntityRepositoryError(#[from] UpdateEntityRepositoryError),
    #[error(transparent)]
    DeleteEntityRepositoryError(#[from] DeleteEntityRepositoryError),
    #[error(transparent)]
    DeleteDiagramEntityMembershipsRepositoryError(
        #[from] DeleteDiagramEntityMembershipsRepositoryError,
    ),
    #[error(transparent)]
    LoadSeedEntitiesRepositoryError(#[from] LoadSeedEntitiesRepositoryError),
    #[error(transparent)]
    LoadActiveEntityIdsRepositoryError(#[from] LoadActiveEntityIdsRepositoryError),
    #[error(transparent)]
    LoadActiveEntitiesByDiagramIdsRepositoryError(
        #[from] LoadActiveEntitiesByDiagramIdsRepositoryError,
    ),
    #[error(transparent)]
    LoadEntitiesByDiagramIdsRepositoryError(#[from] LoadEntitiesByDiagramIdsRepositoryError),
    #[error(transparent)]
    LoadEntitiesByWorldRepositoryError(#[from] LoadEntitiesByWorldRepositoryError),
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
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum CreateDiagramEntityMembershipRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
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
pub enum DeleteDiagramEntityMembershipsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
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

#[derive(Debug, Error)]
pub enum LoadEntitiesByDiagramIdsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum LoadEntitiesByWorldRepositoryError {
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
    async fn create_diagram_entity_membership(
        &self,
        body: CreateDiagramEntityMembershipSchema,
    ) -> Result<(), CreateDiagramEntityMembershipRepositoryError>;
    async fn update_entity(
        &self,
        body: UpdateEntitySchema,
    ) -> Result<(), UpdateEntityRepositoryError>;
    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<usize, DeleteEntityRepositoryError>;
    async fn delete_diagram_entity_memberships(
        &self,
        body: DeleteDiagramEntityMembershipsSchema,
    ) -> Result<Vec<usize>, DeleteDiagramEntityMembershipsRepositoryError>;
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
    async fn load_entities_by_diagram_ids(
        &self,
        body: LoadEntitiesByDiagramIdsSchema,
    ) -> Result<Vec<Entity>, LoadEntitiesByDiagramIdsRepositoryError>;
    async fn load_entities_by_world(
        &self,
        body: LoadEntitiesByWorldSchema,
    ) -> Result<Vec<Entity>, LoadEntitiesByWorldRepositoryError>;
}

pub trait ProvidesEntityRepository: Send + Sync + 'static {
    type T: UsesEntityRepository;
    fn entity_repository(&self) -> &Self::T;
}

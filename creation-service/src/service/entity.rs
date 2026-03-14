use async_trait::async_trait;
use thiserror::Error;

use super::{map_service_result, map_service_result_unit};

use crate::{
    model::entity::{
        CreateEntitySchema, DeleteEntitySchema, Entity, GetEntitiesSchema, UpdateEntitySchema,
    },
    repository::entity::{
        CreateEntityRepositoryError, DeleteEntityRepositoryError, GetEntitiesRepositoryError,
        ProvidesEntityRepository, UpdateEntityRepositoryError, UsesEntityRepository,
    },
};

#[async_trait]
pub trait EntityService: ProvidesEntityRepository {}

#[derive(Debug, Error)]
pub enum EntityServiceError {
    #[error(transparent)]
    GetEntitiesServiceError(#[from] GetEntitiesServiceError),
    #[error(transparent)]
    CreateEntityServiceError(#[from] CreateEntityServiceError),
    #[error(transparent)]
    UpdateEntityServiceError(#[from] UpdateEntityServiceError),
    #[error(transparent)]
    DeleteEntityServiceError(#[from] DeleteEntityServiceError),
}

#[derive(Debug, Error)]
pub enum GetEntitiesServiceError {
    #[error(transparent)]
    GetEntitiesRepositoryError(#[from] GetEntitiesRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[derive(Debug, Error)]
pub enum CreateEntityServiceError {
    #[error(transparent)]
    CreateEntityRepositoryError(#[from] CreateEntityRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[derive(Debug, Error)]
pub enum UpdateEntityServiceError {
    #[error(transparent)]
    UpdateEntityRepositoryError(#[from] UpdateEntityRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteEntityServiceError {
    #[error(transparent)]
    DeleteEntityRepositoryError(#[from] DeleteEntityRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesEntityService {
    async fn get_entities(
        &self,
        body: GetEntitiesSchema,
    ) -> Result<Vec<Entity>, GetEntitiesServiceError>;
    async fn create_entity(&self, body: CreateEntitySchema)
        -> Result<(), CreateEntityServiceError>;
    async fn update_entity(&self, body: UpdateEntitySchema)
        -> Result<(), UpdateEntityServiceError>;
    async fn delete_entity(&self, body: DeleteEntitySchema)
        -> Result<(), DeleteEntityServiceError>;
}

#[async_trait]
impl<T: EntityService> UsesEntityService for T {
    async fn get_entities(
        &self,
        body: GetEntitiesSchema,
    ) -> Result<Vec<Entity>, GetEntitiesServiceError> {
        if body.diagram_id == 0 {
            return Err(GetEntitiesServiceError::InvalidParams);
        }

        map_service_result!(
            self.entity_repository().get_entities(body),
            GetEntitiesServiceError::GetEntitiesRepositoryError
        )
    }

    async fn create_entity(
        &self,
        body: CreateEntitySchema,
    ) -> Result<(), CreateEntityServiceError> {
        if body.diagram_id == 0 || body.name.is_empty() {
            return Err(CreateEntityServiceError::InvalidParams);
        }

        map_service_result_unit!(
            self.entity_repository().create_entity(body),
            CreateEntityServiceError::CreateEntityRepositoryError
        )
    }

    async fn update_entity(
        &self,
        body: UpdateEntitySchema,
    ) -> Result<(), UpdateEntityServiceError> {
        if body.id == 0 || body.diagram_id == 0 || body.name.is_empty() {
            return Err(UpdateEntityServiceError::InvalidParams);
        }

        match self.entity_repository().update_entity(body).await {
            Ok(()) => Ok(()),
            Err(UpdateEntityRepositoryError::NotFound) => Err(UpdateEntityServiceError::NotFound),
            Err(err) => Err(UpdateEntityServiceError::UpdateEntityRepositoryError(err)),
        }
    }

    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<(), DeleteEntityServiceError> {
        if body.id == 0 {
            return Err(DeleteEntityServiceError::InvalidParams);
        }

        match self.entity_repository().delete_entity(body).await {
            Ok(()) => Ok(()),
            Err(DeleteEntityRepositoryError::NotFound) => Err(DeleteEntityServiceError::NotFound),
            Err(err) => Err(DeleteEntityServiceError::DeleteEntityRepositoryError(err)),
        }
    }
}

pub trait ProvidesEntityService: Send + Sync + 'static {
    type T: EntityService;
    fn entity_service(&self) -> &Self::T;
}

use async_trait::async_trait;
use thiserror::Error;

use super::{map_service_result, normalize_name, normalize_optional_text};

use crate::{
    model::entity::{
        CreateEntitySchema, DeleteEntitySchema, Entity, GetEntitiesSchema, UpdateEntitySchema,
        ENTITY_NAME_MAX_CHARS,
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
    async fn create_entity(
        &self,
        body: CreateEntitySchema,
    ) -> Result<usize, CreateEntityServiceError>;
    async fn update_entity(&self, body: UpdateEntitySchema)
        -> Result<(), UpdateEntityServiceError>;
    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<usize, DeleteEntityServiceError>;
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
    ) -> Result<usize, CreateEntityServiceError> {
        let Some(body) = prepare_create_entity(body) else {
            return Err(CreateEntityServiceError::InvalidParams);
        };

        map_service_result!(
            self.entity_repository().create_entity(body),
            CreateEntityServiceError::CreateEntityRepositoryError
        )
    }

    async fn update_entity(
        &self,
        body: UpdateEntitySchema,
    ) -> Result<(), UpdateEntityServiceError> {
        let Some(body) = prepare_update_entity(body) else {
            return Err(UpdateEntityServiceError::InvalidParams);
        };

        match self.entity_repository().update_entity(body).await {
            Ok(()) => Ok(()),
            Err(UpdateEntityRepositoryError::NotFound) => Err(UpdateEntityServiceError::NotFound),
            Err(err) => Err(UpdateEntityServiceError::UpdateEntityRepositoryError(err)),
        }
    }

    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<usize, DeleteEntityServiceError> {
        let Some(body) = prepare_delete_entity(body) else {
            return Err(DeleteEntityServiceError::InvalidParams);
        };

        match self.entity_repository().delete_entity(body).await {
            Ok(diagram_id) => Ok(diagram_id),
            Err(DeleteEntityRepositoryError::NotFound) => Err(DeleteEntityServiceError::NotFound),
            Err(err) => Err(DeleteEntityServiceError::DeleteEntityRepositoryError(err)),
        }
    }
}

pub fn prepare_create_entity(body: CreateEntitySchema) -> Option<CreateEntitySchema> {
    if body.diagram_id == 0 {
        return None;
    }

    let name = normalize_name(&body.name, ENTITY_NAME_MAX_CHARS)?;

    Some(CreateEntitySchema {
        diagram_id: body.diagram_id,
        kind: body.kind,
        name,
        description: normalize_optional_text(body.description),
    })
}

pub fn prepare_update_entity(body: UpdateEntitySchema) -> Option<UpdateEntitySchema> {
    if body.id == 0 || body.diagram_id == 0 {
        return None;
    }

    let name = normalize_name(&body.name, ENTITY_NAME_MAX_CHARS)?;

    Some(UpdateEntitySchema {
        id: body.id,
        diagram_id: body.diagram_id,
        kind: body.kind,
        name,
        description: normalize_optional_text(body.description),
    })
}

pub fn prepare_delete_entity(body: DeleteEntitySchema) -> Option<DeleteEntitySchema> {
    if body.id == 0 {
        None
    } else {
        Some(body)
    }
}

pub trait ProvidesEntityService: Send + Sync + 'static {
    type T: EntityService;
    fn entity_service(&self) -> &Self::T;
}

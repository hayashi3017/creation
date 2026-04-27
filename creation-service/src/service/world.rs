use async_trait::async_trait;
use thiserror::Error;

use super::{map_service_result, normalize_name, normalize_optional_text};
use crate::{
    model::world::{
        CreateWorldSchema, DeleteWorldSchema, GetWorldSchema, GetWorldsSchema, UpdateWorldSchema,
        World, WORLD_NAME_MAX_CHARS,
    },
    repository::world::{
        CreateWorldRepositoryError, DeleteWorldRepositoryError, GetWorldRepositoryError,
        GetWorldsRepositoryError, ProvidesWorldRepository, UpdateWorldRepositoryError,
        UsesWorldRepository,
    },
};

#[async_trait]
pub trait WorldService: ProvidesWorldRepository {}

#[derive(Debug, Error)]
pub enum GetWorldsServiceError {
    #[error(transparent)]
    GetWorldsRepositoryError(#[from] GetWorldsRepositoryError),
}

#[derive(Debug, Error)]
pub enum GetWorldServiceError {
    #[error(transparent)]
    GetWorldRepositoryError(#[from] GetWorldRepositoryError),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum CreateWorldServiceError {
    #[error(transparent)]
    CreateWorldRepositoryError(#[from] CreateWorldRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[derive(Debug, Error)]
pub enum UpdateWorldServiceError {
    #[error(transparent)]
    UpdateWorldRepositoryError(#[from] UpdateWorldRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteWorldServiceError {
    #[error(transparent)]
    DeleteWorldRepositoryError(#[from] DeleteWorldRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesWorldService {
    async fn get_worlds(&self, body: GetWorldsSchema) -> Result<Vec<World>, GetWorldsServiceError>;
    async fn get_world(&self, body: GetWorldSchema) -> Result<World, GetWorldServiceError>;
    async fn create_world(&self, body: CreateWorldSchema)
        -> Result<World, CreateWorldServiceError>;
    async fn update_world(&self, body: UpdateWorldSchema)
        -> Result<World, UpdateWorldServiceError>;
    async fn delete_world(&self, body: DeleteWorldSchema) -> Result<(), DeleteWorldServiceError>;
}

#[async_trait]
impl<T: WorldService> UsesWorldService for T {
    async fn get_worlds(&self, body: GetWorldsSchema) -> Result<Vec<World>, GetWorldsServiceError> {
        map_service_result!(
            self.world_repository().get_worlds(body),
            GetWorldsServiceError::GetWorldsRepositoryError
        )
    }

    async fn get_world(&self, body: GetWorldSchema) -> Result<World, GetWorldServiceError> {
        if body.world_id == 0 {
            return Err(GetWorldServiceError::NotFound);
        }

        match self.world_repository().get_world(body).await? {
            Some(world) => Ok(world),
            None => Err(GetWorldServiceError::NotFound),
        }
    }

    async fn create_world(
        &self,
        body: CreateWorldSchema,
    ) -> Result<World, CreateWorldServiceError> {
        let mut body = body;
        let Some(name) = normalize_name(&body.name, WORLD_NAME_MAX_CHARS) else {
            return Err(CreateWorldServiceError::InvalidParams);
        };

        body.name = name;
        body.description = normalize_optional_text(body.description);

        map_service_result!(
            self.world_repository().create_world(body),
            CreateWorldServiceError::CreateWorldRepositoryError
        )
    }

    async fn update_world(
        &self,
        body: UpdateWorldSchema,
    ) -> Result<World, UpdateWorldServiceError> {
        let mut body = body;
        if body.world_id == 0 {
            return Err(UpdateWorldServiceError::InvalidParams);
        }

        let Some(name) = normalize_name(&body.name, WORLD_NAME_MAX_CHARS) else {
            return Err(UpdateWorldServiceError::InvalidParams);
        };

        body.name = name;
        body.description = normalize_optional_text(body.description);

        match self.world_repository().update_world(body).await {
            Ok(world) => Ok(world),
            Err(UpdateWorldRepositoryError::NotFound) => Err(UpdateWorldServiceError::NotFound),
            Err(err) => Err(UpdateWorldServiceError::UpdateWorldRepositoryError(err)),
        }
    }

    async fn delete_world(&self, body: DeleteWorldSchema) -> Result<(), DeleteWorldServiceError> {
        if body.world_id == 0 {
            return Err(DeleteWorldServiceError::InvalidParams);
        }

        match self.world_repository().delete_world(body).await {
            Ok(()) => Ok(()),
            Err(DeleteWorldRepositoryError::NotFound) => Err(DeleteWorldServiceError::NotFound),
            Err(err) => Err(DeleteWorldServiceError::DeleteWorldRepositoryError(err)),
        }
    }
}

pub trait ProvidesWorldService: Send + Sync + 'static {
    type T: WorldService;
    fn world_service(&self) -> &Self::T;
}

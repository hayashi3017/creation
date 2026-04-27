use async_trait::async_trait;
use creation_service::{
    model::world::{
        CreateWorldSchema, DeleteWorldSchema, GetWorldSchema, GetWorldsSchema, UpdateWorldSchema,
        World,
    },
    service::world::{
        CreateWorldServiceError, DeleteWorldServiceError, GetWorldServiceError,
        GetWorldsServiceError, ProvidesWorldService, UpdateWorldServiceError, UsesWorldService,
    },
};
use thiserror::Error;

use super::{map_usecase_result, map_usecase_result_unit};

#[async_trait]
pub trait WorldUsecase: ProvidesWorldService {}

#[derive(Debug, Error)]
pub enum GetWorldsUsecaseError {
    #[error(transparent)]
    GetWorldsServiceError(#[from] GetWorldsServiceError),
}

#[derive(Debug, Error)]
pub enum GetWorldUsecaseError {
    #[error(transparent)]
    GetWorldServiceError(#[from] GetWorldServiceError),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum CreateWorldUsecaseError {
    #[error(transparent)]
    CreateWorldServiceError(#[from] CreateWorldServiceError),
}

#[derive(Debug, Error)]
pub enum UpdateWorldUsecaseError {
    #[error(transparent)]
    UpdateWorldServiceError(#[from] UpdateWorldServiceError),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteWorldUsecaseError {
    #[error(transparent)]
    DeleteWorldServiceError(#[from] DeleteWorldServiceError),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesWorldUsecase {
    async fn get_worlds(&self, body: GetWorldsSchema) -> Result<Vec<World>, GetWorldsUsecaseError>;
    async fn get_world(&self, body: GetWorldSchema) -> Result<World, GetWorldUsecaseError>;
    async fn create_world(&self, body: CreateWorldSchema)
        -> Result<World, CreateWorldUsecaseError>;
    async fn update_world(&self, body: UpdateWorldSchema)
        -> Result<World, UpdateWorldUsecaseError>;
    async fn delete_world(&self, body: DeleteWorldSchema) -> Result<(), DeleteWorldUsecaseError>;
}

#[async_trait]
impl<T: WorldUsecase> UsesWorldUsecase for T {
    async fn get_worlds(&self, body: GetWorldsSchema) -> Result<Vec<World>, GetWorldsUsecaseError> {
        map_usecase_result!(
            self.world_service().get_worlds(body),
            GetWorldsUsecaseError::GetWorldsServiceError
        )
    }

    async fn get_world(&self, body: GetWorldSchema) -> Result<World, GetWorldUsecaseError> {
        match self.world_service().get_world(body).await {
            Ok(world) => Ok(world),
            Err(GetWorldServiceError::NotFound) => Err(GetWorldUsecaseError::NotFound),
            Err(err) => Err(GetWorldUsecaseError::GetWorldServiceError(err)),
        }
    }

    async fn create_world(
        &self,
        body: CreateWorldSchema,
    ) -> Result<World, CreateWorldUsecaseError> {
        map_usecase_result!(
            self.world_service().create_world(body),
            CreateWorldUsecaseError::CreateWorldServiceError
        )
    }

    async fn update_world(
        &self,
        body: UpdateWorldSchema,
    ) -> Result<World, UpdateWorldUsecaseError> {
        match self.world_service().update_world(body).await {
            Ok(world) => Ok(world),
            Err(UpdateWorldServiceError::NotFound) => Err(UpdateWorldUsecaseError::NotFound),
            Err(err) => Err(UpdateWorldUsecaseError::UpdateWorldServiceError(err)),
        }
    }

    async fn delete_world(&self, body: DeleteWorldSchema) -> Result<(), DeleteWorldUsecaseError> {
        match map_usecase_result_unit!(
            self.world_service().delete_world(body),
            DeleteWorldUsecaseError::DeleteWorldServiceError
        ) {
            Ok(()) => Ok(()),
            Err(DeleteWorldUsecaseError::DeleteWorldServiceError(
                DeleteWorldServiceError::NotFound,
            )) => Err(DeleteWorldUsecaseError::NotFound),
            Err(err) => Err(err),
        }
    }
}

pub trait ProvidesWorldUsecase: Send + Sync + 'static {
    type T: UsesWorldUsecase + Sized;
    fn world_usecase(&self) -> &Self::T;
}

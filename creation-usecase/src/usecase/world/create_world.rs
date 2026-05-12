use async_trait::async_trait;
use creation_service::{
    model::world::{CreateWorldSchema, World},
    service::world::{CreateWorldServiceError, UsesWorldService},
};
use thiserror::Error;

use super::map_usecase_result;
use super::WorldUsecase;

#[derive(Debug, Error)]
pub enum CreateWorldUsecaseError {
    #[error(transparent)]
    CreateWorldServiceError(#[from] CreateWorldServiceError),
}

#[async_trait]
pub trait UsesCreateWorldUsecase {
    async fn create_world(&self, body: CreateWorldSchema)
        -> Result<World, CreateWorldUsecaseError>;
}

#[async_trait]
impl<T: WorldUsecase> UsesCreateWorldUsecase for T {
    async fn create_world(
        &self,
        body: CreateWorldSchema,
    ) -> Result<World, CreateWorldUsecaseError> {
        map_usecase_result!(
            self.world_service().create_world(body),
            CreateWorldUsecaseError::CreateWorldServiceError
        )
    }
}

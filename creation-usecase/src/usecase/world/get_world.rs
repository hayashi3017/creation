use async_trait::async_trait;
use creation_service::{
    model::world::{GetWorldSchema, World},
    service::world::{GetWorldServiceError, UsesWorldService},
};
use thiserror::Error;

use super::WorldUsecase;

#[derive(Debug, Error)]
pub enum GetWorldUsecaseError {
    #[error(transparent)]
    GetWorldServiceError(#[from] GetWorldServiceError),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesGetWorldUsecase {
    async fn get_world(&self, body: GetWorldSchema) -> Result<World, GetWorldUsecaseError>;
}

#[async_trait]
impl<T: WorldUsecase> UsesGetWorldUsecase for T {
    async fn get_world(&self, body: GetWorldSchema) -> Result<World, GetWorldUsecaseError> {
        match self.world_service().get_world(body).await {
            Ok(world) => Ok(world),
            Err(GetWorldServiceError::NotFound) => Err(GetWorldUsecaseError::NotFound),
            Err(err) => Err(GetWorldUsecaseError::GetWorldServiceError(err)),
        }
    }
}

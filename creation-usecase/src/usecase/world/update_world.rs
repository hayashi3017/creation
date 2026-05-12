use async_trait::async_trait;
use creation_service::{
    model::world::{UpdateWorldSchema, World},
    service::world::{UpdateWorldServiceError, ProvidesWorldService, UsesWorldService},
};
use thiserror::Error;

use super::WorldUsecase;

#[derive(Debug, Error)]
pub enum UpdateWorldUsecaseError {
    #[error(transparent)]
    UpdateWorldServiceError(#[from] UpdateWorldServiceError),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesUpdateWorldUsecase {
    async fn update_world(&self, body: UpdateWorldSchema)
        -> Result<World, UpdateWorldUsecaseError>;
}

#[async_trait]
impl<T: WorldUsecase> UsesUpdateWorldUsecase for T {
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
}

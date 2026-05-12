use async_trait::async_trait;
use creation_service::{
    model::world::{GetWorldsSchema, World},
    service::world::{GetWorldsServiceError, ProvidesWorldService, UsesWorldService},
};
use thiserror::Error;

use super::map_usecase_result;
use super::WorldUsecase;

#[derive(Debug, Error)]
pub enum GetWorldsUsecaseError {
    #[error(transparent)]
    GetWorldsServiceError(#[from] GetWorldsServiceError),
}

#[async_trait]
pub trait UsesGetWorldsUsecase {
    async fn get_worlds(&self, body: GetWorldsSchema) -> Result<Vec<World>, GetWorldsUsecaseError>;
}

#[async_trait]
impl<T: WorldUsecase> UsesGetWorldsUsecase for T {
    async fn get_worlds(&self, body: GetWorldsSchema) -> Result<Vec<World>, GetWorldsUsecaseError> {
        map_usecase_result!(
            self.world_service().get_worlds(body),
            GetWorldsUsecaseError::GetWorldsServiceError
        )
    }
}

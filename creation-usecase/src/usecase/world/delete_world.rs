use async_trait::async_trait;
use creation_service::{
    model::world::DeleteWorldSchema,
    service::world::{DeleteWorldServiceError, UsesWorldService},
};
use thiserror::Error;

use super::map_usecase_result_unit;
use super::WorldUsecase;

#[derive(Debug, Error)]
pub enum DeleteWorldUsecaseError {
    #[error(transparent)]
    DeleteWorldServiceError(#[from] DeleteWorldServiceError),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesDeleteWorldUsecase {
    async fn delete_world(&self, body: DeleteWorldSchema) -> Result<(), DeleteWorldUsecaseError>;
}

#[async_trait]
impl<T: WorldUsecase> UsesDeleteWorldUsecase for T {
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

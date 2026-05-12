use async_trait::async_trait;
use creation_service::{
    model::entity::DeleteEntitySchema,
    service::entity::{DeleteEntityServiceError, UsesEntityService},
};
use thiserror::Error;

use super::EntityUsecase;

#[derive(Debug, Error)]
pub enum DeleteEntityUsecaseError {
    #[error(transparent)]
    DeleteEntityServiceError(#[from] DeleteEntityServiceError),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesDeleteEntityUsecase {
    async fn delete_entity(&self, body: DeleteEntitySchema)
        -> Result<(), DeleteEntityUsecaseError>;
}

#[async_trait]
impl<T: EntityUsecase> UsesDeleteEntityUsecase for T {
    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<(), DeleteEntityUsecaseError> {
        match self.entity_service().delete_entity(body).await {
            Ok(_) => Ok(()),
            Err(DeleteEntityServiceError::NotFound) => Err(DeleteEntityUsecaseError::NotFound),
            Err(err) => Err(DeleteEntityUsecaseError::DeleteEntityServiceError(err)),
        }
    }
}

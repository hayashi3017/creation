use async_trait::async_trait;
use creation_service::{
    model::entity::UpdateEntitySchema,
    service::entity::{UpdateEntityServiceError, UsesEntityService},
};
use thiserror::Error;

use super::EntityUsecase;

#[derive(Debug, Error)]
pub enum UpdateEntityUsecaseError {
    #[error(transparent)]
    UpdateEntityServiceError(#[from] UpdateEntityServiceError),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesUpdateEntityUsecase {
    async fn update_entity(&self, body: UpdateEntitySchema)
        -> Result<(), UpdateEntityUsecaseError>;
}

#[async_trait]
impl<T: EntityUsecase> UsesUpdateEntityUsecase for T {
    async fn update_entity(
        &self,
        body: UpdateEntitySchema,
    ) -> Result<(), UpdateEntityUsecaseError> {
        match self.entity_service().update_entity(body).await {
            Ok(()) => Ok(()),
            Err(UpdateEntityServiceError::NotFound) => Err(UpdateEntityUsecaseError::NotFound),
            Err(err) => Err(UpdateEntityUsecaseError::UpdateEntityServiceError(err)),
        }
    }
}

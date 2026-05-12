use async_trait::async_trait;
use creation_service::{
    model::entity::CreateEntitySchema,
    service::entity::{CreateEntityServiceError, UsesEntityService},
};
use thiserror::Error;

use super::map_usecase_result_unit;
use super::EntityUsecase;

#[derive(Debug, Error)]
pub enum CreateEntityUsecaseError {
    #[error(transparent)]
    CreateEntityServiceError(#[from] CreateEntityServiceError),
}

#[async_trait]
pub trait UsesCreateEntityUsecase {
    async fn create_entity(&self, body: CreateEntitySchema)
        -> Result<(), CreateEntityUsecaseError>;
}

#[async_trait]
impl<T: EntityUsecase> UsesCreateEntityUsecase for T {
    async fn create_entity(
        &self,
        body: CreateEntitySchema,
    ) -> Result<(), CreateEntityUsecaseError> {
        map_usecase_result_unit!(
            self.entity_service().create_entity(body),
            CreateEntityUsecaseError::CreateEntityServiceError
        )
    }
}

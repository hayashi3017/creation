use async_trait::async_trait;
use creation_service::{
    model::entity::{Entity, GetEntitiesSchema},
    service::entity::{GetEntitiesServiceError, ProvidesEntityService, UsesEntityService},
};
use thiserror::Error;

use super::map_usecase_result;
use super::EntityUsecase;

#[derive(Debug, Error)]
pub enum GetEntitiesUsecaseError {
    #[error(transparent)]
    GetEntitiesServiceError(#[from] GetEntitiesServiceError),
}

#[async_trait]
pub trait UsesGetEntitiesUsecase {
    async fn get_entities(
        &self,
        body: GetEntitiesSchema,
    ) -> Result<Vec<Entity>, GetEntitiesUsecaseError>;
}

#[async_trait]
impl<T: EntityUsecase> UsesGetEntitiesUsecase for T {
    async fn get_entities(
        &self,
        body: GetEntitiesSchema,
    ) -> Result<Vec<Entity>, GetEntitiesUsecaseError> {
        map_usecase_result!(
            self.entity_service().get_entities(body),
            GetEntitiesUsecaseError::GetEntitiesServiceError
        )
    }
}

use async_trait::async_trait;
use creation_service::{
    model::entity::CreateDiagramEntityMembershipSchema,
    service::entity::{
        CreateDiagramEntityMembershipServiceError, ProvidesEntityService, UsesEntityService,
    },
};
use thiserror::Error;

use super::EntityUsecase;

#[derive(Debug, Error)]
pub enum CreateDiagramEntityMembershipUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    CreateDiagramEntityMembershipServiceError(#[from] CreateDiagramEntityMembershipServiceError),
}

#[async_trait]
pub trait UsesCreateDiagramEntityMembershipUsecase {
    async fn create_diagram_entity_membership(
        &self,
        body: CreateDiagramEntityMembershipSchema,
    ) -> Result<(), CreateDiagramEntityMembershipUsecaseError>;
}

#[async_trait]
impl<T: EntityUsecase> UsesCreateDiagramEntityMembershipUsecase for T {
    async fn create_diagram_entity_membership(
        &self,
        body: CreateDiagramEntityMembershipSchema,
    ) -> Result<(), CreateDiagramEntityMembershipUsecaseError> {
        match self
            .entity_service()
            .create_diagram_entity_membership(body)
            .await
        {
            Ok(()) => Ok(()),
            Err(CreateDiagramEntityMembershipServiceError::InvalidParams) => {
                Err(CreateDiagramEntityMembershipUsecaseError::InvalidParams)
            }
            Err(CreateDiagramEntityMembershipServiceError::NotFound) => {
                Err(CreateDiagramEntityMembershipUsecaseError::NotFound)
            }
            Err(err) => Err(
                CreateDiagramEntityMembershipUsecaseError::CreateDiagramEntityMembershipServiceError(
                    err,
                ),
            ),
        }
    }
}

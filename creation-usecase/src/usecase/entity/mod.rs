pub mod create_diagram_entity_membership;
pub mod create_entity;
pub mod delete_entity;
pub mod get_entities;
pub mod update_entity;

use async_trait::async_trait;
use creation_service::service::entity::ProvidesEntityService;
use thiserror::Error;

use super::map_usecase_result;
use super::map_usecase_result_unit;

pub use create_diagram_entity_membership::{
    CreateDiagramEntityMembershipUsecaseError, UsesCreateDiagramEntityMembershipUsecase,
};
pub use create_entity::{CreateEntityUsecaseError, UsesCreateEntityUsecase};
pub use delete_entity::{DeleteEntityUsecaseError, UsesDeleteEntityUsecase};
pub use get_entities::{GetEntitiesUsecaseError, UsesGetEntitiesUsecase};
pub use update_entity::{UpdateEntityUsecaseError, UsesUpdateEntityUsecase};

#[async_trait]
pub trait EntityUsecase: ProvidesEntityService {}

#[derive(Debug, Error)]
pub enum EntityUsecaseError {
    #[error(transparent)]
    GetEntitiesUsecaseError(#[from] GetEntitiesUsecaseError),
    #[error(transparent)]
    CreateEntityUsecaseError(#[from] CreateEntityUsecaseError),
    #[error(transparent)]
    CreateDiagramEntityMembershipUsecaseError(#[from] CreateDiagramEntityMembershipUsecaseError),
    #[error(transparent)]
    UpdateEntityUsecaseError(#[from] UpdateEntityUsecaseError),
    #[error(transparent)]
    DeleteEntityUsecaseError(#[from] DeleteEntityUsecaseError),
}

#[async_trait]
pub trait UsesEntityUsecase:
    UsesGetEntitiesUsecase
    + UsesCreateEntityUsecase
    + UsesCreateDiagramEntityMembershipUsecase
    + UsesUpdateEntityUsecase
    + UsesDeleteEntityUsecase
{
    async fn get_entities(
        &self,
        body: creation_service::model::entity::GetEntitiesSchema,
    ) -> Result<Vec<creation_service::model::entity::Entity>, GetEntitiesUsecaseError> {
        UsesGetEntitiesUsecase::get_entities(self, body).await
    }

    async fn create_entity(
        &self,
        body: creation_service::model::entity::CreateEntitySchema,
    ) -> Result<(), CreateEntityUsecaseError> {
        UsesCreateEntityUsecase::create_entity(self, body).await
    }

    async fn create_diagram_entity_membership(
        &self,
        body: creation_service::model::entity::CreateDiagramEntityMembershipSchema,
    ) -> Result<(), CreateDiagramEntityMembershipUsecaseError> {
        UsesCreateDiagramEntityMembershipUsecase::create_diagram_entity_membership(self, body).await
    }

    async fn update_entity(
        &self,
        body: creation_service::model::entity::UpdateEntitySchema,
    ) -> Result<(), UpdateEntityUsecaseError> {
        UsesUpdateEntityUsecase::update_entity(self, body).await
    }

    async fn delete_entity(
        &self,
        body: creation_service::model::entity::DeleteEntitySchema,
    ) -> Result<(), DeleteEntityUsecaseError> {
        UsesDeleteEntityUsecase::delete_entity(self, body).await
    }
}

impl<T> UsesEntityUsecase for T where
    T: UsesGetEntitiesUsecase
        + UsesCreateEntityUsecase
        + UsesCreateDiagramEntityMembershipUsecase
        + UsesUpdateEntityUsecase
        + UsesDeleteEntityUsecase
{
}

pub trait ProvidesEntityUsecase: Send + Sync + 'static {
    type T: UsesEntityUsecase + Sized;
    fn entity_usecase(&self) -> &Self::T;
}

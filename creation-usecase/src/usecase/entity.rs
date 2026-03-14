use async_trait::async_trait;
use creation_service::{
    model::entity::{
        CreateEntitySchema, DeleteEntitySchema, Entity, GetEntitiesSchema, UpdateEntitySchema,
    },
    service::entity::{
        CreateEntityServiceError, DeleteEntityServiceError, GetEntitiesServiceError,
        ProvidesEntityService, UpdateEntityServiceError, UsesEntityService,
    },
};
use thiserror::Error;

use super::{map_usecase_result, map_usecase_result_unit};

#[async_trait]
pub trait EntityUsecase: ProvidesEntityService {}

#[derive(Debug, Error)]
pub enum EntityUsecaseError {
    #[error(transparent)]
    GetEntitiesUsecaseError(#[from] GetEntitiesUsecaseError),
    #[error(transparent)]
    CreateEntityUsecaseError(#[from] CreateEntityUsecaseError),
    #[error(transparent)]
    UpdateEntityUsecaseError(#[from] UpdateEntityUsecaseError),
    #[error(transparent)]
    DeleteEntityUsecaseError(#[from] DeleteEntityUsecaseError),
}

#[derive(Debug, Error)]
pub enum GetEntitiesUsecaseError {
    #[error(transparent)]
    GetEntitiesServiceError(#[from] GetEntitiesServiceError),
}

#[derive(Debug, Error)]
pub enum CreateEntityUsecaseError {
    #[error(transparent)]
    CreateEntityServiceError(#[from] CreateEntityServiceError),
}

#[derive(Debug, Error)]
pub enum UpdateEntityUsecaseError {
    #[error(transparent)]
    UpdateEntityServiceError(#[from] UpdateEntityServiceError),
}

#[derive(Debug, Error)]
pub enum DeleteEntityUsecaseError {
    #[error(transparent)]
    DeleteEntityServiceError(#[from] DeleteEntityServiceError),
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
        map_usecase_result_unit!(
            self.entity_service().update_entity(body),
            UpdateEntityUsecaseError::UpdateEntityServiceError
        )
    }
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
        map_usecase_result_unit!(
            self.entity_service().delete_entity(body),
            DeleteEntityUsecaseError::DeleteEntityServiceError
        )
    }
}

#[async_trait]
pub trait UsesEntityUsecase:
    UsesGetEntitiesUsecase + UsesCreateEntityUsecase + UsesUpdateEntityUsecase + UsesDeleteEntityUsecase
{
    async fn get_entities(
        &self,
        body: GetEntitiesSchema,
    ) -> Result<Vec<Entity>, GetEntitiesUsecaseError> {
        UsesGetEntitiesUsecase::get_entities(self, body).await
    }

    async fn create_entity(
        &self,
        body: CreateEntitySchema,
    ) -> Result<(), CreateEntityUsecaseError> {
        UsesCreateEntityUsecase::create_entity(self, body).await
    }

    async fn update_entity(
        &self,
        body: UpdateEntitySchema,
    ) -> Result<(), UpdateEntityUsecaseError> {
        UsesUpdateEntityUsecase::update_entity(self, body).await
    }

    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<(), DeleteEntityUsecaseError> {
        UsesDeleteEntityUsecase::delete_entity(self, body).await
    }
}

impl<T> UsesEntityUsecase for T where
    T: UsesGetEntitiesUsecase
        + UsesCreateEntityUsecase
        + UsesUpdateEntityUsecase
        + UsesDeleteEntityUsecase
{
}

pub trait ProvidesEntityUsecase: Send + Sync + 'static {
    type T: UsesEntityUsecase + Sized;
    fn entity_usecase(&self) -> &Self::T;
}

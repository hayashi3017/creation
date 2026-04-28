use async_trait::async_trait;
use creation_service::{
    model::entity::{
        CreateDiagramEntityMembershipSchema, CreateEntitySchema, DeleteEntitySchema, Entity,
        GetEntitiesSchema, UpdateEntitySchema,
    },
    service::entity::{
        CreateDiagramEntityMembershipServiceError, CreateEntityServiceError,
        DeleteEntityServiceError, GetEntitiesServiceError, ProvidesEntityService,
        UpdateEntityServiceError, UsesEntityService,
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
    CreateDiagramEntityMembershipUsecaseError(#[from] CreateDiagramEntityMembershipUsecaseError),
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
pub enum CreateDiagramEntityMembershipUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    CreateDiagramEntityMembershipServiceError(#[from] CreateDiagramEntityMembershipServiceError),
}

#[derive(Debug, Error)]
pub enum UpdateEntityUsecaseError {
    #[error(transparent)]
    UpdateEntityServiceError(#[from] UpdateEntityServiceError),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteEntityUsecaseError {
    #[error(transparent)]
    DeleteEntityServiceError(#[from] DeleteEntityServiceError),
    #[error("not found")]
    NotFound,
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

    async fn create_diagram_entity_membership(
        &self,
        body: CreateDiagramEntityMembershipSchema,
    ) -> Result<(), CreateDiagramEntityMembershipUsecaseError> {
        UsesCreateDiagramEntityMembershipUsecase::create_diagram_entity_membership(self, body).await
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
        + UsesCreateDiagramEntityMembershipUsecase
        + UsesUpdateEntityUsecase
        + UsesDeleteEntityUsecase
{
}

pub trait ProvidesEntityUsecase: Send + Sync + 'static {
    type T: UsesEntityUsecase + Sized;
    fn entity_usecase(&self) -> &Self::T;
}

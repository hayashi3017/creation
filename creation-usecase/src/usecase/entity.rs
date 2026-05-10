use std::collections::HashSet;

use async_trait::async_trait;
use creation_service::{
    model::diagram::ExistsActiveDiagramSchema,
    model::entity::{
        CreateDiagramEntityMembershipSchema, CreateDiagramEntityMembershipsSchema,
        CreateEntitySchema, DeleteDiagramEntityMembershipsByEntityIdsSchema, DeleteEntitySchema,
        Entity, GetEntitiesSchema, LoadActiveEntityIdsSchema, SyncDiagramEntityMembershipsSchema,
        UpdateEntitySchema,
    },
    service::diagram::{
        ExistsActiveDiagramServiceError, ProvidesDiagramService, UsesDiagramService,
    },
    service::entity::{
        CreateDiagramEntityMembershipServiceError, CreateEntityServiceError,
        DeleteDiagramEntityMembershipServiceError, DeleteEntityServiceError,
        GetEntitiesServiceError, LoadActiveEntityIdsServiceError, ProvidesEntityService,
        UpdateEntityServiceError, UsesEntityService,
    },
    service::transaction::{
        BeginTransactionError, ProvidesTransactionManager, TransactionContext, TransactionError,
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
    SyncDiagramEntityMembershipsUsecaseError(#[from] SyncDiagramEntityMembershipsUsecaseError),
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
pub enum SyncDiagramEntityMembershipsUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    BeginTransactionError(#[from] BeginTransactionError),
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    #[error(transparent)]
    ExistsActiveDiagramServiceError(#[from] ExistsActiveDiagramServiceError),
    #[error(transparent)]
    LoadActiveEntityIdsServiceError(#[from] LoadActiveEntityIdsServiceError),
    #[error(transparent)]
    DeleteDiagramEntityMembershipServiceError(#[from] DeleteDiagramEntityMembershipServiceError),
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
pub trait UsesSyncDiagramEntityMembershipsUsecase {
    async fn sync_diagram_entity_memberships(
        &self,
        body: SyncDiagramEntityMembershipsSchema,
    ) -> Result<(), SyncDiagramEntityMembershipsUsecaseError>;
}

#[async_trait]
impl<T> UsesSyncDiagramEntityMembershipsUsecase for T
where
    T: EntityUsecase + ProvidesTransactionManager,
    <T as ProvidesTransactionManager>::T:
        TransactionContext + ProvidesEntityService + ProvidesDiagramService,
{
    async fn sync_diagram_entity_memberships(
        &self,
        body: SyncDiagramEntityMembershipsSchema,
    ) -> Result<(), SyncDiagramEntityMembershipsUsecaseError> {
        let SyncDiagramEntityMembershipsSchema {
            diagram_id,
            entity_ids,
        } = body;

        if diagram_id == 0 || entity_ids.iter().any(|entity_id| *entity_id == 0) {
            return Err(SyncDiagramEntityMembershipsUsecaseError::InvalidParams);
        }

        let requested_entity_id_set: HashSet<_> = entity_ids.iter().copied().collect();

        let tx = self.begin_transaction().await?;

        if !tx
            .diagram_service()
            .exists_active_diagram(ExistsActiveDiagramSchema { diagram_id })
            .await?
        {
            return Err(SyncDiagramEntityMembershipsUsecaseError::NotFound);
        }

        let active_entity_ids = tx
            .entity_service()
            .load_active_entity_ids(LoadActiveEntityIdsSchema { diagram_id })
            .await?;
        let active_entity_id_set: HashSet<_> = active_entity_ids.into_iter().collect();

        let mut delete_entity_ids: Vec<_> = active_entity_id_set
            .difference(&requested_entity_id_set)
            .copied()
            .collect();
        delete_entity_ids.sort_unstable();

        if !delete_entity_ids.is_empty() {
            tx.entity_service()
                .delete_diagram_entity_memberships_by_entity_ids(
                    DeleteDiagramEntityMembershipsByEntityIdsSchema {
                        diagram_id,
                        entity_ids: delete_entity_ids,
                    },
                )
                .await?;
        }

        let mut create_entity_ids: Vec<_> = requested_entity_id_set
            .difference(&active_entity_id_set)
            .copied()
            .collect();
        create_entity_ids.sort_unstable();

        if !create_entity_ids.is_empty() {
            tx.entity_service()
                .create_diagram_entity_memberships(CreateDiagramEntityMembershipsSchema {
                    diagram_id,
                    entity_ids: create_entity_ids,
                })
                .await?;
        }

        tx.commit().await?;

        Ok(())
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
    + UsesSyncDiagramEntityMembershipsUsecase
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

    async fn sync_diagram_entity_memberships(
        &self,
        body: SyncDiagramEntityMembershipsSchema,
    ) -> Result<(), SyncDiagramEntityMembershipsUsecaseError> {
        UsesSyncDiagramEntityMembershipsUsecase::sync_diagram_entity_memberships(self, body).await
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
        + UsesSyncDiagramEntityMembershipsUsecase
        + UsesUpdateEntityUsecase
        + UsesDeleteEntityUsecase
{
}

pub trait ProvidesEntityUsecase: Send + Sync + 'static {
    type T: UsesEntityUsecase + Sized;
    fn entity_usecase(&self) -> &Self::T;
}

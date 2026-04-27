use async_trait::async_trait;
use creation_service::{
    model::diagram::{
        CreateDiagramSchema, DeleteDiagramSchema, Diagram, GetDiagramsSchema, UpdateDiagramSchema,
    },
    model::{
        entity::DeleteDiagramEntityMembershipsSchema,
        relationship::DeleteRelationshipsForDiagramSchema,
        tree_path::DeleteTreePathsForDiagramSchema,
    },
    service::{
        diagram::{
            CreateDiagramServiceError, DeleteDiagramServiceError, GetDiagramsServiceError,
            ProvidesDiagramService, UpdateDiagramServiceError, UsesDiagramService,
        },
        entity::{
            DeleteDiagramEntityMembershipsServiceError, ProvidesEntityService, UsesEntityService,
        },
        relationship::{
            DeleteRelationshipsForDiagramServiceError, ProvidesRelationshipService,
            UsesRelationshipService,
        },
        transaction::{
            BeginTransactionError, ProvidesTransactionManager, TransactionContext, TransactionError,
        },
        tree_path::{
            DeleteTreePathsForDiagramServiceError, ProvidesTreePathService, UsesTreePathService,
        },
    },
};
use thiserror::Error;

use super::{map_usecase_result, map_usecase_result_unit};

#[async_trait]
pub trait DiagramUsecase: ProvidesDiagramService {}

#[derive(Debug, Error)]
pub enum DiagramUsecaseError {
    #[error(transparent)]
    GetDiagramsUsecaseError(#[from] GetDiagramsUsecaseError),
    #[error(transparent)]
    CreateDiagramUsecaseError(#[from] CreateDiagramUsecaseError),
    #[error(transparent)]
    UpdateDiagramUsecaseError(#[from] UpdateDiagramUsecaseError),
    #[error(transparent)]
    DeleteDiagramUsecaseError(#[from] DeleteDiagramUsecaseError),
}

#[derive(Debug, Error)]
pub enum GetDiagramsUsecaseError {
    #[error(transparent)]
    GetDiagramsServiceError(#[from] GetDiagramsServiceError),
}

#[derive(Debug, Error)]
pub enum CreateDiagramUsecaseError {
    #[error(transparent)]
    CreateDiagramServiceError(#[from] CreateDiagramServiceError),
}

#[derive(Debug, Error)]
pub enum UpdateDiagramUsecaseError {
    #[error(transparent)]
    UpdateDiagramServiceError(#[from] UpdateDiagramServiceError),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteDiagramUsecaseError {
    #[error(transparent)]
    DeleteDiagramServiceError(#[from] DeleteDiagramServiceError),
    #[error(transparent)]
    DeleteDiagramEntityMembershipsServiceError(#[from] DeleteDiagramEntityMembershipsServiceError),
    #[error(transparent)]
    DeleteRelationshipsForDiagramServiceError(#[from] DeleteRelationshipsForDiagramServiceError),
    #[error(transparent)]
    DeleteTreePathsForDiagramServiceError(#[from] DeleteTreePathsForDiagramServiceError),
    #[error(transparent)]
    BeginTransactionError(#[from] BeginTransactionError),
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesGetDiagramsUsecase {
    async fn get_diagrams(
        &self,
        body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsUsecaseError>;
}

#[async_trait]
impl<T: DiagramUsecase> UsesGetDiagramsUsecase for T {
    async fn get_diagrams(
        &self,
        body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsUsecaseError> {
        map_usecase_result!(
            self.diagram_service().get_diagrams(body),
            GetDiagramsUsecaseError::GetDiagramsServiceError
        )
    }
}

#[async_trait]
pub trait UsesCreateDiagramUsecase {
    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramUsecaseError>;
}

#[async_trait]
impl<T: DiagramUsecase> UsesCreateDiagramUsecase for T {
    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramUsecaseError> {
        map_usecase_result_unit!(
            self.diagram_service().create_diagram(body),
            CreateDiagramUsecaseError::CreateDiagramServiceError
        )
    }
}

#[async_trait]
pub trait UsesUpdateDiagramUsecase {
    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramUsecaseError>;
}

#[async_trait]
impl<T: DiagramUsecase> UsesUpdateDiagramUsecase for T {
    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramUsecaseError> {
        match self.diagram_service().update_diagram(body).await {
            Ok(()) => Ok(()),
            Err(UpdateDiagramServiceError::NotFound) => Err(UpdateDiagramUsecaseError::NotFound),
            Err(err) => Err(UpdateDiagramUsecaseError::UpdateDiagramServiceError(err)),
        }
    }
}

#[async_trait]
pub trait UsesDeleteDiagramUsecase {
    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramUsecaseError>;
}

#[async_trait]
impl<T> UsesDeleteDiagramUsecase for T
where
    T: DiagramUsecase + ProvidesTransactionManager,
    <T as ProvidesTransactionManager>::T: TransactionContext,
    <T as ProvidesTransactionManager>::T: ProvidesDiagramService
        + ProvidesEntityService
        + ProvidesRelationshipService
        + ProvidesTreePathService,
{
    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramUsecaseError> {
        let diagram_id = body.diagram_id;
        if diagram_id == 0 {
            return Err(DeleteDiagramUsecaseError::DeleteDiagramServiceError(
                DeleteDiagramServiceError::InvalidParams,
            ));
        }

        let tx = self.begin_transaction().await?;

        tx.tree_path_service()
            .delete_tree_paths_for_diagram(DeleteTreePathsForDiagramSchema { diagram_id })
            .await?;

        tx.relationship_service()
            .delete_relationships_for_diagram(DeleteRelationshipsForDiagramSchema { diagram_id })
            .await?;

        tx.entity_service()
            .delete_diagram_entity_memberships(DeleteDiagramEntityMembershipsSchema { diagram_id })
            .await?;

        match tx.diagram_service().delete_diagram(body).await {
            Ok(()) => Ok(()),
            Err(DeleteDiagramServiceError::NotFound) => {
                return Err(DeleteDiagramUsecaseError::NotFound)
            }
            Err(err) => Err(DeleteDiagramUsecaseError::DeleteDiagramServiceError(err)),
        }?;

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait]
pub trait UsesDiagramUsecase:
    UsesGetDiagramsUsecase
    + UsesCreateDiagramUsecase
    + UsesUpdateDiagramUsecase
    + UsesDeleteDiagramUsecase
{
    async fn get_diagrams(
        &self,
        body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsUsecaseError> {
        UsesGetDiagramsUsecase::get_diagrams(self, body).await
    }

    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramUsecaseError> {
        UsesCreateDiagramUsecase::create_diagram(self, body).await
    }

    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramUsecaseError> {
        UsesUpdateDiagramUsecase::update_diagram(self, body).await
    }

    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramUsecaseError> {
        UsesDeleteDiagramUsecase::delete_diagram(self, body).await
    }
}

impl<T> UsesDiagramUsecase for T where
    T: UsesGetDiagramsUsecase
        + UsesCreateDiagramUsecase
        + UsesUpdateDiagramUsecase
        + UsesDeleteDiagramUsecase
{
}

pub trait ProvidesDiagramUsecase: Send + Sync + 'static {
    type T: UsesDiagramUsecase + Sized;
    fn diagram_usecase(&self) -> &Self::T;
}

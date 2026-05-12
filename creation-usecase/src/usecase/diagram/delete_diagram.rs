use async_trait::async_trait;
use creation_service::{
    model::{
        diagram::DeleteDiagramSchema, entity::DeleteDiagramEntityMembershipsSchema,
        relationship::DeleteRelationshipsForDiagramSchema,
        tree_path::DeleteTreePathsForDiagramSchema,
    },
    service::{
        diagram::{DeleteDiagramServiceError, ProvidesDiagramService, UsesDiagramService},
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

use super::DiagramUsecase;

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
        if diagram_id == 0 || body.world_id == 0 {
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

use async_trait::async_trait;
use creation_service::{
    model::{
        diagram::ExistsActiveDiagramSchema,
        relationship::{DeleteRelationshipSchema, LoadRelationshipDiagramIdSchema},
        tree_path::SyncTreePathsByEntityIdsSchema,
    },
    repository::{
        diagram::{
            ExistsActiveDiagramRepositoryError, ProvidesDiagramRepository, UsesDiagramRepository,
        },
        relationship::{
            LoadRelationshipDiagramIdRepositoryError, ProvidesRelationshipRepository,
            UsesRelationshipRepository,
        },
    },
    service::{
        relationship::{
            DeleteRelationshipServiceError, ProvidesRelationshipService, UsesRelationshipService,
        },
        transaction::{
            BeginTransactionError, ProvidesTransactionManager, TransactionContext, TransactionError,
        },
        tree_path::{ProvidesTreePathService, SyncTreePathsServiceError, UsesTreePathService},
    },
};
use thiserror::Error;

use super::{map_tree_path_transaction_error, normalize_entity_ids, RelationshipUsecase};

#[derive(Debug, Error)]
pub enum DeleteRelationshipUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    BeginTransactionError(#[from] BeginTransactionError),
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesDeleteRelationshipUsecase {
    async fn delete_relationship(
        &self,
        body: DeleteRelationshipSchema,
    ) -> Result<(), DeleteRelationshipUsecaseError>;
}

#[async_trait]
impl<T> UsesDeleteRelationshipUsecase for T
where
    T: RelationshipUsecase,
    <T as ProvidesTransactionManager>::T: TransactionContext,
    <T as ProvidesTransactionManager>::T: ProvidesDiagramRepository
        + ProvidesRelationshipRepository
        + ProvidesRelationshipService
        + ProvidesTreePathService,
{
    async fn delete_relationship(
        &self,
        body: DeleteRelationshipSchema,
    ) -> Result<(), DeleteRelationshipUsecaseError> {
        if body.relationship_id == 0 {
            return Err(DeleteRelationshipUsecaseError::InvalidParams);
        }

        let tx = self.begin_transaction().await?;

        let Some(diagram_id) = tx
            .relationship_repository()
            .load_relationship_diagram_id(LoadRelationshipDiagramIdSchema {
                relationship_id: body.relationship_id,
            })
            .await
            .map_err(map_delete_relationship_load_diagram_id_error)?
        else {
            return Err(DeleteRelationshipUsecaseError::NotFound);
        };

        if !tx
            .diagram_repository()
            .exists_active_diagram(ExistsActiveDiagramSchema { diagram_id })
            .await
            .map_err(map_delete_relationship_diagram_error)?
        {
            return Err(DeleteRelationshipUsecaseError::NotFound);
        }

        let endpoints = tx
            .relationship_service()
            .delete_relationship(body)
            .await
            .map_err(map_delete_relationship_service_error)?;

        tx.tree_path_service()
            .sync_tree_paths_by_entity_ids(SyncTreePathsByEntityIdsSchema {
                entity_ids: normalize_entity_ids(vec![
                    endpoints.source_entity_id,
                    endpoints.target_entity_id,
                ]),
            })
            .await
            .map_err(map_delete_relationship_tree_path_error)?;

        tx.commit()
            .await
            .map_err(map_delete_relationship_transaction_error)?;

        Ok(())
    }
}

fn map_delete_relationship_service_error(
    err: DeleteRelationshipServiceError,
) -> DeleteRelationshipUsecaseError {
    match err {
        DeleteRelationshipServiceError::InvalidParams => {
            DeleteRelationshipUsecaseError::InvalidParams
        }
        DeleteRelationshipServiceError::NotFound => DeleteRelationshipUsecaseError::NotFound,
        DeleteRelationshipServiceError::DeleteRelationshipRepositoryError(err) => {
            DeleteRelationshipUsecaseError::TransactionError(match err {
                creation_service::repository::relationship::DeleteRelationshipRepositoryError::Db(err) => TransactionError::Db(err),
                creation_service::repository::relationship::DeleteRelationshipRepositoryError::NotFound => TransactionError::NotFound,
            })
        }
    }
}

fn map_delete_relationship_load_diagram_id_error(
    err: LoadRelationshipDiagramIdRepositoryError,
) -> DeleteRelationshipUsecaseError {
    DeleteRelationshipUsecaseError::TransactionError(match err {
        LoadRelationshipDiagramIdRepositoryError::Db(err) => TransactionError::Db(err),
    })
}

fn map_delete_relationship_diagram_error(
    err: ExistsActiveDiagramRepositoryError,
) -> DeleteRelationshipUsecaseError {
    DeleteRelationshipUsecaseError::TransactionError(match err {
        ExistsActiveDiagramRepositoryError::Db(err) => TransactionError::Db(err),
    })
}

fn map_delete_relationship_tree_path_error(
    err: SyncTreePathsServiceError,
) -> DeleteRelationshipUsecaseError {
    match err {
        SyncTreePathsServiceError::CycleDetected => DeleteRelationshipUsecaseError::InvalidParams,
        err => {
            DeleteRelationshipUsecaseError::TransactionError(map_tree_path_transaction_error(err))
        }
    }
}

fn map_delete_relationship_transaction_error(
    err: TransactionError,
) -> DeleteRelationshipUsecaseError {
    match err {
        TransactionError::NotFound => DeleteRelationshipUsecaseError::NotFound,
        err => DeleteRelationshipUsecaseError::TransactionError(err),
    }
}

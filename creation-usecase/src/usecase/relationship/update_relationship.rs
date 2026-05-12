use async_trait::async_trait;
use creation_service::{
    model::{
        diagram::ExistsActiveDiagramSchema,
        relationship::{LoadRelationshipDiagramIdSchema, UpdateRelationshipSchema},
        tree_path::SyncTreePathsByEntityIdsSchema,
    },
    repository::diagram::{
        ExistsActiveDiagramRepositoryError, ProvidesDiagramRepository, UsesDiagramRepository,
    },
    repository::relationship::{
        LoadRelationshipDiagramIdRepositoryError, ProvidesRelationshipRepository,
        UpdateRelationshipRepositoryError, UsesRelationshipRepository,
    },
    service::{
        relationship::{
            ProvidesRelationshipService, UpdateRelationshipServiceError, UsesRelationshipService,
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
pub enum UpdateRelationshipUsecaseError {
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
pub trait UsesUpdateRelationshipUsecase {
    async fn update_relationship(
        &self,
        body: UpdateRelationshipSchema,
    ) -> Result<(), UpdateRelationshipUsecaseError>;
}

#[async_trait]
impl<T> UsesUpdateRelationshipUsecase for T
where
    T: RelationshipUsecase,
    <T as ProvidesTransactionManager>::T: TransactionContext,
    <T as ProvidesTransactionManager>::T: ProvidesDiagramRepository
        + ProvidesRelationshipRepository
        + ProvidesRelationshipService
        + ProvidesTreePathService,
{
    async fn update_relationship(
        &self,
        body: UpdateRelationshipSchema,
    ) -> Result<(), UpdateRelationshipUsecaseError> {
        if body.relationship_id == 0 {
            return Err(UpdateRelationshipUsecaseError::InvalidParams);
        }

        let next_entity_ids = vec![body.source_entity_id, body.target_entity_id];
        let tx = self.begin_transaction().await?;

        let Some(diagram_id) = tx
            .relationship_repository()
            .load_relationship_diagram_id(LoadRelationshipDiagramIdSchema {
                relationship_id: body.relationship_id,
            })
            .await
            .map_err(map_update_relationship_load_diagram_id_error)?
        else {
            return Err(UpdateRelationshipUsecaseError::NotFound);
        };

        if !tx
            .diagram_repository()
            .exists_active_diagram(ExistsActiveDiagramSchema { diagram_id })
            .await
            .map_err(map_update_relationship_diagram_error)?
        {
            return Err(UpdateRelationshipUsecaseError::NotFound);
        }

        let previous_endpoints = tx
            .relationship_service()
            .update_relationship(body)
            .await
            .map_err(map_update_relationship_service_error)?;

        let mut affected_entity_ids = next_entity_ids;
        affected_entity_ids.push(previous_endpoints.previous_source_entity_id);
        affected_entity_ids.push(previous_endpoints.previous_target_entity_id);

        tx.tree_path_service()
            .sync_tree_paths_by_entity_ids(SyncTreePathsByEntityIdsSchema {
                entity_ids: normalize_entity_ids(affected_entity_ids),
            })
            .await
            .map_err(map_update_relationship_tree_path_error)?;

        tx.commit()
            .await
            .map_err(map_update_relationship_transaction_error)?;

        Ok(())
    }
}

fn map_update_relationship_service_error(
    err: UpdateRelationshipServiceError,
) -> UpdateRelationshipUsecaseError {
    match err {
        UpdateRelationshipServiceError::InvalidParams => {
            UpdateRelationshipUsecaseError::InvalidParams
        }
        UpdateRelationshipServiceError::NotFound => UpdateRelationshipUsecaseError::NotFound,
        UpdateRelationshipServiceError::UpdateRelationshipRepositoryError(err) => {
            UpdateRelationshipUsecaseError::TransactionError(match err {
                UpdateRelationshipRepositoryError::Db(err) => TransactionError::Db(err),
                UpdateRelationshipRepositoryError::NotFound => TransactionError::NotFound,
            })
        }
    }
}

fn map_update_relationship_load_diagram_id_error(
    err: LoadRelationshipDiagramIdRepositoryError,
) -> UpdateRelationshipUsecaseError {
    UpdateRelationshipUsecaseError::TransactionError(match err {
        LoadRelationshipDiagramIdRepositoryError::Db(err) => TransactionError::Db(err),
    })
}

fn map_update_relationship_diagram_error(
    err: ExistsActiveDiagramRepositoryError,
) -> UpdateRelationshipUsecaseError {
    UpdateRelationshipUsecaseError::TransactionError(match err {
        ExistsActiveDiagramRepositoryError::Db(err) => TransactionError::Db(err),
    })
}

fn map_update_relationship_tree_path_error(
    err: SyncTreePathsServiceError,
) -> UpdateRelationshipUsecaseError {
    match err {
        SyncTreePathsServiceError::CycleDetected => UpdateRelationshipUsecaseError::InvalidParams,
        err => {
            UpdateRelationshipUsecaseError::TransactionError(map_tree_path_transaction_error(err))
        }
    }
}

fn map_update_relationship_transaction_error(
    err: TransactionError,
) -> UpdateRelationshipUsecaseError {
    match err {
        TransactionError::NotFound => UpdateRelationshipUsecaseError::NotFound,
        err => UpdateRelationshipUsecaseError::TransactionError(err),
    }
}

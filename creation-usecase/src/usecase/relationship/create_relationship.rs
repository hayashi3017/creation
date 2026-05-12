use async_trait::async_trait;
use creation_service::{
    model::{
        diagram::ExistsActiveDiagramSchema,
        relationship::CreateRelationshipSchema,
        tree_path::SyncTreePathsByEntityIdsSchema,
    },
    repository::diagram::{ExistsActiveDiagramRepositoryError, ProvidesDiagramRepository},
    repository::relationship::CreateRelationshipRepositoryError,
    service::{
        relationship::{CreateRelationshipServiceError, ProvidesRelationshipService},
        transaction::{BeginTransactionError, ProvidesTransactionManager, TransactionContext, TransactionError},
        tree_path::{ProvidesTreePathService, SyncTreePathsServiceError},
    },
};
use thiserror::Error;

use super::{RelationshipUsecase, map_tree_path_transaction_error, normalize_entity_ids};

#[derive(Debug, Error)]
pub enum CreateRelationshipUsecaseError {
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
pub trait UsesCreateRelationshipUsecase {
    async fn create_relationship(
        &self,
        body: CreateRelationshipSchema,
    ) -> Result<(), CreateRelationshipUsecaseError>;
}

#[async_trait]
impl<T> UsesCreateRelationshipUsecase for T
where
    T: RelationshipUsecase,
{
    async fn create_relationship(
        &self,
        body: CreateRelationshipSchema,
    ) -> Result<(), CreateRelationshipUsecaseError> {
        if body.diagram_id == 0 {
            return Err(CreateRelationshipUsecaseError::InvalidParams);
        }

        let affected_entity_ids =
            normalize_entity_ids(vec![body.source_entity_id, body.target_entity_id]);
        let tx = self.begin_transaction().await?;

        if !tx
            .diagram_repository()
            .exists_active_diagram(ExistsActiveDiagramSchema {
                diagram_id: body.diagram_id,
            })
            .await
            .map_err(map_create_relationship_diagram_error)?
        {
            return Err(CreateRelationshipUsecaseError::NotFound);
        }

        tx.relationship_service()
            .create_relationship(body)
            .await
            .map_err(map_create_relationship_service_error)?;

        tx.tree_path_service()
            .sync_tree_paths_by_entity_ids(SyncTreePathsByEntityIdsSchema {
                entity_ids: affected_entity_ids,
            })
            .await
            .map_err(map_create_relationship_tree_path_error)?;

        tx.commit()
            .await
            .map_err(map_create_relationship_transaction_error)?;

        Ok(())
    }
}

fn map_create_relationship_service_error(
    err: CreateRelationshipServiceError,
) -> CreateRelationshipUsecaseError {
    match err {
        CreateRelationshipServiceError::InvalidParams => {
            CreateRelationshipUsecaseError::InvalidParams
        }
        CreateRelationshipServiceError::NotFound => CreateRelationshipUsecaseError::NotFound,
        CreateRelationshipServiceError::CreateRelationshipRepositoryError(err) => {
            CreateRelationshipUsecaseError::TransactionError(match err {
                CreateRelationshipRepositoryError::Db(err) => TransactionError::Db(err),
                CreateRelationshipRepositoryError::NotFound => TransactionError::NotFound,
            })
        }
    }
}

fn map_create_relationship_diagram_error(
    err: ExistsActiveDiagramRepositoryError,
) -> CreateRelationshipUsecaseError {
    CreateRelationshipUsecaseError::TransactionError(match err {
        ExistsActiveDiagramRepositoryError::Db(err) => TransactionError::Db(err),
    })
}

fn map_create_relationship_tree_path_error(
    err: SyncTreePathsServiceError,
) -> CreateRelationshipUsecaseError {
    match err {
        SyncTreePathsServiceError::CycleDetected => CreateRelationshipUsecaseError::InvalidParams,
        err => {
            CreateRelationshipUsecaseError::TransactionError(map_tree_path_transaction_error(err))
        }
    }
}

fn map_create_relationship_transaction_error(
    err: TransactionError,
) -> CreateRelationshipUsecaseError {
    match err {
        TransactionError::NotFound => CreateRelationshipUsecaseError::NotFound,
        err => CreateRelationshipUsecaseError::TransactionError(err),
    }
}

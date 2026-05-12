use async_trait::async_trait;
use creation_service::repository::person::DeletePersonRepositoryError;
use creation_service::{
    model::entity::DeleteEntitySchema,
    model::person::DeletePersonSchema,
    model::relationship::DeleteRelationshipsForEntitySchema,
    model::tree_path::SyncTreePathsByEntityIdsSchema,
    repository::entity::{
        DeleteEntityRepositoryError, LoadActiveEntitiesByDiagramIdsRepositoryError,
        LoadSeedEntitiesRepositoryError,
    },
    repository::relationship::DeleteRelationshipsForEntityRepositoryError,
    repository::relationship::{ProvidesRelationshipRepository, UsesRelationshipRepository},
    repository::tree_path::{
        CreateTreePathsRepositoryError, DeleteTreePathsByEntityIdsRepositoryError,
        LoadStaleRelatedConnectionsRepositoryError,
    },
    service::entity::{DeleteEntityServiceError, ProvidesEntityService, UsesEntityService},
    service::person::{
        prepare_delete_person, DeletePersonRecordServiceError, ProvidesPersonService,
        UsesPersonService,
    },
    service::transaction::{ProvidesTransactionManager, TransactionContext, TransactionError},
    service::tree_path::{ProvidesTreePathService, SyncTreePathsServiceError, UsesTreePathService},
};
use thiserror::Error;

use super::PersonUsecase;

#[derive(Debug, Error)]
pub enum DeletePersonUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    BeginTransactionError(#[from] creation_service::service::transaction::BeginTransactionError),
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesDeletePersonUsecase {
    async fn delete_person(&self, body: DeletePersonSchema)
        -> Result<(), DeletePersonUsecaseError>;
}

#[async_trait]
impl<T> UsesDeletePersonUsecase for T
where
    T: PersonUsecase,
    <T as ProvidesTransactionManager>::T: TransactionContext,
    <T as ProvidesTransactionManager>::T: ProvidesEntityService
        + ProvidesPersonService
        + ProvidesRelationshipRepository
        + ProvidesTreePathService,
{
    async fn delete_person(
        &self,
        body: DeletePersonSchema,
    ) -> Result<(), DeletePersonUsecaseError> {
        let body = prepare_delete_person(body).ok_or(DeletePersonUsecaseError::InvalidParams)?;
        let entity_id = body.entity_id;

        let tx = self.begin_transaction().await?;

        tx.entity_service()
            .delete_entity(DeleteEntitySchema {
                entity_id,
                world_id: body.world_id,
            })
            .await
            .map_err(map_delete_person_entity_error)?;

        tx.person_service()
            .delete_person_record(body)
            .await
            .map_err(map_delete_person_record_error)?;

        let mut affected_entity_ids = tx
            .relationship_repository()
            .delete_relationships_for_entity(DeleteRelationshipsForEntitySchema { entity_id })
            .await
            .map_err(map_delete_person_relationship_error)?;
        affected_entity_ids.push(entity_id);
        affected_entity_ids.sort_unstable();
        affected_entity_ids.dedup();

        tx.tree_path_service()
            .sync_tree_paths_by_entity_ids(SyncTreePathsByEntityIdsSchema {
                entity_ids: affected_entity_ids,
            })
            .await
            .map_err(map_delete_person_tree_path_error)?;

        tx.commit()
            .await
            .map_err(map_delete_person_transaction_error)?;

        Ok(())
    }
}

fn map_delete_person_entity_error(err: DeleteEntityServiceError) -> DeletePersonUsecaseError {
    match err {
        DeleteEntityServiceError::NotFound => DeletePersonUsecaseError::NotFound,
        DeleteEntityServiceError::DeleteEntityRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(match err {
                DeleteEntityRepositoryError::Db(err) => TransactionError::Db(err),
                DeleteEntityRepositoryError::NotFound => TransactionError::NotFound,
            })
        }
        DeleteEntityServiceError::InvalidParams => DeletePersonUsecaseError::InvalidParams,
    }
}

fn map_delete_person_relationship_error(
    err: DeleteRelationshipsForEntityRepositoryError,
) -> DeletePersonUsecaseError {
    match err {
        DeleteRelationshipsForEntityRepositoryError::Db(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(err))
        }
    }
}

fn map_delete_person_tree_path_error(err: SyncTreePathsServiceError) -> DeletePersonUsecaseError {
    match err {
        SyncTreePathsServiceError::CycleDetected => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(sqlx::Error::Protocol(
                "cycle detected while rebuilding tree_path after person delete".to_string(),
            )))
        }
        SyncTreePathsServiceError::LoadSeedEntitiesRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                LoadSeedEntitiesRepositoryError::Db(err) => err,
            }))
        }
        SyncTreePathsServiceError::LoadActiveEntitiesByDiagramIdsRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                LoadActiveEntitiesByDiagramIdsRepositoryError::Db(err) => err,
            }))
        }
        SyncTreePathsServiceError::LoadRelationshipEdgesByDiagramIdsRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                creation_service::repository::relationship::LoadRelationshipEdgesByDiagramIdsRepositoryError::Db(err) => err,
            }))
        }
        SyncTreePathsServiceError::LoadStaleRelatedConnectionsRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                LoadStaleRelatedConnectionsRepositoryError::Db(err) => err,
            }))
        }
        SyncTreePathsServiceError::DeleteTreePathsByEntityIdsRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                DeleteTreePathsByEntityIdsRepositoryError::Db(err) => err,
            }))
        }
        SyncTreePathsServiceError::CreateTreePathsRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                CreateTreePathsRepositoryError::Db(err) => err,
            }))
        }
    }
}

fn map_delete_person_transaction_error(err: TransactionError) -> DeletePersonUsecaseError {
    match err {
        TransactionError::NotFound => DeletePersonUsecaseError::NotFound,
        err => DeletePersonUsecaseError::TransactionError(err),
    }
}

fn map_delete_person_record_error(err: DeletePersonRecordServiceError) -> DeletePersonUsecaseError {
    match err {
        DeletePersonRecordServiceError::NotFound => DeletePersonUsecaseError::NotFound,
        DeletePersonRecordServiceError::DeletePersonRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(match err {
                DeletePersonRepositoryError::Db(err) => TransactionError::Db(err),
                DeletePersonRepositoryError::NotFound => TransactionError::NotFound,
            })
        }
        DeletePersonRecordServiceError::InvalidParams => DeletePersonUsecaseError::InvalidParams,
    }
}

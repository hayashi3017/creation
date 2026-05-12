pub mod create_relationship;
pub mod delete_relationship;
pub mod get_relationships;
pub mod update_relationship;

use async_trait::async_trait;
use creation_service::{
    repository::diagram::ProvidesDiagramRepository,
    service::{
        relationship::ProvidesRelationshipService, transaction::ProvidesTransactionManager,
        tree_path::ProvidesTreePathService,
    },
};
use thiserror::Error;

pub use create_relationship::{CreateRelationshipUsecaseError, UsesCreateRelationshipUsecase};
pub use delete_relationship::{DeleteRelationshipUsecaseError, UsesDeleteRelationshipUsecase};
pub use get_relationships::{GetRelationshipsUsecaseError, UsesGetRelationshipsUsecase};
pub use update_relationship::{UpdateRelationshipUsecaseError, UsesUpdateRelationshipUsecase};

#[async_trait]
pub trait RelationshipUsecase:
    ProvidesDiagramRepository
    + ProvidesRelationshipService
    + ProvidesTreePathService
    + ProvidesTransactionManager
{
}

#[derive(Debug, Error)]
pub enum RelationshipUsecaseError {
    #[error(transparent)]
    GetRelationshipsUsecaseError(#[from] GetRelationshipsUsecaseError),
    #[error(transparent)]
    CreateRelationshipUsecaseError(#[from] CreateRelationshipUsecaseError),
    #[error(transparent)]
    UpdateRelationshipUsecaseError(#[from] UpdateRelationshipUsecaseError),
    #[error(transparent)]
    DeleteRelationshipUsecaseError(#[from] DeleteRelationshipUsecaseError),
}

#[async_trait]
pub trait UsesRelationshipUsecase:
    UsesGetRelationshipsUsecase
    + UsesCreateRelationshipUsecase
    + UsesUpdateRelationshipUsecase
    + UsesDeleteRelationshipUsecase
{
    async fn get_relationships(
        &self,
        body: creation_service::model::relationship::GetRelationshipsSchema,
    ) -> Result<
        Vec<creation_service::model::relationship::Relationship>,
        GetRelationshipsUsecaseError,
    > {
        UsesGetRelationshipsUsecase::get_relationships(self, body).await
    }

    async fn create_relationship(
        &self,
        body: creation_service::model::relationship::CreateRelationshipSchema,
    ) -> Result<(), CreateRelationshipUsecaseError> {
        UsesCreateRelationshipUsecase::create_relationship(self, body).await
    }

    async fn update_relationship(
        &self,
        body: creation_service::model::relationship::UpdateRelationshipSchema,
    ) -> Result<(), UpdateRelationshipUsecaseError> {
        UsesUpdateRelationshipUsecase::update_relationship(self, body).await
    }

    async fn delete_relationship(
        &self,
        body: creation_service::model::relationship::DeleteRelationshipSchema,
    ) -> Result<(), DeleteRelationshipUsecaseError> {
        UsesDeleteRelationshipUsecase::delete_relationship(self, body).await
    }
}

impl<T> UsesRelationshipUsecase for T where
    T: UsesGetRelationshipsUsecase
        + UsesCreateRelationshipUsecase
        + UsesUpdateRelationshipUsecase
        + UsesDeleteRelationshipUsecase
{
}

pub trait ProvidesRelationshipUsecase: Send + Sync + 'static {
    type T: UsesRelationshipUsecase + Sized;
    fn relationship_usecase(&self) -> &Self::T;
}

pub(crate) fn normalize_entity_ids(mut entity_ids: Vec<usize>) -> Vec<usize> {
    entity_ids.sort_unstable();
    entity_ids.dedup();
    entity_ids
}

pub(crate) fn map_delete_relationship_tree_path_error(
    err: creation_service::service::tree_path::SyncTreePathsServiceError,
) -> creation_service::service::transaction::TransactionError {
    match err {
        creation_service::service::tree_path::SyncTreePathsServiceError::CycleDetected => {
            creation_service::service::transaction::TransactionError::Db(sqlx::Error::Protocol(
                "cycle detected while rebuilding tree_path after relationship change".to_string(),
            ))
        }
        _ => map_tree_path_transaction_error(err),
    }
}

pub(crate) fn map_tree_path_transaction_error(
    err: creation_service::service::tree_path::SyncTreePathsServiceError,
) -> creation_service::service::transaction::TransactionError {
    use creation_service::service::transaction::TransactionError;
    use creation_service::service::tree_path::SyncTreePathsServiceError;

    match err {
        SyncTreePathsServiceError::LoadSeedEntitiesRepositoryError(err) => TransactionError::Db(
            match err {
                creation_service::repository::entity::LoadSeedEntitiesRepositoryError::Db(err) => {
                    err
                }
            },
        ),
        SyncTreePathsServiceError::LoadActiveEntitiesByDiagramIdsRepositoryError(err) => {
            TransactionError::Db(match err {
                creation_service::repository::entity::LoadActiveEntitiesByDiagramIdsRepositoryError::Db(
                    err,
                ) => err,
            })
        }
        SyncTreePathsServiceError::LoadRelationshipEdgesByDiagramIdsRepositoryError(err) => {
            TransactionError::Db(match err {
                creation_service::repository::relationship::LoadRelationshipEdgesByDiagramIdsRepositoryError::Db(err) => err,
            })
        }
        SyncTreePathsServiceError::LoadStaleRelatedConnectionsRepositoryError(err) => {
            TransactionError::Db(match err {
                creation_service::repository::tree_path::LoadStaleRelatedConnectionsRepositoryError::Db(err) => err,
            })
        }
        SyncTreePathsServiceError::DeleteTreePathsByEntityIdsRepositoryError(err) => {
            TransactionError::Db(match err {
                creation_service::repository::tree_path::DeleteTreePathsByEntityIdsRepositoryError::Db(err) => err,
            })
        }
        SyncTreePathsServiceError::CreateTreePathsRepositoryError(err) => TransactionError::Db(
            match err {
                creation_service::repository::tree_path::CreateTreePathsRepositoryError::Db(err) => err,
            },
        ),
        SyncTreePathsServiceError::CycleDetected => TransactionError::Db(sqlx::Error::Protocol(
            "cycle detected while rebuilding tree_path after relationship change".to_string(),
        )),
    }
}

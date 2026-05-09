use async_trait::async_trait;
use creation_service::{
    model::relationship::{
        CreateRelationshipSchema, DeleteRelationshipSchema, GetRelationshipsSchema,
        LoadRelationshipWorldIdSchema, Relationship, UpdateRelationshipSchema,
    },
    model::tree_path::SyncTreePathsByEntityIdsSchema,
    model::world::GetWorldSchema,
    repository::relationship::{
        CreateRelationshipRepositoryError, DeleteRelationshipRepositoryError,
        LoadRelationshipWorldIdRepositoryError, ProvidesRelationshipRepository,
        UpdateRelationshipRepositoryError, UsesRelationshipRepository,
    },
    service::{
        relationship::{
            CreateRelationshipServiceError, DeleteRelationshipServiceError,
            GetRelationshipsServiceError, ProvidesRelationshipService,
            UpdateRelationshipServiceError, UsesRelationshipService,
        },
        transaction::{
            BeginTransactionError, ProvidesTransactionManager, TransactionContext, TransactionError,
        },
        tree_path::{ProvidesTreePathService, SyncTreePathsServiceError, UsesTreePathService},
        world::{GetWorldServiceError, ProvidesWorldService, UsesWorldService},
    },
};
use thiserror::Error;

#[async_trait]
pub trait RelationshipUsecase:
    ProvidesWorldService
    + ProvidesRelationshipService
    + ProvidesTreePathService
    + ProvidesTransactionManager
where
    <Self as ProvidesTransactionManager>::T: TransactionContext,
    <Self as ProvidesTransactionManager>::T:
        ProvidesRelationshipRepository + ProvidesRelationshipService + ProvidesTreePathService,
{
}

#[derive(Debug, Error)]
pub enum GetRelationshipsUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    GetRelationshipsServiceError(#[from] GetRelationshipsServiceError),
    #[error(transparent)]
    GetWorldServiceError(#[from] GetWorldServiceError),
    #[error("not found")]
    NotFound,
}

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
pub trait UsesGetRelationshipsUsecase {
    async fn get_relationships(
        &self,
        body: GetRelationshipsSchema,
    ) -> Result<Vec<Relationship>, GetRelationshipsUsecaseError>;
}

#[async_trait]
impl<T> UsesGetRelationshipsUsecase for T
where
    T: RelationshipUsecase,
    <T as ProvidesTransactionManager>::T: TransactionContext,
    <T as ProvidesTransactionManager>::T:
        ProvidesRelationshipRepository + ProvidesRelationshipService + ProvidesTreePathService,
{
    async fn get_relationships(
        &self,
        body: GetRelationshipsSchema,
    ) -> Result<Vec<Relationship>, GetRelationshipsUsecaseError> {
        if body.world_id == 0 {
            return Err(GetRelationshipsUsecaseError::InvalidParams);
        }

        match self
            .world_service()
            .get_world(GetWorldSchema {
                world_id: body.world_id,
            })
            .await
        {
            Ok(_) => {}
            Err(GetWorldServiceError::NotFound) => {
                return Err(GetRelationshipsUsecaseError::NotFound)
            }
            Err(err) => return Err(GetRelationshipsUsecaseError::GetWorldServiceError(err)),
        }

        match self.relationship_service().get_relationships(body).await {
            Ok(relationships) => Ok(relationships),
            Err(GetRelationshipsServiceError::InvalidParams) => {
                Err(GetRelationshipsUsecaseError::InvalidParams)
            }
            Err(err) => Err(GetRelationshipsUsecaseError::GetRelationshipsServiceError(
                err,
            )),
        }
    }
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
    <T as ProvidesTransactionManager>::T: TransactionContext,
    <T as ProvidesTransactionManager>::T:
        ProvidesRelationshipRepository + ProvidesRelationshipService + ProvidesTreePathService,
{
    async fn create_relationship(
        &self,
        body: CreateRelationshipSchema,
    ) -> Result<(), CreateRelationshipUsecaseError> {
        if body.world_id == 0 {
            return Err(CreateRelationshipUsecaseError::InvalidParams);
        }

        let world_id = body.world_id;
        let affected_entity_ids =
            normalize_entity_ids(vec![body.source_entity_id, body.target_entity_id]);
        let tx = self.begin_transaction().await?;

        tx.relationship_service()
            .create_relationship(body)
            .await
            .map_err(map_create_relationship_service_error)?;

        tx.tree_path_service()
            .sync_tree_paths_by_entity_ids(SyncTreePathsByEntityIdsSchema {
                world_id,
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
    <T as ProvidesTransactionManager>::T:
        ProvidesRelationshipRepository + ProvidesRelationshipService + ProvidesTreePathService,
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

        let Some(world_id) = tx
            .relationship_repository()
            .load_relationship_world_id(LoadRelationshipWorldIdSchema {
                relationship_id: body.relationship_id,
            })
            .await
            .map_err(map_update_relationship_load_world_id_error)?
        else {
            return Err(UpdateRelationshipUsecaseError::NotFound);
        };

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
                world_id,
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
    <T as ProvidesTransactionManager>::T:
        ProvidesRelationshipRepository + ProvidesRelationshipService + ProvidesTreePathService,
{
    async fn delete_relationship(
        &self,
        body: DeleteRelationshipSchema,
    ) -> Result<(), DeleteRelationshipUsecaseError> {
        if body.relationship_id == 0 {
            return Err(DeleteRelationshipUsecaseError::InvalidParams);
        }

        let tx = self.begin_transaction().await?;

        let Some(world_id) = tx
            .relationship_repository()
            .load_relationship_world_id(LoadRelationshipWorldIdSchema {
                relationship_id: body.relationship_id,
            })
            .await
            .map_err(map_delete_relationship_load_world_id_error)?
        else {
            return Err(DeleteRelationshipUsecaseError::NotFound);
        };

        let endpoints = tx
            .relationship_service()
            .delete_relationship(body)
            .await
            .map_err(map_delete_relationship_service_error)?;

        tx.tree_path_service()
            .sync_tree_paths_by_entity_ids(SyncTreePathsByEntityIdsSchema {
                world_id,
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

fn map_update_relationship_load_world_id_error(
    err: LoadRelationshipWorldIdRepositoryError,
) -> UpdateRelationshipUsecaseError {
    UpdateRelationshipUsecaseError::TransactionError(match err {
        LoadRelationshipWorldIdRepositoryError::Db(err) => TransactionError::Db(err),
    })
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
                DeleteRelationshipRepositoryError::Db(err) => TransactionError::Db(err),
                DeleteRelationshipRepositoryError::NotFound => TransactionError::NotFound,
            })
        }
    }
}

fn map_delete_relationship_load_world_id_error(
    err: LoadRelationshipWorldIdRepositoryError,
) -> DeleteRelationshipUsecaseError {
    DeleteRelationshipUsecaseError::TransactionError(match err {
        LoadRelationshipWorldIdRepositoryError::Db(err) => TransactionError::Db(err),
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

fn map_tree_path_transaction_error(err: SyncTreePathsServiceError) -> TransactionError {
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
        SyncTreePathsServiceError::LoadEntitiesByWorldRepositoryError(err) => {
            TransactionError::Db(match err {
                creation_service::repository::entity::LoadEntitiesByWorldRepositoryError::Db(
                    err,
                ) => err,
            })
        }
        SyncTreePathsServiceError::LoadRelationshipEdgesByWorldIdRepositoryError(err) => {
            TransactionError::Db(match err {
                creation_service::repository::relationship::LoadRelationshipEdgesByWorldIdRepositoryError::Db(err) => err,
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

fn normalize_entity_ids(mut entity_ids: Vec<usize>) -> Vec<usize> {
    entity_ids.sort_unstable();
    entity_ids.dedup();
    entity_ids
}

fn map_create_relationship_transaction_error(
    err: TransactionError,
) -> CreateRelationshipUsecaseError {
    match err {
        TransactionError::NotFound => CreateRelationshipUsecaseError::NotFound,
        err => CreateRelationshipUsecaseError::TransactionError(err),
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

fn map_delete_relationship_transaction_error(
    err: TransactionError,
) -> DeleteRelationshipUsecaseError {
    match err {
        TransactionError::NotFound => DeleteRelationshipUsecaseError::NotFound,
        err => DeleteRelationshipUsecaseError::TransactionError(err),
    }
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
        body: GetRelationshipsSchema,
    ) -> Result<Vec<Relationship>, GetRelationshipsUsecaseError> {
        UsesGetRelationshipsUsecase::get_relationships(self, body).await
    }

    async fn create_relationship(
        &self,
        body: CreateRelationshipSchema,
    ) -> Result<(), CreateRelationshipUsecaseError> {
        UsesCreateRelationshipUsecase::create_relationship(self, body).await
    }

    async fn update_relationship(
        &self,
        body: UpdateRelationshipSchema,
    ) -> Result<(), UpdateRelationshipUsecaseError> {
        UsesUpdateRelationshipUsecase::update_relationship(self, body).await
    }

    async fn delete_relationship(
        &self,
        body: DeleteRelationshipSchema,
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

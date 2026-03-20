use async_trait::async_trait;
use thiserror::Error;

use crate::model::tree_path::{
    CreateTreePathsSchema, DeleteTreePathsByEntityIdsSchema, LoadStaleRelatedConnectionsSchema,
    LoadStaleRelatedEntityIdsSchema, TreePathConnection,
};

pub trait TreePathRepository: Send + Sync + 'static {}

#[derive(Debug, Error)]
pub enum LoadStaleRelatedEntityIdsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum LoadStaleRelatedConnectionsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum DeleteTreePathsByEntityIdsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum CreateTreePathsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[async_trait]
pub trait UsesTreePathRepository: Send + Sync + 'static {
    async fn load_stale_related_entity_ids(
        &self,
        body: LoadStaleRelatedEntityIdsSchema,
    ) -> Result<Vec<usize>, LoadStaleRelatedEntityIdsRepositoryError>;
    async fn load_stale_related_connections(
        &self,
        body: LoadStaleRelatedConnectionsSchema,
    ) -> Result<Vec<TreePathConnection>, LoadStaleRelatedConnectionsRepositoryError>;
    async fn delete_tree_paths_by_entity_ids(
        &self,
        body: DeleteTreePathsByEntityIdsSchema,
    ) -> Result<(), DeleteTreePathsByEntityIdsRepositoryError>;
    async fn create_tree_paths(
        &self,
        body: CreateTreePathsSchema,
    ) -> Result<(), CreateTreePathsRepositoryError>;
}

pub trait ProvidesTreePathRepository: Send + Sync + 'static {
    type T: UsesTreePathRepository;
    fn tree_path_repository(&self) -> &Self::T;
}

use async_trait::async_trait;
use thiserror::Error;

use crate::model::relationship::{
    CreateRelationshipSchema, DeleteRelationshipSchema, DeleteRelationshipsForEntitySchema,
    DiagramRelationshipEdge, GetRelationshipsSchema, LoadRelationshipEdgesByDiagramIdsSchema,
    LoadRelationshipEdgesSchema, Relationship, RelationshipEdge, RelationshipEndpoints,
    UpdateRelationshipSchema, UpdatedRelationshipEndpoints,
};

pub trait RelationshipRepository: Send + Sync + 'static {}

#[derive(Debug, Error)]
pub enum GetRelationshipsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum CreateRelationshipRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum UpdateRelationshipRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteRelationshipRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteRelationshipsForEntityRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum LoadRelationshipEdgesRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum LoadRelationshipEdgesByDiagramIdsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[async_trait]
pub trait UsesRelationshipRepository: Send + Sync + 'static {
    async fn get_relationships(
        &self,
        body: GetRelationshipsSchema,
    ) -> Result<Vec<Relationship>, GetRelationshipsRepositoryError>;
    async fn create_relationship(
        &self,
        body: CreateRelationshipSchema,
    ) -> Result<(), CreateRelationshipRepositoryError>;
    async fn update_relationship(
        &self,
        body: UpdateRelationshipSchema,
    ) -> Result<UpdatedRelationshipEndpoints, UpdateRelationshipRepositoryError>;
    async fn delete_relationship(
        &self,
        body: DeleteRelationshipSchema,
    ) -> Result<RelationshipEndpoints, DeleteRelationshipRepositoryError>;
    async fn delete_relationships_for_entity(
        &self,
        body: DeleteRelationshipsForEntitySchema,
    ) -> Result<Vec<usize>, DeleteRelationshipsForEntityRepositoryError>;
    async fn load_relationship_edges(
        &self,
        body: LoadRelationshipEdgesSchema,
    ) -> Result<Vec<RelationshipEdge>, LoadRelationshipEdgesRepositoryError>;
    async fn load_relationship_edges_by_diagram_ids(
        &self,
        body: LoadRelationshipEdgesByDiagramIdsSchema,
    ) -> Result<Vec<DiagramRelationshipEdge>, LoadRelationshipEdgesByDiagramIdsRepositoryError>;
}

pub trait ProvidesRelationshipRepository: Send + Sync + 'static {
    type T: UsesRelationshipRepository;
    fn relationship_repository(&self) -> &Self::T;
}

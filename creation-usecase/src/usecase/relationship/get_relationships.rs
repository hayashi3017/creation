use async_trait::async_trait;
use creation_service::{
    model::{
        diagram::ExistsActiveDiagramSchema,
        relationship::{GetRelationshipsSchema, Relationship},
    },
    repository::diagram::{ExistsActiveDiagramRepositoryError, UsesDiagramRepository},
    service::relationship::{GetRelationshipsServiceError, UsesRelationshipService},
};
use thiserror::Error;

use super::RelationshipUsecase;

#[derive(Debug, Error)]
pub enum GetRelationshipsUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    GetRelationshipsServiceError(#[from] GetRelationshipsServiceError),
    #[error(transparent)]
    ExistsActiveDiagramRepositoryError(#[from] ExistsActiveDiagramRepositoryError),
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
{
    async fn get_relationships(
        &self,
        body: GetRelationshipsSchema,
    ) -> Result<Vec<Relationship>, GetRelationshipsUsecaseError> {
        if body.diagram_id == 0 {
            return Err(GetRelationshipsUsecaseError::InvalidParams);
        }

        if !self
            .diagram_repository()
            .exists_active_diagram(ExistsActiveDiagramSchema {
                diagram_id: body.diagram_id,
            })
            .await?
        {
            return Err(GetRelationshipsUsecaseError::NotFound);
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

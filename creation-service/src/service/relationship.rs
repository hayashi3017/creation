use async_trait::async_trait;
use thiserror::Error;

use super::{map_service_result, normalize_optional_text};

use crate::{
    model::relationship::{
        CreateRelationshipSchema, DeleteRelationshipSchema, GetRelationshipsSchema, Relationship,
        RelationshipEndpoints, RelationshipKind, UpdateRelationshipSchema,
        UpdatedRelationshipEndpoints,
    },
    repository::relationship::{
        CreateRelationshipRepositoryError, DeleteRelationshipRepositoryError,
        GetRelationshipsRepositoryError, ProvidesRelationshipRepository,
        UpdateRelationshipRepositoryError, UsesRelationshipRepository,
    },
};

#[async_trait]
pub trait RelationshipService: ProvidesRelationshipRepository {}

#[derive(Debug, Error)]
pub enum GetRelationshipsServiceError {
    #[error(transparent)]
    GetRelationshipsRepositoryError(#[from] GetRelationshipsRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[derive(Debug, Error)]
pub enum CreateRelationshipServiceError {
    #[error(transparent)]
    CreateRelationshipRepositoryError(#[from] CreateRelationshipRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum UpdateRelationshipServiceError {
    #[error(transparent)]
    UpdateRelationshipRepositoryError(#[from] UpdateRelationshipRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteRelationshipServiceError {
    #[error(transparent)]
    DeleteRelationshipRepositoryError(#[from] DeleteRelationshipRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesRelationshipService {
    async fn get_relationships(
        &self,
        body: GetRelationshipsSchema,
    ) -> Result<Vec<Relationship>, GetRelationshipsServiceError>;
    async fn create_relationship(
        &self,
        body: CreateRelationshipSchema,
    ) -> Result<(), CreateRelationshipServiceError>;
    async fn update_relationship(
        &self,
        body: UpdateRelationshipSchema,
    ) -> Result<UpdatedRelationshipEndpoints, UpdateRelationshipServiceError>;
    async fn delete_relationship(
        &self,
        body: DeleteRelationshipSchema,
    ) -> Result<RelationshipEndpoints, DeleteRelationshipServiceError>;
}

#[async_trait]
impl<T: RelationshipService> UsesRelationshipService for T {
    async fn get_relationships(
        &self,
        body: GetRelationshipsSchema,
    ) -> Result<Vec<Relationship>, GetRelationshipsServiceError> {
        if body.diagram_id == 0 {
            return Err(GetRelationshipsServiceError::InvalidParams);
        }

        map_service_result!(
            self.relationship_repository().get_relationships(body),
            GetRelationshipsServiceError::GetRelationshipsRepositoryError
        )
    }

    async fn create_relationship(
        &self,
        body: CreateRelationshipSchema,
    ) -> Result<(), CreateRelationshipServiceError> {
        let Some(body) = prepare_create_relationship(body) else {
            return Err(CreateRelationshipServiceError::InvalidParams);
        };

        match self
            .relationship_repository()
            .create_relationship(body.clone())
            .await
        {
            Ok(()) => {}
            Err(CreateRelationshipRepositoryError::NotFound) => {
                return Err(CreateRelationshipServiceError::NotFound);
            }
            Err(err) => {
                return Err(CreateRelationshipServiceError::CreateRelationshipRepositoryError(err))
            }
        }

        Ok(())
    }

    async fn update_relationship(
        &self,
        body: UpdateRelationshipSchema,
    ) -> Result<UpdatedRelationshipEndpoints, UpdateRelationshipServiceError> {
        let Some(body) = prepare_update_relationship(body) else {
            return Err(UpdateRelationshipServiceError::InvalidParams);
        };

        match self
            .relationship_repository()
            .update_relationship(body)
            .await
        {
            Ok(previous_endpoints) => Ok(previous_endpoints),
            Err(UpdateRelationshipRepositoryError::NotFound) => {
                Err(UpdateRelationshipServiceError::NotFound)
            }
            Err(err) => Err(UpdateRelationshipServiceError::UpdateRelationshipRepositoryError(err)),
        }
    }

    async fn delete_relationship(
        &self,
        body: DeleteRelationshipSchema,
    ) -> Result<RelationshipEndpoints, DeleteRelationshipServiceError> {
        let Some(body) = prepare_delete_relationship(body) else {
            return Err(DeleteRelationshipServiceError::InvalidParams);
        };

        match self
            .relationship_repository()
            .delete_relationship(body)
            .await
        {
            Ok(endpoints) => Ok(endpoints),
            Err(DeleteRelationshipRepositoryError::NotFound) => {
                Err(DeleteRelationshipServiceError::NotFound)
            }
            Err(err) => Err(DeleteRelationshipServiceError::DeleteRelationshipRepositoryError(err)),
        }
    }
}

pub fn prepare_create_relationship(
    body: CreateRelationshipSchema,
) -> Option<CreateRelationshipSchema> {
    if body.diagram_id == 0 {
        return None;
    }

    normalize_relationship_body(
        body.source_entity_id,
        body.target_entity_id,
        body.kind,
        body.start_date,
        body.end_date,
        body.notes,
    )
    .map(
        |(source_entity_id, target_entity_id, kind, start_date, end_date, notes)| {
            CreateRelationshipSchema {
                diagram_id: body.diagram_id,
                source_entity_id,
                target_entity_id,
                kind,
                start_date,
                end_date,
                notes,
            }
        },
    )
}

pub fn prepare_update_relationship(
    body: UpdateRelationshipSchema,
) -> Option<UpdateRelationshipSchema> {
    if body.id == 0 {
        return None;
    }

    normalize_relationship_body(
        body.source_entity_id,
        body.target_entity_id,
        body.kind,
        body.start_date,
        body.end_date,
        body.notes,
    )
    .map(
        |(source_entity_id, target_entity_id, kind, start_date, end_date, notes)| {
            UpdateRelationshipSchema {
                id: body.id,
                source_entity_id,
                target_entity_id,
                kind,
                start_date,
                end_date,
                notes,
            }
        },
    )
}

pub fn prepare_delete_relationship(
    body: DeleteRelationshipSchema,
) -> Option<DeleteRelationshipSchema> {
    if body.id == 0 {
        None
    } else {
        Some(body)
    }
}

fn normalize_relationship_body(
    source_entity_id: usize,
    target_entity_id: usize,
    kind: RelationshipKind,
    start_date: Option<chrono::NaiveDate>,
    end_date: Option<chrono::NaiveDate>,
    notes: Option<String>,
) -> Option<(
    usize,
    usize,
    RelationshipKind,
    Option<chrono::NaiveDate>,
    Option<chrono::NaiveDate>,
    Option<String>,
)> {
    if source_entity_id == 0 || target_entity_id == 0 || source_entity_id == target_entity_id {
        return None;
    }

    if !kind.is_lineage() {
        return None;
    }

    if start_date
        .zip(end_date)
        .is_some_and(|(start, end)| start > end)
    {
        return None;
    }

    Some((
        source_entity_id,
        target_entity_id,
        kind,
        start_date,
        end_date,
        normalize_optional_text(notes),
    ))
}

pub trait ProvidesRelationshipService: Send + Sync + 'static {
    type T: RelationshipService;
    fn relationship_service(&self) -> &Self::T;
}

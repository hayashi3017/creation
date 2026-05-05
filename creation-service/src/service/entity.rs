use async_trait::async_trait;
use thiserror::Error;

use super::{map_service_result, normalize_name, normalize_optional_text};

use crate::{
    model::entity::{
        CreateDiagramEntityMembershipSchema, CreateEntitySchema,
        DeleteDiagramEntityMembershipsSchema, DeleteEntitySchema, Entity, GetEntitiesSchema,
        LoadEntitiesByDiagramIdsSchema, LoadEntitiesByWorldSchema, UpdateEntitySchema,
        ENTITY_NAME_MAX_CHARS,
    },
    repository::entity::{
        CreateDiagramEntityMembershipRepositoryError, CreateEntityRepositoryError,
        DeleteDiagramEntityMembershipsRepositoryError, DeleteEntityRepositoryError,
        GetEntitiesRepositoryError, LoadEntitiesByDiagramIdsRepositoryError,
        LoadEntitiesByWorldRepositoryError, ProvidesEntityRepository, UpdateEntityRepositoryError,
        UsesEntityRepository,
    },
};

#[async_trait]
pub trait EntityService: ProvidesEntityRepository {}

#[derive(Debug, Error)]
pub enum EntityServiceError {
    #[error(transparent)]
    GetEntitiesServiceError(#[from] GetEntitiesServiceError),
    #[error(transparent)]
    CreateEntityServiceError(#[from] CreateEntityServiceError),
    #[error(transparent)]
    CreateDiagramEntityMembershipServiceError(#[from] CreateDiagramEntityMembershipServiceError),
    #[error(transparent)]
    UpdateEntityServiceError(#[from] UpdateEntityServiceError),
    #[error(transparent)]
    DeleteEntityServiceError(#[from] DeleteEntityServiceError),
    #[error(transparent)]
    DeleteDiagramEntityMembershipsServiceError(#[from] DeleteDiagramEntityMembershipsServiceError),
    #[error(transparent)]
    LoadEntitiesByDiagramIdsServiceError(#[from] LoadEntitiesByDiagramIdsServiceError),
    #[error(transparent)]
    LoadEntitiesByWorldServiceError(#[from] LoadEntitiesByWorldServiceError),
}

#[derive(Debug, Error)]
pub enum GetEntitiesServiceError {
    #[error(transparent)]
    GetEntitiesRepositoryError(#[from] GetEntitiesRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[derive(Debug, Error)]
pub enum CreateEntityServiceError {
    #[error(transparent)]
    CreateEntityRepositoryError(#[from] CreateEntityRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[derive(Debug, Error)]
pub enum CreateDiagramEntityMembershipServiceError {
    #[error(transparent)]
    CreateDiagramEntityMembershipRepositoryError(
        #[from] CreateDiagramEntityMembershipRepositoryError,
    ),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum UpdateEntityServiceError {
    #[error(transparent)]
    UpdateEntityRepositoryError(#[from] UpdateEntityRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteEntityServiceError {
    #[error(transparent)]
    DeleteEntityRepositoryError(#[from] DeleteEntityRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteDiagramEntityMembershipsServiceError {
    #[error(transparent)]
    DeleteDiagramEntityMembershipsRepositoryError(
        #[from] DeleteDiagramEntityMembershipsRepositoryError,
    ),
    #[error("invalid parameter")]
    InvalidParams,
}

#[derive(Debug, Error)]
pub enum LoadEntitiesByDiagramIdsServiceError {
    #[error(transparent)]
    LoadEntitiesByDiagramIdsRepositoryError(#[from] LoadEntitiesByDiagramIdsRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[derive(Debug, Error)]
pub enum LoadEntitiesByWorldServiceError {
    #[error(transparent)]
    LoadEntitiesByWorldRepositoryError(#[from] LoadEntitiesByWorldRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[async_trait]
pub trait UsesEntityService {
    async fn get_entities(
        &self,
        body: GetEntitiesSchema,
    ) -> Result<Vec<Entity>, GetEntitiesServiceError>;
    async fn create_entity(
        &self,
        body: CreateEntitySchema,
    ) -> Result<usize, CreateEntityServiceError>;
    async fn create_diagram_entity_membership(
        &self,
        body: CreateDiagramEntityMembershipSchema,
    ) -> Result<(), CreateDiagramEntityMembershipServiceError>;
    async fn update_entity(&self, body: UpdateEntitySchema)
        -> Result<(), UpdateEntityServiceError>;
    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<usize, DeleteEntityServiceError>;
    async fn delete_diagram_entity_memberships(
        &self,
        body: DeleteDiagramEntityMembershipsSchema,
    ) -> Result<Vec<usize>, DeleteDiagramEntityMembershipsServiceError>;
    async fn load_entities_by_diagram_ids(
        &self,
        body: LoadEntitiesByDiagramIdsSchema,
    ) -> Result<Vec<Entity>, LoadEntitiesByDiagramIdsServiceError>;
    async fn load_entities_by_world(
        &self,
        body: LoadEntitiesByWorldSchema,
    ) -> Result<Vec<Entity>, LoadEntitiesByWorldServiceError>;
}

#[async_trait]
impl<T: EntityService> UsesEntityService for T {
    async fn get_entities(
        &self,
        body: GetEntitiesSchema,
    ) -> Result<Vec<Entity>, GetEntitiesServiceError> {
        if body.world_id == 0 || body.diagram_id == 0 {
            return Err(GetEntitiesServiceError::InvalidParams);
        }

        map_service_result!(
            self.entity_repository().get_entities(body),
            GetEntitiesServiceError::GetEntitiesRepositoryError
        )
    }

    async fn create_entity(
        &self,
        body: CreateEntitySchema,
    ) -> Result<usize, CreateEntityServiceError> {
        let Some(body) = prepare_create_entity(body) else {
            return Err(CreateEntityServiceError::InvalidParams);
        };

        match self.entity_repository().create_entity(body).await {
            Ok(entity_id) => Ok(entity_id),
            Err(CreateEntityRepositoryError::NotFound) => {
                Err(CreateEntityServiceError::InvalidParams)
            }
            Err(err) => Err(CreateEntityServiceError::CreateEntityRepositoryError(err)),
        }
    }

    async fn create_diagram_entity_membership(
        &self,
        body: CreateDiagramEntityMembershipSchema,
    ) -> Result<(), CreateDiagramEntityMembershipServiceError> {
        if body.diagram_id == 0 || body.entity_id == 0 {
            return Err(CreateDiagramEntityMembershipServiceError::InvalidParams);
        }

        match self
            .entity_repository()
            .create_diagram_entity_membership(body)
            .await
        {
            Ok(()) => Ok(()),
            Err(CreateDiagramEntityMembershipRepositoryError::NotFound) => {
                Err(CreateDiagramEntityMembershipServiceError::NotFound)
            }
            Err(err) => Err(
                CreateDiagramEntityMembershipServiceError::CreateDiagramEntityMembershipRepositoryError(
                    err,
                ),
            ),
        }
    }

    async fn update_entity(
        &self,
        body: UpdateEntitySchema,
    ) -> Result<(), UpdateEntityServiceError> {
        let Some(body) = prepare_update_entity(body) else {
            return Err(UpdateEntityServiceError::InvalidParams);
        };

        match self.entity_repository().update_entity(body).await {
            Ok(()) => Ok(()),
            Err(UpdateEntityRepositoryError::NotFound) => Err(UpdateEntityServiceError::NotFound),
            Err(err) => Err(UpdateEntityServiceError::UpdateEntityRepositoryError(err)),
        }
    }

    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<usize, DeleteEntityServiceError> {
        let Some(body) = prepare_delete_entity(body) else {
            return Err(DeleteEntityServiceError::InvalidParams);
        };

        match self.entity_repository().delete_entity(body).await {
            Ok(diagram_id) => Ok(diagram_id),
            Err(DeleteEntityRepositoryError::NotFound) => Err(DeleteEntityServiceError::NotFound),
            Err(err) => Err(DeleteEntityServiceError::DeleteEntityRepositoryError(err)),
        }
    }

    async fn delete_diagram_entity_memberships(
        &self,
        body: DeleteDiagramEntityMembershipsSchema,
    ) -> Result<Vec<usize>, DeleteDiagramEntityMembershipsServiceError> {
        if body.diagram_id == 0 {
            return Err(DeleteDiagramEntityMembershipsServiceError::InvalidParams);
        }

        self.entity_repository()
            .delete_diagram_entity_memberships(body)
            .await
            .map_err(DeleteDiagramEntityMembershipsServiceError::DeleteDiagramEntityMembershipsRepositoryError)
    }

    async fn load_entities_by_diagram_ids(
        &self,
        body: LoadEntitiesByDiagramIdsSchema,
    ) -> Result<Vec<Entity>, LoadEntitiesByDiagramIdsServiceError> {
        if body.diagram_ids.iter().any(|diagram_id| *diagram_id == 0) {
            return Err(LoadEntitiesByDiagramIdsServiceError::InvalidParams);
        }

        if body.diagram_ids.is_empty() {
            return Ok(Vec::new());
        }

        self.entity_repository()
            .load_entities_by_diagram_ids(body)
            .await
            .map_err(LoadEntitiesByDiagramIdsServiceError::LoadEntitiesByDiagramIdsRepositoryError)
    }

    async fn load_entities_by_world(
        &self,
        body: LoadEntitiesByWorldSchema,
    ) -> Result<Vec<Entity>, LoadEntitiesByWorldServiceError> {
        if body.world_id == 0 {
            return Err(LoadEntitiesByWorldServiceError::InvalidParams);
        }

        self.entity_repository()
            .load_entities_by_world(body)
            .await
            .map_err(LoadEntitiesByWorldServiceError::LoadEntitiesByWorldRepositoryError)
    }
}

pub fn prepare_create_entity(body: CreateEntitySchema) -> Option<CreateEntitySchema> {
    if body.world_id == 0 {
        return None;
    }

    let name = normalize_name(&body.name, ENTITY_NAME_MAX_CHARS)?;

    Some(CreateEntitySchema {
        world_id: body.world_id,
        kind: body.kind,
        name,
        description: normalize_optional_text(body.description),
    })
}

pub fn prepare_update_entity(body: UpdateEntitySchema) -> Option<UpdateEntitySchema> {
    if body.entity_id == 0 || body.world_id == 0 {
        return None;
    }

    let name = normalize_name(&body.name, ENTITY_NAME_MAX_CHARS)?;

    Some(UpdateEntitySchema {
        entity_id: body.entity_id,
        world_id: body.world_id,
        kind: body.kind,
        name,
        description: normalize_optional_text(body.description),
    })
}

pub fn prepare_delete_entity(body: DeleteEntitySchema) -> Option<DeleteEntitySchema> {
    if body.entity_id == 0 || body.world_id == 0 {
        None
    } else {
        Some(body)
    }
}

pub trait ProvidesEntityService: Send + Sync + 'static {
    type T: EntityService;
    fn entity_service(&self) -> &Self::T;
}

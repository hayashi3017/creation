use async_trait::async_trait;
use thiserror::Error;

use super::{map_service_result, normalize_name, normalize_optional_text};

use crate::{
    model::entity::{
        CreateDiagramEntityMembershipSchema, CreateDiagramEntityMembershipsSchema,
        CreateEntitySchema, DeleteDiagramEntityMembershipSchema,
        DeleteDiagramEntityMembershipsByEntityIdsSchema, DeleteDiagramEntityMembershipsSchema,
        DeleteEntitySchema, Entity, GetEntitiesSchema, LoadActiveEntityIdsSchema,
        LoadEntitiesByDiagramIdsSchema, LoadEntitiesByWorldSchema, LoadSeedEntitiesSchema,
        SeedEntity, SyncEntityDiagramMembershipsSchema, UpdateEntitySchema, ENTITY_NAME_MAX_CHARS,
    },
    repository::entity::{
        CreateDiagramEntityMembershipRepositoryError, CreateEntityRepositoryError,
        DeleteDiagramEntityMembershipRepositoryError,
        DeleteDiagramEntityMembershipsRepositoryError, DeleteEntityRepositoryError,
        GetEntitiesRepositoryError, LoadActiveEntityIdsRepositoryError,
        LoadEntitiesByDiagramIdsRepositoryError, LoadEntitiesByWorldRepositoryError,
        LoadSeedEntitiesRepositoryError, ProvidesEntityRepository,
        SyncEntityDiagramMembershipsRepositoryError, UpdateEntityRepositoryError,
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
    DeleteDiagramEntityMembershipServiceError(#[from] DeleteDiagramEntityMembershipServiceError),
    #[error(transparent)]
    SyncEntityDiagramMembershipsServiceError(#[from] SyncEntityDiagramMembershipsServiceError),
    #[error(transparent)]
    UpdateEntityServiceError(#[from] UpdateEntityServiceError),
    #[error(transparent)]
    DeleteEntityServiceError(#[from] DeleteEntityServiceError),
    #[error(transparent)]
    DeleteDiagramEntityMembershipsServiceError(#[from] DeleteDiagramEntityMembershipsServiceError),
    #[error(transparent)]
    LoadActiveEntityIdsServiceError(#[from] LoadActiveEntityIdsServiceError),
    #[error(transparent)]
    LoadEntitiesByDiagramIdsServiceError(#[from] LoadEntitiesByDiagramIdsServiceError),
    #[error(transparent)]
    LoadEntitiesByWorldServiceError(#[from] LoadEntitiesByWorldServiceError),
    #[error(transparent)]
    LoadSeedEntitiesServiceError(#[from] LoadSeedEntitiesServiceError),
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
pub enum DeleteDiagramEntityMembershipServiceError {
    #[error(transparent)]
    DeleteDiagramEntityMembershipRepositoryError(
        #[from] DeleteDiagramEntityMembershipRepositoryError,
    ),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum SyncEntityDiagramMembershipsServiceError {
    #[error(transparent)]
    SyncEntityDiagramMembershipsRepositoryError(
        #[from] SyncEntityDiagramMembershipsRepositoryError,
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
pub enum LoadActiveEntityIdsServiceError {
    #[error(transparent)]
    LoadActiveEntityIdsRepositoryError(#[from] LoadActiveEntityIdsRepositoryError),
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

#[derive(Debug, Error)]
pub enum LoadSeedEntitiesServiceError {
    #[error(transparent)]
    LoadSeedEntitiesRepositoryError(#[from] LoadSeedEntitiesRepositoryError),
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
    async fn create_diagram_entity_memberships(
        &self,
        body: CreateDiagramEntityMembershipsSchema,
    ) -> Result<(), CreateDiagramEntityMembershipServiceError>;
    async fn delete_diagram_entity_membership(
        &self,
        body: DeleteDiagramEntityMembershipSchema,
    ) -> Result<(), DeleteDiagramEntityMembershipServiceError>;
    async fn delete_diagram_entity_memberships_by_entity_ids(
        &self,
        body: DeleteDiagramEntityMembershipsByEntityIdsSchema,
    ) -> Result<(), DeleteDiagramEntityMembershipServiceError>;
    async fn sync_entity_diagram_memberships(
        &self,
        body: SyncEntityDiagramMembershipsSchema,
    ) -> Result<(), SyncEntityDiagramMembershipsServiceError>;
    async fn update_entity(&self, body: UpdateEntitySchema)
        -> Result<(), UpdateEntityServiceError>;
    async fn delete_entity(
        &self,
        body: DeleteEntitySchema,
    ) -> Result<usize, DeleteEntityServiceError>;
    async fn load_active_entity_ids(
        &self,
        body: LoadActiveEntityIdsSchema,
    ) -> Result<Vec<usize>, LoadActiveEntityIdsServiceError>;
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
    async fn load_seed_entities(
        &self,
        body: LoadSeedEntitiesSchema,
    ) -> Result<Vec<SeedEntity>, LoadSeedEntitiesServiceError>;
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

    async fn create_diagram_entity_memberships(
        &self,
        body: CreateDiagramEntityMembershipsSchema,
    ) -> Result<(), CreateDiagramEntityMembershipServiceError> {
        if body.diagram_id == 0 || body.entity_ids.iter().any(|entity_id| *entity_id == 0) {
            return Err(CreateDiagramEntityMembershipServiceError::InvalidParams);
        }

        self.entity_repository()
            .create_diagram_entity_memberships(body)
            .await
            .map_err(CreateDiagramEntityMembershipServiceError::CreateDiagramEntityMembershipRepositoryError)
    }

    async fn delete_diagram_entity_membership(
        &self,
        body: DeleteDiagramEntityMembershipSchema,
    ) -> Result<(), DeleteDiagramEntityMembershipServiceError> {
        if body.diagram_id == 0 || body.entity_id == 0 {
            return Err(DeleteDiagramEntityMembershipServiceError::InvalidParams);
        }

        match self
            .entity_repository()
            .delete_diagram_entity_membership(body)
            .await
        {
            Ok(()) => Ok(()),
            Err(DeleteDiagramEntityMembershipRepositoryError::NotFound) => {
                Err(DeleteDiagramEntityMembershipServiceError::NotFound)
            }
            Err(err) => Err(
                DeleteDiagramEntityMembershipServiceError::DeleteDiagramEntityMembershipRepositoryError(
                    err,
                ),
            ),
        }
    }

    async fn delete_diagram_entity_memberships_by_entity_ids(
        &self,
        body: DeleteDiagramEntityMembershipsByEntityIdsSchema,
    ) -> Result<(), DeleteDiagramEntityMembershipServiceError> {
        if body.diagram_id == 0 || body.entity_ids.iter().any(|entity_id| *entity_id == 0) {
            return Err(DeleteDiagramEntityMembershipServiceError::InvalidParams);
        }

        self.entity_repository()
            .delete_diagram_entity_memberships_by_entity_ids(body)
            .await
            .map_err(DeleteDiagramEntityMembershipServiceError::DeleteDiagramEntityMembershipRepositoryError)
    }

    async fn load_active_entity_ids(
        &self,
        body: LoadActiveEntityIdsSchema,
    ) -> Result<Vec<usize>, LoadActiveEntityIdsServiceError> {
        if body.diagram_id == 0 {
            return Err(LoadActiveEntityIdsServiceError::InvalidParams);
        }

        self.entity_repository()
            .load_active_entity_ids(body)
            .await
            .map_err(LoadActiveEntityIdsServiceError::LoadActiveEntityIdsRepositoryError)
    }

    async fn sync_entity_diagram_memberships(
        &self,
        body: SyncEntityDiagramMembershipsSchema,
    ) -> Result<(), SyncEntityDiagramMembershipsServiceError> {
        if body.entity_id == 0 || body.world_id == 0 || body.diagram_ids.contains(&0) {
            return Err(SyncEntityDiagramMembershipsServiceError::InvalidParams);
        }

        match self
            .entity_repository()
            .sync_entity_diagram_memberships(body)
            .await
        {
            Ok(()) => Ok(()),
            Err(SyncEntityDiagramMembershipsRepositoryError::NotFound) => {
                Err(SyncEntityDiagramMembershipsServiceError::NotFound)
            }
            Err(err) => Err(
                SyncEntityDiagramMembershipsServiceError::SyncEntityDiagramMembershipsRepositoryError(
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

    async fn load_seed_entities(
        &self,
        body: LoadSeedEntitiesSchema,
    ) -> Result<Vec<SeedEntity>, LoadSeedEntitiesServiceError> {
        if body.entity_ids.iter().any(|entity_id| *entity_id == 0) {
            return Err(LoadSeedEntitiesServiceError::InvalidParams);
        }

        if body.entity_ids.is_empty() {
            return Ok(Vec::new());
        }

        self.entity_repository()
            .load_seed_entities(body)
            .await
            .map_err(LoadSeedEntitiesServiceError::LoadSeedEntitiesRepositoryError)
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

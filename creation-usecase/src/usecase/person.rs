use std::collections::HashMap;

use async_trait::async_trait;
use creation_service::{
    model::{
        entity::{
            CreateDiagramEntityMembershipSchema, CreateEntitySchema, DeleteEntitySchema,
            EntityKind, GetEntitiesSchema, UpdateEntitySchema,
        },
        person::{
            CreatePersonRecordSchema, CreatePersonSchema, DeletePersonSchema,
            GetPersonRecordsSchema, GetPersonsSchema, Person, PersonRecord,
            UpdatePersonRecordSchema, UpdatePersonSchema,
        },
        relationship::DeleteRelationshipsForEntitySchema,
        tree_path::SyncTreePathsByEntityIdsSchema,
    },
    repository::relationship::{
        DeleteRelationshipsForEntityRepositoryError, ProvidesRelationshipRepository,
        UsesRelationshipRepository,
    },
    service::{
        entity::{
            CreateDiagramEntityMembershipServiceError, CreateEntityServiceError,
            DeleteEntityServiceError, GetEntitiesServiceError, ProvidesEntityService,
            UpdateEntityServiceError, UsesEntityService,
        },
        person::{
            prepare_create_person, prepare_delete_person, prepare_update_person,
            CreatePersonRecordServiceError, DeletePersonRecordServiceError,
            GetPersonRecordsServiceError, ProvidesPersonService, UpdatePersonRecordServiceError,
            UsesPersonService,
        },
        transaction::{
            BeginTransactionError, ProvidesTransactionManager, TransactionContext, TransactionError,
        },
        tree_path::{ProvidesTreePathService, SyncTreePathsServiceError, UsesTreePathService},
    },
};
use thiserror::Error;

#[async_trait]
pub trait PersonUsecase:
    ProvidesEntityService + ProvidesPersonService + ProvidesTransactionManager
where
    <Self as ProvidesTransactionManager>::T: TransactionContext,
    <Self as ProvidesTransactionManager>::T: ProvidesEntityService
        + ProvidesPersonService
        + ProvidesRelationshipRepository
        + ProvidesTreePathService,
{
}

#[derive(Debug, Error)]
pub enum PersonUsecaseError {
    #[error(transparent)]
    GetPersonsUsecaseError(#[from] GetPersonsUsecaseError),
    #[error(transparent)]
    CreatePersonUsecaseError(#[from] CreatePersonUsecaseError),
    #[error(transparent)]
    UpdatePersonUsecaseError(#[from] UpdatePersonUsecaseError),
    #[error(transparent)]
    DeletePersonUsecaseError(#[from] DeletePersonUsecaseError),
}

#[derive(Debug, Error)]
pub enum GetPersonsUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    GetEntitiesServiceError(#[from] GetEntitiesServiceError),
    #[error(transparent)]
    GetPersonRecordsServiceError(#[from] GetPersonRecordsServiceError),
}

#[derive(Debug, Error)]
pub enum CreatePersonUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    BeginTransactionError(#[from] BeginTransactionError),
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
}

#[derive(Debug, Error)]
pub enum UpdatePersonUsecaseError {
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
pub enum DeletePersonUsecaseError {
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
pub trait UsesGetPersonsUsecase {
    async fn get_persons(
        &self,
        body: GetPersonsSchema,
    ) -> Result<Vec<Person>, GetPersonsUsecaseError>;
}

#[async_trait]
impl<T> UsesGetPersonsUsecase for T
where
    T: PersonUsecase,
    <T as ProvidesTransactionManager>::T: TransactionContext,
    <T as ProvidesTransactionManager>::T: ProvidesEntityService
        + ProvidesPersonService
        + ProvidesRelationshipRepository
        + ProvidesTreePathService,
{
    async fn get_persons(
        &self,
        body: GetPersonsSchema,
    ) -> Result<Vec<Person>, GetPersonsUsecaseError> {
        if body.diagram_id == 0 {
            return Err(GetPersonsUsecaseError::InvalidParams);
        }

        let entities = match self
            .entity_service()
            .get_entities(GetEntitiesSchema {
                diagram_id: body.diagram_id,
            })
            .await
        {
            Ok(entities) => entities,
            Err(GetEntitiesServiceError::InvalidParams) => {
                return Err(GetPersonsUsecaseError::InvalidParams);
            }
            Err(err) => return Err(GetPersonsUsecaseError::GetEntitiesServiceError(err)),
        };

        let person_entities: Vec<_> = entities
            .into_iter()
            .filter(|entity| matches!(entity.kind, EntityKind::Person))
            .collect();

        if person_entities.is_empty() {
            return Ok(Vec::new());
        }

        let entity_ids = person_entities
            .iter()
            .map(|entity| entity.entity_id)
            .collect::<Vec<_>>();

        let person_records = match self
            .person_service()
            .get_person_records(GetPersonRecordsSchema { entity_ids })
            .await
        {
            Ok(person_records) => person_records,
            Err(GetPersonRecordsServiceError::InvalidParams) => {
                return Err(GetPersonsUsecaseError::InvalidParams);
            }
            Err(err) => return Err(GetPersonsUsecaseError::GetPersonRecordsServiceError(err)),
        };

        Ok(merge_persons(person_entities, person_records))
    }
}

#[async_trait]
pub trait UsesCreatePersonUsecase {
    async fn create_person(&self, body: CreatePersonSchema)
        -> Result<(), CreatePersonUsecaseError>;
}

#[async_trait]
impl<T> UsesCreatePersonUsecase for T
where
    T: PersonUsecase,
    <T as ProvidesTransactionManager>::T: TransactionContext,
    <T as ProvidesTransactionManager>::T: ProvidesEntityService
        + ProvidesPersonService
        + ProvidesRelationshipRepository
        + ProvidesTreePathService,
{
    async fn create_person(
        &self,
        body: CreatePersonSchema,
    ) -> Result<(), CreatePersonUsecaseError> {
        let body = prepare_create_person(body).ok_or(CreatePersonUsecaseError::InvalidParams)?;

        let tx = self.begin_transaction().await?;

        let entity_id = tx
            .entity_service()
            .create_entity(CreateEntitySchema {
                world_id: body.world_id,
                kind: EntityKind::Person,
                name: body.name,
                description: body.description,
            })
            .await
            .map_err(map_create_person_entity_error)?;

        if let Some(diagram_id) = body.diagram_id {
            tx.entity_service()
                .create_diagram_entity_membership(CreateDiagramEntityMembershipSchema {
                    diagram_id,
                    entity_id,
                })
                .await
                .map_err(map_create_person_membership_error)?;
        }

        tx.person_service()
            .create_person_record(CreatePersonRecordSchema {
                entity_id,
                gender: body.gender,
                birth_date: body.birth_date,
                death_date: body.death_date,
                birthplace: body.birthplace,
                residence: body.residence,
                photo_url: body.photo_url,
            })
            .await
            .map_err(map_create_person_record_error)?;

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait]
pub trait UsesUpdatePersonUsecase {
    async fn update_person(&self, body: UpdatePersonSchema)
        -> Result<(), UpdatePersonUsecaseError>;
}

#[async_trait]
impl<T> UsesUpdatePersonUsecase for T
where
    T: PersonUsecase,
    <T as ProvidesTransactionManager>::T: TransactionContext,
    <T as ProvidesTransactionManager>::T: ProvidesEntityService
        + ProvidesPersonService
        + ProvidesRelationshipRepository
        + ProvidesTreePathService,
{
    async fn update_person(
        &self,
        body: UpdatePersonSchema,
    ) -> Result<(), UpdatePersonUsecaseError> {
        let body = prepare_update_person(body).ok_or(UpdatePersonUsecaseError::InvalidParams)?;

        let tx = self.begin_transaction().await?;

        tx.entity_service()
            .update_entity(UpdateEntitySchema {
                entity_id: body.entity_id,
                kind: EntityKind::Person,
                name: body.name,
                description: body.description,
            })
            .await
            .map_err(map_update_person_entity_error)?;

        tx.person_service()
            .update_person_record(UpdatePersonRecordSchema {
                entity_id: body.entity_id,
                gender: body.gender,
                birth_date: body.birth_date,
                death_date: body.death_date,
                birthplace: body.birthplace,
                residence: body.residence,
                photo_url: body.photo_url,
            })
            .await
            .map_err(map_update_person_record_error)?;

        tx.commit()
            .await
            .map_err(map_update_person_transaction_error)?;

        Ok(())
    }
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
            .delete_entity(DeleteEntitySchema { entity_id })
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

fn merge_persons(
    person_entities: Vec<creation_service::model::entity::Entity>,
    person_records: Vec<PersonRecord>,
) -> Vec<Person> {
    let mut records_by_entity_id: HashMap<usize, PersonRecord> = person_records
        .into_iter()
        .map(|record| (record.entity_id, record))
        .collect();

    person_entities
        .into_iter()
        .filter_map(|entity| {
            records_by_entity_id
                .remove(&entity.entity_id)
                .map(|record| Person {
                    entity_id: entity.entity_id,
                    diagram_id: entity.diagram_id,
                    name: entity.name,
                    description: entity.description,
                    gender: record.gender,
                    birth_date: record.birth_date,
                    death_date: record.death_date,
                    birthplace: record.birthplace,
                    residence: record.residence,
                    photo_url: record.photo_url,
                })
        })
        .collect()
}

fn map_create_person_entity_error(err: CreateEntityServiceError) -> CreatePersonUsecaseError {
    match err {
        CreateEntityServiceError::CreateEntityRepositoryError(err) => {
            CreatePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                creation_service::repository::entity::CreateEntityRepositoryError::Db(err) => err,
                creation_service::repository::entity::CreateEntityRepositoryError::NotFound => {
                    sqlx::Error::RowNotFound
                }
            }))
        }
        CreateEntityServiceError::InvalidParams => CreatePersonUsecaseError::InvalidParams,
    }
}

fn map_create_person_membership_error(
    err: CreateDiagramEntityMembershipServiceError,
) -> CreatePersonUsecaseError {
    match err {
        CreateDiagramEntityMembershipServiceError::InvalidParams
        | CreateDiagramEntityMembershipServiceError::NotFound => {
            CreatePersonUsecaseError::InvalidParams
        }
        CreateDiagramEntityMembershipServiceError::CreateDiagramEntityMembershipRepositoryError(
            err,
        ) => CreatePersonUsecaseError::TransactionError(TransactionError::Db(match err {
            creation_service::repository::entity::CreateDiagramEntityMembershipRepositoryError::Db(
                err,
            ) => err,
            creation_service::repository::entity::CreateDiagramEntityMembershipRepositoryError::NotFound => {
                sqlx::Error::RowNotFound
            }
        })),
    }
}

fn map_create_person_record_error(err: CreatePersonRecordServiceError) -> CreatePersonUsecaseError {
    match err {
        CreatePersonRecordServiceError::CreatePersonRepositoryError(err) => {
            CreatePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                creation_service::repository::person::CreatePersonRepositoryError::Db(err) => err,
            }))
        }
        CreatePersonRecordServiceError::InvalidParams => CreatePersonUsecaseError::InvalidParams,
    }
}

fn map_update_person_entity_error(err: UpdateEntityServiceError) -> UpdatePersonUsecaseError {
    match err {
        UpdateEntityServiceError::NotFound => UpdatePersonUsecaseError::NotFound,
        UpdateEntityServiceError::UpdateEntityRepositoryError(err) => {
            UpdatePersonUsecaseError::TransactionError(match err {
                creation_service::repository::entity::UpdateEntityRepositoryError::Db(err) => {
                    TransactionError::Db(err)
                }
                creation_service::repository::entity::UpdateEntityRepositoryError::NotFound => {
                    TransactionError::NotFound
                }
            })
        }
        UpdateEntityServiceError::InvalidParams => UpdatePersonUsecaseError::InvalidParams,
    }
}

fn map_update_person_record_error(err: UpdatePersonRecordServiceError) -> UpdatePersonUsecaseError {
    match err {
        UpdatePersonRecordServiceError::NotFound => UpdatePersonUsecaseError::NotFound,
        UpdatePersonRecordServiceError::UpdatePersonRepositoryError(err) => {
            UpdatePersonUsecaseError::TransactionError(match err {
                creation_service::repository::person::UpdatePersonRepositoryError::Db(err) => {
                    TransactionError::Db(err)
                }
                creation_service::repository::person::UpdatePersonRepositoryError::NotFound => {
                    TransactionError::NotFound
                }
            })
        }
        UpdatePersonRecordServiceError::InvalidParams => UpdatePersonUsecaseError::InvalidParams,
    }
}

fn map_update_person_transaction_error(err: TransactionError) -> UpdatePersonUsecaseError {
    match err {
        TransactionError::NotFound => UpdatePersonUsecaseError::NotFound,
        err => UpdatePersonUsecaseError::TransactionError(err),
    }
}

fn map_delete_person_entity_error(err: DeleteEntityServiceError) -> DeletePersonUsecaseError {
    match err {
        DeleteEntityServiceError::NotFound => DeletePersonUsecaseError::NotFound,
        DeleteEntityServiceError::DeleteEntityRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(match err {
                creation_service::repository::entity::DeleteEntityRepositoryError::Db(err) => {
                    TransactionError::Db(err)
                }
                creation_service::repository::entity::DeleteEntityRepositoryError::NotFound => {
                    TransactionError::NotFound
                }
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
                creation_service::repository::entity::LoadSeedEntitiesRepositoryError::Db(err) => {
                    err
                }
            }))
        }
        SyncTreePathsServiceError::LoadActiveEntitiesByDiagramIdsRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                creation_service::repository::entity::LoadActiveEntitiesByDiagramIdsRepositoryError::Db(err) => err,
            }))
        }
        SyncTreePathsServiceError::LoadRelationshipEdgesByDiagramIdsRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                creation_service::repository::relationship::LoadRelationshipEdgesByDiagramIdsRepositoryError::Db(err) => err,
            }))
        }
        SyncTreePathsServiceError::LoadStaleRelatedConnectionsRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                creation_service::repository::tree_path::LoadStaleRelatedConnectionsRepositoryError::Db(err) => err,
            }))
        }
        SyncTreePathsServiceError::DeleteTreePathsByEntityIdsRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                creation_service::repository::tree_path::DeleteTreePathsByEntityIdsRepositoryError::Db(err) => err,
            }))
        }
        SyncTreePathsServiceError::CreateTreePathsRepositoryError(err) => {
            DeletePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                creation_service::repository::tree_path::CreateTreePathsRepositoryError::Db(err) => err,
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
                creation_service::repository::person::DeletePersonRepositoryError::Db(err) => {
                    TransactionError::Db(err)
                }
                creation_service::repository::person::DeletePersonRepositoryError::NotFound => {
                    TransactionError::NotFound
                }
            })
        }
        DeletePersonRecordServiceError::InvalidParams => DeletePersonUsecaseError::InvalidParams,
    }
}

#[async_trait]
pub trait UsesPersonUsecase:
    UsesGetPersonsUsecase + UsesCreatePersonUsecase + UsesUpdatePersonUsecase + UsesDeletePersonUsecase
{
    async fn get_persons(
        &self,
        body: GetPersonsSchema,
    ) -> Result<Vec<Person>, GetPersonsUsecaseError> {
        UsesGetPersonsUsecase::get_persons(self, body).await
    }

    async fn create_person(
        &self,
        body: CreatePersonSchema,
    ) -> Result<(), CreatePersonUsecaseError> {
        UsesCreatePersonUsecase::create_person(self, body).await
    }

    async fn update_person(
        &self,
        body: UpdatePersonSchema,
    ) -> Result<(), UpdatePersonUsecaseError> {
        UsesUpdatePersonUsecase::update_person(self, body).await
    }

    async fn delete_person(
        &self,
        body: DeletePersonSchema,
    ) -> Result<(), DeletePersonUsecaseError> {
        UsesDeletePersonUsecase::delete_person(self, body).await
    }
}

impl<T> UsesPersonUsecase for T where
    T: UsesGetPersonsUsecase
        + UsesCreatePersonUsecase
        + UsesUpdatePersonUsecase
        + UsesDeletePersonUsecase
{
}

pub trait ProvidesPersonUsecase: Send + Sync + 'static {
    type T: UsesPersonUsecase + Sized;
    fn person_usecase(&self) -> &Self::T;
}

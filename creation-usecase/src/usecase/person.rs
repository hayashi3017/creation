use std::collections::HashMap;

use async_trait::async_trait;
use creation_service::{
    model::{
        entity::{
            CreateEntitySchema, DeleteEntitySchema, EntityKind, GetEntitiesSchema,
            UpdateEntitySchema,
        },
        person::{
            CreatePersonRecordSchema, CreatePersonSchema, DeletePersonSchema,
            GetPersonRecordsSchema, GetPersonsSchema, Person, PersonRecord,
            UpdatePersonRecordSchema, UpdatePersonSchema,
        },
    },
    repository::unit_of_work::{
        BeginPersonWriteUnitOfWorkError, PersonWriteUnitOfWork, PersonWriteUnitOfWorkError,
        ProvidesPersonWriteUnitOfWork,
    },
    service::{
        entity::{GetEntitiesServiceError, ProvidesEntityService, UsesEntityService},
        person::{
            prepare_create_person, prepare_delete_person, prepare_update_person,
            GetPersonRecordsServiceError, ProvidesPersonService, UsesPersonService,
        },
    },
};
use thiserror::Error;

#[async_trait]
pub trait PersonUsecase:
    ProvidesEntityService + ProvidesPersonService + ProvidesPersonWriteUnitOfWork
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
    BeginPersonWriteUnitOfWorkError(#[from] BeginPersonWriteUnitOfWorkError),
    #[error(transparent)]
    PersonWriteUnitOfWorkError(#[from] PersonWriteUnitOfWorkError),
}

#[derive(Debug, Error)]
pub enum UpdatePersonUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    BeginPersonWriteUnitOfWorkError(#[from] BeginPersonWriteUnitOfWorkError),
    #[error(transparent)]
    PersonWriteUnitOfWorkError(#[from] PersonWriteUnitOfWorkError),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeletePersonUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    BeginPersonWriteUnitOfWorkError(#[from] BeginPersonWriteUnitOfWorkError),
    #[error(transparent)]
    PersonWriteUnitOfWorkError(#[from] PersonWriteUnitOfWorkError),
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
impl<T: PersonUsecase> UsesGetPersonsUsecase for T {
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
            .map(|entity| entity.id)
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
impl<T: PersonUsecase> UsesCreatePersonUsecase for T {
    async fn create_person(
        &self,
        body: CreatePersonSchema,
    ) -> Result<(), CreatePersonUsecaseError> {
        let body = prepare_create_person(body).ok_or(CreatePersonUsecaseError::InvalidParams)?;

        let mut tx = self.begin_person_write_unit_of_work().await?;

        let entity_id = tx
            .create_entity(CreateEntitySchema {
                diagram_id: body.diagram_id,
                kind: EntityKind::Person,
                name: body.name,
                description: body.description,
            })
            .await?;

        tx.create_person_record(CreatePersonRecordSchema {
            entity_id,
            gender: body.gender,
            birth_date: body.birth_date,
            death_date: body.death_date,
            birthplace: body.birthplace,
            residence: body.residence,
            photo_url: body.photo_url,
        })
        .await?;

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
impl<T: PersonUsecase> UsesUpdatePersonUsecase for T {
    async fn update_person(
        &self,
        body: UpdatePersonSchema,
    ) -> Result<(), UpdatePersonUsecaseError> {
        let body = prepare_update_person(body).ok_or(UpdatePersonUsecaseError::InvalidParams)?;

        let mut tx = self.begin_person_write_unit_of_work().await?;

        tx.update_entity(UpdateEntitySchema {
            id: body.entity_id,
            diagram_id: body.diagram_id,
            kind: EntityKind::Person,
            name: body.name,
            description: body.description,
        })
        .await
        .map_err(map_update_person_write_error)?;

        tx.update_person_record(UpdatePersonRecordSchema {
            entity_id: body.entity_id,
            gender: body.gender,
            birth_date: body.birth_date,
            death_date: body.death_date,
            birthplace: body.birthplace,
            residence: body.residence,
            photo_url: body.photo_url,
        })
        .await
        .map_err(map_update_person_write_error)?;

        tx.commit().await.map_err(map_update_person_write_error)?;

        Ok(())
    }
}

#[async_trait]
pub trait UsesDeletePersonUsecase {
    async fn delete_person(&self, body: DeletePersonSchema)
        -> Result<(), DeletePersonUsecaseError>;
}

#[async_trait]
impl<T: PersonUsecase> UsesDeletePersonUsecase for T {
    async fn delete_person(
        &self,
        body: DeletePersonSchema,
    ) -> Result<(), DeletePersonUsecaseError> {
        let body = prepare_delete_person(body).ok_or(DeletePersonUsecaseError::InvalidParams)?;

        let mut tx = self.begin_person_write_unit_of_work().await?;

        tx.delete_entity(DeleteEntitySchema { id: body.entity_id })
            .await
            .map_err(map_delete_person_write_error)?;

        tx.delete_person_record(body)
            .await
            .map_err(map_delete_person_write_error)?;

        tx.commit().await.map_err(map_delete_person_write_error)?;

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
                .remove(&entity.id)
                .map(|record| Person {
                    entity_id: entity.id,
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

fn map_update_person_write_error(err: PersonWriteUnitOfWorkError) -> UpdatePersonUsecaseError {
    match err {
        PersonWriteUnitOfWorkError::NotFound => UpdatePersonUsecaseError::NotFound,
        err => UpdatePersonUsecaseError::PersonWriteUnitOfWorkError(err),
    }
}

fn map_delete_person_write_error(err: PersonWriteUnitOfWorkError) -> DeletePersonUsecaseError {
    match err {
        PersonWriteUnitOfWorkError::NotFound => DeletePersonUsecaseError::NotFound,
        err => DeletePersonUsecaseError::PersonWriteUnitOfWorkError(err),
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

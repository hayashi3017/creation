use async_trait::async_trait;
use thiserror::Error;

use super::{
    entity::{prepare_create_entity, prepare_delete_entity, prepare_update_entity},
    map_service_result, map_service_result_unit, normalize_optional_text_with_max_chars,
};

use crate::{
    model::{
        entity::{CreateEntitySchema, DeleteEntitySchema, EntityKind, UpdateEntitySchema},
        person::{
            CreatePersonRecordSchema, CreatePersonSchema, DeletePersonSchema,
            GetPersonRecordsSchema, PersonRecord, UpdatePersonRecordSchema, UpdatePersonSchema,
            PERSON_BIRTHPLACE_MAX_CHARS, PERSON_PHOTO_URL_MAX_CHARS, PERSON_RESIDENCE_MAX_CHARS,
        },
    },
    repository::person::{
        CreatePersonRepositoryError, DeletePersonRepositoryError, GetPersonRecordsRepositoryError,
        ProvidesPersonRepository, UpdatePersonRepositoryError, UsesPersonRepository,
    },
};

#[async_trait]
pub trait PersonService: ProvidesPersonRepository {}

#[derive(Debug, Error)]
pub enum PersonServiceError {
    #[error(transparent)]
    GetPersonRecordsServiceError(#[from] GetPersonRecordsServiceError),
    #[error(transparent)]
    CreatePersonRecordServiceError(#[from] CreatePersonRecordServiceError),
    #[error(transparent)]
    UpdatePersonRecordServiceError(#[from] UpdatePersonRecordServiceError),
    #[error(transparent)]
    DeletePersonRecordServiceError(#[from] DeletePersonRecordServiceError),
}

#[derive(Debug, Error)]
pub enum GetPersonRecordsServiceError {
    #[error(transparent)]
    GetPersonRecordsRepositoryError(#[from] GetPersonRecordsRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[derive(Debug, Error)]
pub enum CreatePersonRecordServiceError {
    #[error(transparent)]
    CreatePersonRepositoryError(#[from] CreatePersonRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[derive(Debug, Error)]
pub enum UpdatePersonRecordServiceError {
    #[error(transparent)]
    UpdatePersonRepositoryError(#[from] UpdatePersonRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeletePersonRecordServiceError {
    #[error(transparent)]
    DeletePersonRepositoryError(#[from] DeletePersonRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesPersonService {
    async fn get_person_records(
        &self,
        body: GetPersonRecordsSchema,
    ) -> Result<Vec<PersonRecord>, GetPersonRecordsServiceError>;
    async fn create_person_record(
        &self,
        body: CreatePersonRecordSchema,
    ) -> Result<(), CreatePersonRecordServiceError>;
    async fn update_person_record(
        &self,
        body: UpdatePersonRecordSchema,
    ) -> Result<(), UpdatePersonRecordServiceError>;
    async fn delete_person_record(
        &self,
        body: DeletePersonSchema,
    ) -> Result<(), DeletePersonRecordServiceError>;
}

#[async_trait]
impl<T: PersonService> UsesPersonService for T {
    async fn get_person_records(
        &self,
        body: GetPersonRecordsSchema,
    ) -> Result<Vec<PersonRecord>, GetPersonRecordsServiceError> {
        if body.entity_ids.iter().any(|entity_id| *entity_id == 0) {
            return Err(GetPersonRecordsServiceError::InvalidParams);
        }

        if body.entity_ids.is_empty() {
            return Ok(Vec::new());
        }

        map_service_result!(
            self.person_repository().get_person_records(body),
            GetPersonRecordsServiceError::GetPersonRecordsRepositoryError
        )
    }

    async fn create_person_record(
        &self,
        body: CreatePersonRecordSchema,
    ) -> Result<(), CreatePersonRecordServiceError> {
        let Some(body) = prepare_create_person_record(body) else {
            return Err(CreatePersonRecordServiceError::InvalidParams);
        };

        map_service_result_unit!(
            self.person_repository().create_person_record(body),
            CreatePersonRecordServiceError::CreatePersonRepositoryError
        )
    }

    async fn update_person_record(
        &self,
        body: UpdatePersonRecordSchema,
    ) -> Result<(), UpdatePersonRecordServiceError> {
        let Some(body) = prepare_update_person_record(body) else {
            return Err(UpdatePersonRecordServiceError::InvalidParams);
        };

        match self.person_repository().update_person_record(body).await {
            Ok(()) => Ok(()),
            Err(UpdatePersonRepositoryError::NotFound) => {
                Err(UpdatePersonRecordServiceError::NotFound)
            }
            Err(err) => Err(UpdatePersonRecordServiceError::UpdatePersonRepositoryError(
                err,
            )),
        }
    }

    async fn delete_person_record(
        &self,
        body: DeletePersonSchema,
    ) -> Result<(), DeletePersonRecordServiceError> {
        let Some(body) = prepare_delete_person(body) else {
            return Err(DeletePersonRecordServiceError::InvalidParams);
        };

        match self.person_repository().delete_person_record(body).await {
            Ok(()) => Ok(()),
            Err(DeletePersonRepositoryError::NotFound) => {
                Err(DeletePersonRecordServiceError::NotFound)
            }
            Err(err) => Err(DeletePersonRecordServiceError::DeletePersonRepositoryError(
                err,
            )),
        }
    }
}

pub fn prepare_create_person(body: CreatePersonSchema) -> Option<CreatePersonSchema> {
    let entity = prepare_create_entity(CreateEntitySchema {
        diagram_id: body.diagram_id,
        kind: EntityKind::Person,
        name: body.name,
        description: body.description,
    })?;

    let (birthplace, residence, photo_url) =
        normalize_person_text_fields(body.birthplace, body.residence, body.photo_url)?;

    Some(CreatePersonSchema {
        diagram_id: entity.diagram_id,
        name: entity.name,
        description: entity.description,
        gender: body.gender,
        birth_date: body.birth_date,
        death_date: body.death_date,
        birthplace,
        residence,
        photo_url,
    })
}

pub fn prepare_update_person(body: UpdatePersonSchema) -> Option<UpdatePersonSchema> {
    let entity = prepare_update_entity(UpdateEntitySchema {
        entity_id: body.entity_id,
        diagram_id: body.diagram_id,
        kind: EntityKind::Person,
        name: body.name,
        description: body.description,
    })?;

    let (birthplace, residence, photo_url) =
        normalize_person_text_fields(body.birthplace, body.residence, body.photo_url)?;

    Some(UpdatePersonSchema {
        entity_id: entity.entity_id,
        diagram_id: entity.diagram_id,
        name: entity.name,
        description: entity.description,
        gender: body.gender,
        birth_date: body.birth_date,
        death_date: body.death_date,
        birthplace,
        residence,
        photo_url,
    })
}

pub fn prepare_create_person_record(
    body: CreatePersonRecordSchema,
) -> Option<CreatePersonRecordSchema> {
    if body.entity_id == 0 {
        return None;
    }

    let (birthplace, residence, photo_url) =
        normalize_person_text_fields(body.birthplace, body.residence, body.photo_url)?;

    Some(CreatePersonRecordSchema {
        entity_id: body.entity_id,
        gender: body.gender,
        birth_date: body.birth_date,
        death_date: body.death_date,
        birthplace,
        residence,
        photo_url,
    })
}

pub fn prepare_update_person_record(
    body: UpdatePersonRecordSchema,
) -> Option<UpdatePersonRecordSchema> {
    if body.entity_id == 0 {
        return None;
    }

    let (birthplace, residence, photo_url) =
        normalize_person_text_fields(body.birthplace, body.residence, body.photo_url)?;

    Some(UpdatePersonRecordSchema {
        entity_id: body.entity_id,
        gender: body.gender,
        birth_date: body.birth_date,
        death_date: body.death_date,
        birthplace,
        residence,
        photo_url,
    })
}

pub fn prepare_delete_person(body: DeletePersonSchema) -> Option<DeletePersonSchema> {
    prepare_delete_entity(DeleteEntitySchema {
        entity_id: body.entity_id,
    })
    .map(|entity| DeletePersonSchema {
        entity_id: entity.entity_id,
    })
}

fn normalize_person_text_fields(
    birthplace: Option<String>,
    residence: Option<String>,
    photo_url: Option<String>,
) -> Option<(Option<String>, Option<String>, Option<String>)> {
    let birthplace =
        normalize_optional_text_with_max_chars(birthplace, PERSON_BIRTHPLACE_MAX_CHARS)?;
    let residence = normalize_optional_text_with_max_chars(residence, PERSON_RESIDENCE_MAX_CHARS)?;
    let photo_url = normalize_optional_text_with_max_chars(photo_url, PERSON_PHOTO_URL_MAX_CHARS)?;

    Some((birthplace, residence, photo_url))
}

pub trait ProvidesPersonService: Send + Sync + 'static {
    type T: PersonService;
    fn person_service(&self) -> &Self::T;
}

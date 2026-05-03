use async_trait::async_trait;
use thiserror::Error;

use super::{
    entity::{prepare_create_entity, prepare_delete_entity, prepare_update_entity},
    map_service_result, map_service_result_unit, normalize_optional_text,
    normalize_optional_text_with_max_chars,
};

use crate::{
    model::{
        entity::{CreateEntitySchema, DeleteEntitySchema, EntityKind, UpdateEntitySchema},
        person::{
            CreatePersonRecordSchema, CreatePersonSchema, DeletePersonSchema,
            GetPersonRecordsSchema, PersonRecord, UpdatePersonRecordSchema, UpdatePersonSchema,
            PERSON_BIRTHPLACE_MAX_CHARS, PERSON_DEATHPLACE_MAX_CHARS, PERSON_NAME_PART_MAX_CHARS,
            PERSON_PHOTO_URL_MAX_CHARS, PERSON_RESIDENCE_MAX_CHARS,
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
        world_id: body.world_id,
        kind: EntityKind::Person,
        name: body.name,
        description: body.description,
    })?;

    let text_fields = normalize_person_text_fields(PersonTextFields {
        first_name: body.first_name,
        middle_name: body.middle_name,
        last_name: body.last_name,
        first_name_kana: body.first_name_kana,
        middle_name_kana: body.middle_name_kana,
        last_name_kana: body.last_name_kana,
        first_name_romaji: body.first_name_romaji,
        middle_name_romaji: body.middle_name_romaji,
        last_name_romaji: body.last_name_romaji,
        birthplace: body.birthplace,
        deathplace: body.deathplace,
        residence: body.residence,
        photo_url: body.photo_url,
        profile_text: body.profile_text,
    })?;

    Some(CreatePersonSchema {
        world_id: entity.world_id,
        diagram_id: body.diagram_id,
        name: entity.name,
        description: entity.description,
        first_name: text_fields.first_name,
        middle_name: text_fields.middle_name,
        last_name: text_fields.last_name,
        first_name_kana: text_fields.first_name_kana,
        middle_name_kana: text_fields.middle_name_kana,
        last_name_kana: text_fields.last_name_kana,
        first_name_romaji: text_fields.first_name_romaji,
        middle_name_romaji: text_fields.middle_name_romaji,
        last_name_romaji: text_fields.last_name_romaji,
        gender: body.gender,
        birth_date: body.birth_date,
        death_date: body.death_date,
        birthplace: text_fields.birthplace,
        deathplace: text_fields.deathplace,
        residence: text_fields.residence,
        photo_url: text_fields.photo_url,
        profile_text: text_fields.profile_text,
    })
}

pub fn prepare_update_person(body: UpdatePersonSchema) -> Option<UpdatePersonSchema> {
    let entity = prepare_update_entity(UpdateEntitySchema {
        entity_id: body.entity_id,
        kind: EntityKind::Person,
        name: body.name,
        description: body.description,
    })?;

    let text_fields = normalize_person_text_fields(PersonTextFields {
        first_name: body.first_name,
        middle_name: body.middle_name,
        last_name: body.last_name,
        first_name_kana: body.first_name_kana,
        middle_name_kana: body.middle_name_kana,
        last_name_kana: body.last_name_kana,
        first_name_romaji: body.first_name_romaji,
        middle_name_romaji: body.middle_name_romaji,
        last_name_romaji: body.last_name_romaji,
        birthplace: body.birthplace,
        deathplace: body.deathplace,
        residence: body.residence,
        photo_url: body.photo_url,
        profile_text: body.profile_text,
    })?;

    Some(UpdatePersonSchema {
        entity_id: entity.entity_id,
        name: entity.name,
        description: entity.description,
        first_name: text_fields.first_name,
        middle_name: text_fields.middle_name,
        last_name: text_fields.last_name,
        first_name_kana: text_fields.first_name_kana,
        middle_name_kana: text_fields.middle_name_kana,
        last_name_kana: text_fields.last_name_kana,
        first_name_romaji: text_fields.first_name_romaji,
        middle_name_romaji: text_fields.middle_name_romaji,
        last_name_romaji: text_fields.last_name_romaji,
        gender: body.gender,
        birth_date: body.birth_date,
        death_date: body.death_date,
        birthplace: text_fields.birthplace,
        deathplace: text_fields.deathplace,
        residence: text_fields.residence,
        photo_url: text_fields.photo_url,
        profile_text: text_fields.profile_text,
    })
}

pub fn prepare_create_person_record(
    body: CreatePersonRecordSchema,
) -> Option<CreatePersonRecordSchema> {
    if body.entity_id == 0 {
        return None;
    }

    let text_fields = normalize_person_text_fields(PersonTextFields {
        first_name: body.first_name,
        middle_name: body.middle_name,
        last_name: body.last_name,
        first_name_kana: body.first_name_kana,
        middle_name_kana: body.middle_name_kana,
        last_name_kana: body.last_name_kana,
        first_name_romaji: body.first_name_romaji,
        middle_name_romaji: body.middle_name_romaji,
        last_name_romaji: body.last_name_romaji,
        birthplace: body.birthplace,
        deathplace: body.deathplace,
        residence: body.residence,
        photo_url: body.photo_url,
        profile_text: body.profile_text,
    })?;

    Some(CreatePersonRecordSchema {
        entity_id: body.entity_id,
        first_name: text_fields.first_name,
        middle_name: text_fields.middle_name,
        last_name: text_fields.last_name,
        first_name_kana: text_fields.first_name_kana,
        middle_name_kana: text_fields.middle_name_kana,
        last_name_kana: text_fields.last_name_kana,
        first_name_romaji: text_fields.first_name_romaji,
        middle_name_romaji: text_fields.middle_name_romaji,
        last_name_romaji: text_fields.last_name_romaji,
        gender: body.gender,
        birth_date: body.birth_date,
        death_date: body.death_date,
        birthplace: text_fields.birthplace,
        deathplace: text_fields.deathplace,
        residence: text_fields.residence,
        photo_url: text_fields.photo_url,
        profile_text: text_fields.profile_text,
    })
}

pub fn prepare_update_person_record(
    body: UpdatePersonRecordSchema,
) -> Option<UpdatePersonRecordSchema> {
    if body.entity_id == 0 {
        return None;
    }

    let text_fields = normalize_person_text_fields(PersonTextFields {
        first_name: body.first_name,
        middle_name: body.middle_name,
        last_name: body.last_name,
        first_name_kana: body.first_name_kana,
        middle_name_kana: body.middle_name_kana,
        last_name_kana: body.last_name_kana,
        first_name_romaji: body.first_name_romaji,
        middle_name_romaji: body.middle_name_romaji,
        last_name_romaji: body.last_name_romaji,
        birthplace: body.birthplace,
        deathplace: body.deathplace,
        residence: body.residence,
        photo_url: body.photo_url,
        profile_text: body.profile_text,
    })?;

    Some(UpdatePersonRecordSchema {
        entity_id: body.entity_id,
        first_name: text_fields.first_name,
        middle_name: text_fields.middle_name,
        last_name: text_fields.last_name,
        first_name_kana: text_fields.first_name_kana,
        middle_name_kana: text_fields.middle_name_kana,
        last_name_kana: text_fields.last_name_kana,
        first_name_romaji: text_fields.first_name_romaji,
        middle_name_romaji: text_fields.middle_name_romaji,
        last_name_romaji: text_fields.last_name_romaji,
        gender: body.gender,
        birth_date: body.birth_date,
        death_date: body.death_date,
        birthplace: text_fields.birthplace,
        deathplace: text_fields.deathplace,
        residence: text_fields.residence,
        photo_url: text_fields.photo_url,
        profile_text: text_fields.profile_text,
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

#[derive(Debug)]
struct PersonTextFields {
    first_name: Option<String>,
    middle_name: Option<String>,
    last_name: Option<String>,
    first_name_kana: Option<String>,
    middle_name_kana: Option<String>,
    last_name_kana: Option<String>,
    first_name_romaji: Option<String>,
    middle_name_romaji: Option<String>,
    last_name_romaji: Option<String>,
    birthplace: Option<String>,
    deathplace: Option<String>,
    residence: Option<String>,
    photo_url: Option<String>,
    profile_text: Option<String>,
}

fn normalize_person_text_fields(fields: PersonTextFields) -> Option<PersonTextFields> {
    Some(PersonTextFields {
        first_name: normalize_optional_text_with_max_chars(
            fields.first_name,
            PERSON_NAME_PART_MAX_CHARS,
        )?,
        middle_name: normalize_optional_text_with_max_chars(
            fields.middle_name,
            PERSON_NAME_PART_MAX_CHARS,
        )?,
        last_name: normalize_optional_text_with_max_chars(
            fields.last_name,
            PERSON_NAME_PART_MAX_CHARS,
        )?,
        first_name_kana: normalize_optional_text_with_max_chars(
            fields.first_name_kana,
            PERSON_NAME_PART_MAX_CHARS,
        )?,
        middle_name_kana: normalize_optional_text_with_max_chars(
            fields.middle_name_kana,
            PERSON_NAME_PART_MAX_CHARS,
        )?,
        last_name_kana: normalize_optional_text_with_max_chars(
            fields.last_name_kana,
            PERSON_NAME_PART_MAX_CHARS,
        )?,
        first_name_romaji: normalize_optional_text_with_max_chars(
            fields.first_name_romaji,
            PERSON_NAME_PART_MAX_CHARS,
        )?,
        middle_name_romaji: normalize_optional_text_with_max_chars(
            fields.middle_name_romaji,
            PERSON_NAME_PART_MAX_CHARS,
        )?,
        last_name_romaji: normalize_optional_text_with_max_chars(
            fields.last_name_romaji,
            PERSON_NAME_PART_MAX_CHARS,
        )?,
        birthplace: normalize_optional_text_with_max_chars(
            fields.birthplace,
            PERSON_BIRTHPLACE_MAX_CHARS,
        )?,
        deathplace: normalize_optional_text_with_max_chars(
            fields.deathplace,
            PERSON_DEATHPLACE_MAX_CHARS,
        )?,
        residence: normalize_optional_text_with_max_chars(
            fields.residence,
            PERSON_RESIDENCE_MAX_CHARS,
        )?,
        photo_url: normalize_optional_text_with_max_chars(
            fields.photo_url,
            PERSON_PHOTO_URL_MAX_CHARS,
        )?,
        profile_text: normalize_optional_text(fields.profile_text),
    })
}

pub trait ProvidesPersonService: Send + Sync + 'static {
    type T: PersonService;
    fn person_service(&self) -> &Self::T;
}

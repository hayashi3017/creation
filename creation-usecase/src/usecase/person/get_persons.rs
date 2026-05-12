use async_trait::async_trait;
use creation_service::{
    model::{
        entity::{EntityKind, GetEntitiesSchema},
        person::{GetPersonRecordsSchema, GetPersonsSchema, Person},
    },
    service::{
        entity::{GetEntitiesServiceError, ProvidesEntityService, UsesEntityService},
        person::{GetPersonRecordsServiceError, ProvidesPersonService, UsesPersonService},
        transaction::{ProvidesTransactionManager, TransactionContext},
    },
};
use std::collections::HashMap;
use thiserror::Error;

use super::PersonUsecase;

#[derive(Debug, Error)]
pub enum GetPersonsUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    GetEntitiesServiceError(#[from] GetEntitiesServiceError),
    #[error(transparent)]
    GetPersonRecordsServiceError(#[from] GetPersonRecordsServiceError),
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
    <T as ProvidesTransactionManager>::T: ProvidesEntityService + ProvidesPersonService,
{
    async fn get_persons(
        &self,
        body: GetPersonsSchema,
    ) -> Result<Vec<Person>, GetPersonsUsecaseError> {
        if body.world_id == 0 || body.diagram_id == 0 {
            return Err(GetPersonsUsecaseError::InvalidParams);
        }

        let entities = match self
            .entity_service()
            .get_entities(GetEntitiesSchema {
                world_id: body.world_id,
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

fn merge_persons(
    person_entities: Vec<creation_service::model::entity::Entity>,
    person_records: Vec<creation_service::model::person::PersonRecord>,
) -> Vec<Person> {
    let mut records_by_entity_id: HashMap<usize, creation_service::model::person::PersonRecord> =
        person_records
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
                    first_name: record.first_name,
                    middle_name: record.middle_name,
                    last_name: record.last_name,
                    first_name_kana: record.first_name_kana,
                    middle_name_kana: record.middle_name_kana,
                    last_name_kana: record.last_name_kana,
                    first_name_romaji: record.first_name_romaji,
                    middle_name_romaji: record.middle_name_romaji,
                    last_name_romaji: record.last_name_romaji,
                    gender: record.gender,
                    birth_date: record.birth_date,
                    death_date: record.death_date,
                    birthplace: record.birthplace,
                    deathplace: record.deathplace,
                    residence: record.residence,
                    photo_url: record.photo_url,
                    profile_text: record.profile_text,
                })
        })
        .collect()
}

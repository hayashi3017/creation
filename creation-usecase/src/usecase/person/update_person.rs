use async_trait::async_trait;
use creation_service::repository::person::UpdatePersonRepositoryError;
use creation_service::{
    model::entity::{EntityKind, UpdateEntitySchema},
    model::person::UpdatePersonRecordSchema,
    model::person::UpdatePersonSchema,
    repository::entity::UpdateEntityRepositoryError,
    service::entity::{ProvidesEntityService, UpdateEntityServiceError, UsesEntityService},
    service::person::{
        prepare_update_person, ProvidesPersonService, UpdatePersonRecordServiceError,
        UsesPersonService,
    },
    service::transaction::{ProvidesTransactionManager, TransactionContext, TransactionError},
};
use thiserror::Error;

use super::PersonUsecase;

#[derive(Debug, Error)]
pub enum UpdatePersonUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    BeginTransactionError(#[from] creation_service::service::transaction::BeginTransactionError),
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    #[error("not found")]
    NotFound,
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
    <T as ProvidesTransactionManager>::T: ProvidesEntityService + ProvidesPersonService,
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
                world_id: body.world_id,
                kind: EntityKind::Person,
                name: body.name,
                description: body.description,
            })
            .await
            .map_err(map_update_person_entity_error)?;

        tx.person_service()
            .update_person_record(UpdatePersonRecordSchema {
                entity_id: body.entity_id,
                first_name: body.first_name,
                middle_name: body.middle_name,
                last_name: body.last_name,
                first_name_kana: body.first_name_kana,
                middle_name_kana: body.middle_name_kana,
                last_name_kana: body.last_name_kana,
                first_name_romaji: body.first_name_romaji,
                middle_name_romaji: body.middle_name_romaji,
                last_name_romaji: body.last_name_romaji,
                gender: body.gender,
                birth_date: body.birth_date,
                death_date: body.death_date,
                birthplace: body.birthplace,
                deathplace: body.deathplace,
                residence: body.residence,
                photo_url: body.photo_url,
                profile_text: body.profile_text,
            })
            .await
            .map_err(map_update_person_record_error)?;

        tx.commit()
            .await
            .map_err(map_update_person_transaction_error)?;

        Ok(())
    }
}

fn map_update_person_entity_error(err: UpdateEntityServiceError) -> UpdatePersonUsecaseError {
    match err {
        UpdateEntityServiceError::NotFound => UpdatePersonUsecaseError::NotFound,
        UpdateEntityServiceError::UpdateEntityRepositoryError(err) => {
            UpdatePersonUsecaseError::TransactionError(match err {
                UpdateEntityRepositoryError::Db(err) => TransactionError::Db(err),
                UpdateEntityRepositoryError::NotFound => TransactionError::NotFound,
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
                UpdatePersonRepositoryError::Db(err) => TransactionError::Db(err),
                UpdatePersonRepositoryError::NotFound => TransactionError::NotFound,
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

use async_trait::async_trait;
use creation_service::repository::person::CreatePersonRepositoryError;
use creation_service::service::transaction::TransactionError;
use creation_service::{
    model::entity::{CreateDiagramEntityMembershipSchema, CreateEntitySchema, EntityKind},
    model::person::CreatePersonRecordSchema,
    model::person::CreatePersonSchema,
    repository::entity::{
        CreateDiagramEntityMembershipRepositoryError, CreateEntityRepositoryError,
    },
    service::entity::{
        CreateDiagramEntityMembershipServiceError, CreateEntityServiceError, ProvidesEntityService,
        UsesEntityService,
    },
    service::person::{
        prepare_create_person, CreatePersonRecordServiceError, ProvidesPersonService,
        UsesPersonService,
    },
    service::transaction::{BeginTransactionError, ProvidesTransactionManager, TransactionContext},
};
use thiserror::Error;

use super::PersonUsecase;

#[derive(Debug, Error)]
pub enum CreatePersonUsecaseError {
    #[error("invalid parameter")]
    InvalidParams,
    #[error(transparent)]
    BeginTransactionError(#[from] BeginTransactionError),
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
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
    <T as ProvidesTransactionManager>::T: ProvidesEntityService + ProvidesPersonService,
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
            .map_err(map_create_person_record_error)?;

        tx.commit().await?;

        Ok(())
    }
}

fn map_create_person_entity_error(err: CreateEntityServiceError) -> CreatePersonUsecaseError {
    match err {
        CreateEntityServiceError::CreateEntityRepositoryError(err) => {
            CreatePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                CreateEntityRepositoryError::Db(err) => err,
                CreateEntityRepositoryError::NotFound => sqlx::Error::RowNotFound,
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
            CreateDiagramEntityMembershipRepositoryError::Db(err) => err,
            CreateDiagramEntityMembershipRepositoryError::NotFound => sqlx::Error::RowNotFound,
        })),
    }
}

fn map_create_person_record_error(err: CreatePersonRecordServiceError) -> CreatePersonUsecaseError {
    match err {
        CreatePersonRecordServiceError::CreatePersonRepositoryError(err) => {
            CreatePersonUsecaseError::TransactionError(TransactionError::Db(match err {
                CreatePersonRepositoryError::Db(err) => err,
            }))
        }
        CreatePersonRecordServiceError::InvalidParams => CreatePersonUsecaseError::InvalidParams,
    }
}

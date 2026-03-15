use async_trait::async_trait;
use thiserror::Error;

use crate::model::person::{
    CreatePersonRecordSchema, DeletePersonSchema, GetPersonRecordsSchema, PersonRecord,
    UpdatePersonRecordSchema,
};

pub trait PersonRepository: Send + Sync + 'static {}

#[derive(Debug, Error)]
pub enum PersonRepositoryError {
    #[error(transparent)]
    GetPersonRecordsRepositoryError(#[from] GetPersonRecordsRepositoryError),
    #[error(transparent)]
    CreatePersonRepositoryError(#[from] CreatePersonRepositoryError),
    #[error(transparent)]
    UpdatePersonRepositoryError(#[from] UpdatePersonRepositoryError),
    #[error(transparent)]
    DeletePersonRepositoryError(#[from] DeletePersonRepositoryError),
}

#[derive(Debug, Error)]
pub enum GetPersonRecordsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum CreatePersonRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum UpdatePersonRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeletePersonRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesPersonRepository: Send + Sync + 'static {
    async fn get_person_records(
        &self,
        body: GetPersonRecordsSchema,
    ) -> Result<Vec<PersonRecord>, GetPersonRecordsRepositoryError>;
    async fn create_person_record(
        &self,
        body: CreatePersonRecordSchema,
    ) -> Result<(), CreatePersonRepositoryError>;
    async fn update_person_record(
        &self,
        body: UpdatePersonRecordSchema,
    ) -> Result<(), UpdatePersonRepositoryError>;
    async fn delete_person_record(
        &self,
        body: DeletePersonSchema,
    ) -> Result<(), DeletePersonRepositoryError>;
}

pub trait ProvidesPersonRepository: Send + Sync + 'static {
    type T: UsesPersonRepository;
    fn person_repository(&self) -> &Self::T;
}

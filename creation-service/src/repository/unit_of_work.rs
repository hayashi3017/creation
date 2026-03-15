use async_trait::async_trait;
use thiserror::Error;

use crate::model::{
    entity::{CreateEntitySchema, DeleteEntitySchema, UpdateEntitySchema},
    person::{CreatePersonRecordSchema, DeletePersonSchema, UpdatePersonRecordSchema},
};

#[derive(Debug, Error)]
pub enum BeginPersonWriteUnitOfWorkError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum PersonWriteUnitOfWorkError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait PersonWriteUnitOfWork: Send {
    async fn create_entity(
        &mut self,
        body: CreateEntitySchema,
    ) -> Result<usize, PersonWriteUnitOfWorkError>;
    async fn create_person_record(
        &mut self,
        body: CreatePersonRecordSchema,
    ) -> Result<(), PersonWriteUnitOfWorkError>;
    async fn update_entity(
        &mut self,
        body: UpdateEntitySchema,
    ) -> Result<(), PersonWriteUnitOfWorkError>;
    async fn update_person_record(
        &mut self,
        body: UpdatePersonRecordSchema,
    ) -> Result<(), PersonWriteUnitOfWorkError>;
    async fn delete_entity(
        &mut self,
        body: DeleteEntitySchema,
    ) -> Result<(), PersonWriteUnitOfWorkError>;
    async fn delete_person_record(
        &mut self,
        body: DeletePersonSchema,
    ) -> Result<(), PersonWriteUnitOfWorkError>;
    async fn commit(self) -> Result<(), PersonWriteUnitOfWorkError>
    where
        Self: Sized;
}

#[async_trait]
pub trait ProvidesPersonWriteUnitOfWork: Send + Sync + 'static {
    type T: PersonWriteUnitOfWork;

    async fn begin_person_write_unit_of_work(
        &self,
    ) -> Result<Self::T, BeginPersonWriteUnitOfWorkError>;
}

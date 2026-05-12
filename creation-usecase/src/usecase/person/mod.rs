pub mod create_person;
pub mod delete_person;
pub mod get_persons;
pub mod update_person;

use async_trait::async_trait;
use creation_service::service::{
    entity::ProvidesEntityService,
    person::ProvidesPersonService,
    transaction::ProvidesTransactionManager,
};
use thiserror::Error;

pub use create_person::{CreatePersonUsecaseError, UsesCreatePersonUsecase};
pub use delete_person::{DeletePersonUsecaseError, UsesDeletePersonUsecase};
pub use get_persons::{GetPersonsUsecaseError, UsesGetPersonsUsecase};
pub use update_person::{UpdatePersonUsecaseError, UsesUpdatePersonUsecase};

#[async_trait]
pub trait PersonUsecase:
    ProvidesEntityService + ProvidesPersonService + ProvidesTransactionManager
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

#[async_trait]
pub trait UsesPersonUsecase:
    UsesGetPersonsUsecase + UsesCreatePersonUsecase + UsesUpdatePersonUsecase + UsesDeletePersonUsecase
{
    async fn get_persons(
        &self,
        body: creation_service::model::person::GetPersonsSchema,
    ) -> Result<Vec<creation_service::model::person::Person>, GetPersonsUsecaseError> {
        UsesGetPersonsUsecase::get_persons(self, body).await
    }

    async fn create_person(
        &self,
        body: creation_service::model::person::CreatePersonSchema,
    ) -> Result<(), CreatePersonUsecaseError> {
        UsesCreatePersonUsecase::create_person(self, body).await
    }

    async fn update_person(
        &self,
        body: creation_service::model::person::UpdatePersonSchema,
    ) -> Result<(), UpdatePersonUsecaseError> {
        UsesUpdatePersonUsecase::update_person(self, body).await
    }

    async fn delete_person(
        &self,
        body: creation_service::model::person::DeletePersonSchema,
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

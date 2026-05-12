pub mod create_diagram;
pub mod delete_diagram;
pub mod get_diagrams;
pub mod update_diagram;

use async_trait::async_trait;
use creation_service::service::{
    diagram::ProvidesDiagramService,
    entity::ProvidesEntityService,
    relationship::ProvidesRelationshipService,
    transaction::ProvidesTransactionManager,
    tree_path::ProvidesTreePathService,
};
use thiserror::Error;

pub use create_diagram::{CreateDiagramUsecaseError, UsesCreateDiagramUsecase};
pub use delete_diagram::{DeleteDiagramUsecaseError, UsesDeleteDiagramUsecase};
pub use get_diagrams::{GetDiagramsUsecaseError, UsesGetDiagramsUsecase};
pub use update_diagram::{UpdateDiagramUsecaseError, UsesUpdateDiagramUsecase};

#[async_trait]
pub trait DiagramUsecase: ProvidesDiagramService {}

#[derive(Debug, Error)]
pub enum DiagramUsecaseError {
    #[error(transparent)]
    GetDiagramsUsecaseError(#[from] GetDiagramsUsecaseError),
    #[error(transparent)]
    CreateDiagramUsecaseError(#[from] CreateDiagramUsecaseError),
    #[error(transparent)]
    UpdateDiagramUsecaseError(#[from] UpdateDiagramUsecaseError),
    #[error(transparent)]
    DeleteDiagramUsecaseError(#[from] DeleteDiagramUsecaseError),
}

#[async_trait]
pub trait UsesDiagramUsecase:
    UsesGetDiagramsUsecase
    + UsesCreateDiagramUsecase
    + UsesUpdateDiagramUsecase
    + UsesDeleteDiagramUsecase
{
    async fn get_diagrams(
        &self,
        body: creation_service::model::diagram::GetDiagramsSchema,
    ) -> Result<Vec<creation_service::model::diagram::Diagram>, GetDiagramsUsecaseError> {
        UsesGetDiagramsUsecase::get_diagrams(self, body).await
    }

    async fn create_diagram(
        &self,
        body: creation_service::model::diagram::CreateDiagramSchema,
    ) -> Result<(), CreateDiagramUsecaseError> {
        UsesCreateDiagramUsecase::create_diagram(self, body).await
    }

    async fn update_diagram(
        &self,
        body: creation_service::model::diagram::UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramUsecaseError> {
        UsesUpdateDiagramUsecase::update_diagram(self, body).await
    }

    async fn delete_diagram(
        &self,
        body: creation_service::model::diagram::DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramUsecaseError> {
        UsesDeleteDiagramUsecase::delete_diagram(self, body).await
    }
}

impl<T> UsesDiagramUsecase for T where
    T: UsesGetDiagramsUsecase
        + UsesCreateDiagramUsecase
        + UsesUpdateDiagramUsecase
        + UsesDeleteDiagramUsecase
{
}

pub trait ProvidesDiagramUsecase: Send + Sync + 'static {
    type T: UsesDiagramUsecase + Sized;
    fn diagram_usecase(&self) -> &Self::T;
}

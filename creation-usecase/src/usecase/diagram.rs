use async_trait::async_trait;
use creation_service::{
    model::diagram::{
        CreateDiagramSchema, DeleteDiagramSchema, Diagram, GetDiagramsSchema, UpdateDiagramSchema,
    },
    service::diagram::{
        CreateDiagramServiceError, DeleteDiagramServiceError, GetDiagramsServiceError,
        ProvidesDiagramService, UpdateDiagramServiceError, UsesDiagramService,
    },
};
use thiserror::Error;

use super::{map_usecase_result, map_usecase_result_unit};

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

#[derive(Debug, Error)]
pub enum GetDiagramsUsecaseError {
    #[error(transparent)]
    GetDiagramsServiceError(#[from] GetDiagramsServiceError),
}

#[derive(Debug, Error)]
pub enum CreateDiagramUsecaseError {
    #[error(transparent)]
    CreateDiagramServiceError(#[from] CreateDiagramServiceError),
}

#[derive(Debug, Error)]
pub enum UpdateDiagramUsecaseError {
    #[error(transparent)]
    UpdateDiagramServiceError(#[from] UpdateDiagramServiceError),
}

#[derive(Debug, Error)]
pub enum DeleteDiagramUsecaseError {
    #[error(transparent)]
    DeleteDiagramServiceError(#[from] DeleteDiagramServiceError),
}

#[async_trait]
pub trait UsesGetDiagramsUsecase {
    async fn get_diagrams(
        &self,
        body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsUsecaseError>;
}

#[async_trait]
impl<T: DiagramUsecase> UsesGetDiagramsUsecase for T {
    async fn get_diagrams(
        &self,
        body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsUsecaseError> {
        map_usecase_result!(
            self.diagram_service().get_diagrams(body),
            GetDiagramsUsecaseError::GetDiagramsServiceError
        )
    }
}

#[async_trait]
pub trait UsesCreateDiagramUsecase {
    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramUsecaseError>;
}

#[async_trait]
impl<T: DiagramUsecase> UsesCreateDiagramUsecase for T {
    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramUsecaseError> {
        map_usecase_result_unit!(
            self.diagram_service().create_diagram(body),
            CreateDiagramUsecaseError::CreateDiagramServiceError
        )
    }
}

#[async_trait]
pub trait UsesUpdateDiagramUsecase {
    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramUsecaseError>;
}

#[async_trait]
impl<T: DiagramUsecase> UsesUpdateDiagramUsecase for T {
    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramUsecaseError> {
        map_usecase_result_unit!(
            self.diagram_service().update_diagram(body),
            UpdateDiagramUsecaseError::UpdateDiagramServiceError
        )
    }
}

#[async_trait]
pub trait UsesDeleteDiagramUsecase {
    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramUsecaseError>;
}

#[async_trait]
impl<T: DiagramUsecase> UsesDeleteDiagramUsecase for T {
    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramUsecaseError> {
        map_usecase_result_unit!(
            self.diagram_service().delete_diagram(body),
            DeleteDiagramUsecaseError::DeleteDiagramServiceError
        )
    }
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
        body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsUsecaseError> {
        UsesGetDiagramsUsecase::get_diagrams(self, body).await
    }

    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramUsecaseError> {
        UsesCreateDiagramUsecase::create_diagram(self, body).await
    }

    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramUsecaseError> {
        UsesUpdateDiagramUsecase::update_diagram(self, body).await
    }

    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
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

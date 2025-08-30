use async_trait::async_trait;
use thiserror::Error;

use crate::{
    model::diagram::{
        CreateDiagramSchema, DeleteDiagramSchema, Diagram, GetDiagramsSchema, UpdateDiagramSchema,
    },
    repository::diagram::{
        CreateDiagramRepositoryError, DeleteDiagramRepositoryError, GetDiagramsRepositoryError,
        ProvidesDiagramRepository, UpdateDiagramRepositoryError, UsesDiagramRepository,
    },
};

#[async_trait]
pub trait DiagramService: ProvidesDiagramRepository {}

#[derive(Debug, Error)]
pub enum DiagramServiceError {
    #[error(transparent)]
    GetDiagramsServiceError(#[from] GetDiagramsServiceError),
    #[error(transparent)]
    CreateDiagramServiceError(#[from] CreateDiagramServiceError),
    #[error(transparent)]
    UpdateDiagramServiceError(#[from] UpdateDiagramServiceError),
    #[error(transparent)]
    DeleteDiagramServiceError(#[from] DeleteDiagramServiceError),
}

#[derive(Debug, Error)]
pub enum GetDiagramsServiceError {
    #[error(transparent)]
    GetDiagramsRepositoryError(#[from] GetDiagramsRepositoryError),
}

#[derive(Debug, Error)]
pub enum CreateDiagramServiceError {
    #[error(transparent)]
    CreateDiagramRepositoryError(#[from] CreateDiagramRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("duplicate diagram")]
    DuplicateDiagram,
}

#[derive(Debug, Error)]
pub enum UpdateDiagramServiceError {
    #[error(transparent)]
    UpdateDiagramRepositoryError(#[from] UpdateDiagramRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[derive(Debug, Error)]
pub enum DeleteDiagramServiceError {
    #[error(transparent)]
    DeleteDiagramRepositoryError(#[from] DeleteDiagramRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
}

#[async_trait]
pub trait UsesDiagramService {
    async fn get_diagrams(
        &self,
        body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsServiceError>;
    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramServiceError>;
    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramServiceError>;
    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramServiceError>;
}

#[async_trait]
impl<T: DiagramService> UsesDiagramService for T {
    async fn get_diagrams(
        &self,
        body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsServiceError> {
        match self.diagram_repository().get_diagrams(body).await {
            Err(err) => Err(GetDiagramsServiceError::GetDiagramsRepositoryError(err)),
            Ok(diagmras) => Ok(diagmras),
        }
    }

    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramServiceError> {
        if body.name.is_empty() {
            return Err(CreateDiagramServiceError::InvalidParams);
        }

        match self.diagram_repository().create_diagram(body).await {
            Err(err) => Err(CreateDiagramServiceError::CreateDiagramRepositoryError(err)),
            Ok(_) => Ok(()),
        }
    }

    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramServiceError> {
        match self.diagram_repository().update_diagram(body).await {
            Err(err) => Err(UpdateDiagramServiceError::UpdateDiagramRepositoryError(err)),
            Ok(_) => Ok(()),
        }
    }

    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramServiceError> {
        match self.diagram_repository().delete_diagram(body).await {
            Err(err) => Err(DeleteDiagramServiceError::DeleteDiagramRepositoryError(err)),
            Ok(_) => Ok(()),
        }
    }
}

pub trait ProvidesDiagramService: Send + Sync + 'static {
    type T: DiagramService;
    fn diagram_service(&self) -> &Self::T;
}

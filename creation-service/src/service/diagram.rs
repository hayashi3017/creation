use async_trait::async_trait;
use thiserror::Error;

use super::{map_service_result, map_service_result_unit, normalize_name, normalize_optional_text};

use crate::{
    model::diagram::{
        CreateDiagramSchema, DeleteDiagramSchema, Diagram, GetDiagramsSchema, UpdateDiagramSchema,
        DIAGRAM_NAME_MAX_CHARS,
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
    #[error("invalid parameter")]
    InvalidParams,
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
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteDiagramServiceError {
    #[error(transparent)]
    DeleteDiagramRepositoryError(#[from] DeleteDiagramRepositoryError),
    #[error("invalid parameter")]
    InvalidParams,
    #[error("not found")]
    NotFound,
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
        if body.world_id == 0 {
            return Err(GetDiagramsServiceError::InvalidParams);
        }

        map_service_result!(
            self.diagram_repository().get_diagrams(body),
            GetDiagramsServiceError::GetDiagramsRepositoryError
        )
    }

    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramServiceError> {
        let mut body = body;
        if body.world_id == 0 {
            return Err(CreateDiagramServiceError::InvalidParams);
        }

        let Some(name) = normalize_name(&body.name, DIAGRAM_NAME_MAX_CHARS) else {
            return Err(CreateDiagramServiceError::InvalidParams);
        };

        body.name = name;
        body.description = normalize_optional_text(body.description);

        map_service_result_unit!(
            self.diagram_repository().create_diagram(body),
            CreateDiagramServiceError::CreateDiagramRepositoryError
        )
    }

    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramServiceError> {
        let mut body = body;

        if body.diagram_id == 0 || body.world_id == 0 {
            return Err(UpdateDiagramServiceError::InvalidParams);
        }

        let Some(name) = normalize_name(&body.name, DIAGRAM_NAME_MAX_CHARS) else {
            return Err(UpdateDiagramServiceError::InvalidParams);
        };

        body.name = name;
        body.description = normalize_optional_text(body.description);

        match self.diagram_repository().update_diagram(body).await {
            Ok(()) => Ok(()),
            Err(UpdateDiagramRepositoryError::NotFound) => Err(UpdateDiagramServiceError::NotFound),
            Err(err) => Err(UpdateDiagramServiceError::UpdateDiagramRepositoryError(err)),
        }
    }

    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramServiceError> {
        if body.diagram_id == 0 || body.world_id == 0 {
            return Err(DeleteDiagramServiceError::InvalidParams);
        }

        match self.diagram_repository().delete_diagram(body).await {
            Ok(()) => Ok(()),
            Err(DeleteDiagramRepositoryError::NotFound) => Err(DeleteDiagramServiceError::NotFound),
            Err(err) => Err(DeleteDiagramServiceError::DeleteDiagramRepositoryError(err)),
        }
    }
}

pub trait ProvidesDiagramService: Send + Sync + 'static {
    type T: DiagramService;
    fn diagram_service(&self) -> &Self::T;
}

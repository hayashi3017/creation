use async_trait::async_trait;
use thiserror::Error;

use crate::model::diagram::{
    CreateDiagramSchema, DeleteDiagramSchema, Diagram, GetDiagramsSchema, UpdateDiagramSchema,
};

pub trait DiagramRepository: Send + Sync + 'static {}

#[derive(Debug, Error)]
pub enum DiagramRepositoryError {
    #[error(transparent)]
    GetDiagramsRepositoryError(#[from] GetDiagramsRepositoryError),
    #[error(transparent)]
    CreateDiagramRepositoryError(#[from] CreateDiagramRepositoryError),
    #[error(transparent)]
    UpdateDiagramRepositoryError(#[from] UpdateDiagramRepositoryError),
    #[error(transparent)]
    DeleteDiagramRepositoryError(#[from] DeleteDiagramRepositoryError),
}

#[derive(Debug, Error)]
pub enum GetDiagramsRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum CreateDiagramRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum UpdateDiagramRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteDiagramRepositoryError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait UsesDiagramRepository: Send + Sync + 'static {
    async fn get_diagrams(
        &self,
        body: GetDiagramsSchema,
    ) -> Result<Vec<Diagram>, GetDiagramsRepositoryError>;
    async fn create_diagram(
        &self,
        body: CreateDiagramSchema,
    ) -> Result<(), CreateDiagramRepositoryError>;
    async fn update_diagram(
        &self,
        body: UpdateDiagramSchema,
    ) -> Result<(), UpdateDiagramRepositoryError>;
    async fn delete_diagram(
        &self,
        body: DeleteDiagramSchema,
    ) -> Result<(), DeleteDiagramRepositoryError>;
}

pub trait ProvidesDiagramRepository: Send + Sync + 'static {
    type T: UsesDiagramRepository;
    fn diagram_repository(&self) -> &Self::T;
}

use async_trait::async_trait;
use creation_service::model::diagram::CreateDiagramSchema;
use creation_service::service::diagram::{CreateDiagramServiceError, UsesDiagramService};
use thiserror::Error;

use super::DiagramUsecase;

#[derive(Debug, Error)]
pub enum CreateDiagramUsecaseError {
    #[error(transparent)]
    CreateDiagramServiceError(#[from] CreateDiagramServiceError),
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
        self.diagram_service()
            .create_diagram(body)
            .await
            .map_err(CreateDiagramUsecaseError::CreateDiagramServiceError)
    }
}

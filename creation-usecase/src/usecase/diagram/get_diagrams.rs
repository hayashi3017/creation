use async_trait::async_trait;
use creation_service::model::diagram::{Diagram, GetDiagramsSchema};
use creation_service::service::diagram::{GetDiagramsServiceError, UsesDiagramService};
use thiserror::Error;

use super::DiagramUsecase;

#[derive(Debug, Error)]
pub enum GetDiagramsUsecaseError {
    #[error(transparent)]
    GetDiagramsServiceError(#[from] GetDiagramsServiceError),
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
        self.diagram_service()
            .get_diagrams(body)
            .await
            .map_err(GetDiagramsUsecaseError::GetDiagramsServiceError)
    }
}

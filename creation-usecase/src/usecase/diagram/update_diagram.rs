use async_trait::async_trait;
use creation_service::model::diagram::UpdateDiagramSchema;
use creation_service::service::diagram::{UpdateDiagramServiceError, UsesDiagramService};
use thiserror::Error;

use super::DiagramUsecase;

#[derive(Debug, Error)]
pub enum UpdateDiagramUsecaseError {
    #[error(transparent)]
    UpdateDiagramServiceError(#[from] UpdateDiagramServiceError),
    #[error("not found")]
    NotFound,
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
        match self.diagram_service().update_diagram(body).await {
            Ok(()) => Ok(()),
            Err(UpdateDiagramServiceError::NotFound) => Err(UpdateDiagramUsecaseError::NotFound),
            Err(err) => Err(UpdateDiagramUsecaseError::UpdateDiagramServiceError(err)),
        }
    }
}

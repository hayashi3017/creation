pub mod get_genealogy_diagram;

use async_trait::async_trait;
use creation_service::{
    repository::diagram::ProvidesDiagramRepository,
    service::{
        entity::ProvidesEntityService, kinship_derivation::ProvidesKinshipDerivationService,
        person::ProvidesPersonService, relationship::ProvidesRelationshipService,
    },
};
use thiserror::Error;

pub use get_genealogy_diagram::{GetGenealogyDiagramUsecaseError, UsesGetGenealogyDiagramUsecase};

#[async_trait]
pub trait GenealogyDiagramUsecase:
    ProvidesDiagramRepository
    + ProvidesEntityService
    + ProvidesPersonService
    + ProvidesRelationshipService
    + ProvidesKinshipDerivationService
{
}

#[derive(Debug, Error)]
pub enum GenealogyDiagramUsecaseError {
    #[error(transparent)]
    GetGenealogyDiagramUsecaseError(#[from] GetGenealogyDiagramUsecaseError),
}

#[async_trait]
pub trait UsesGenealogyDiagramUsecase: UsesGetGenealogyDiagramUsecase {
    async fn get_genealogy_diagram(
        &self,
        body: creation_service::model::genealogy_diagram::GetGenealogyDiagramSchema,
    ) -> Result<
        creation_service::model::genealogy_diagram::GenealogyDiagramGraph,
        GetGenealogyDiagramUsecaseError,
    > {
        UsesGetGenealogyDiagramUsecase::get_genealogy_diagram(self, body).await
    }
}

impl<T> UsesGenealogyDiagramUsecase for T where T: UsesGetGenealogyDiagramUsecase {}

pub trait ProvidesGenealogyDiagramUsecase: Send + Sync + 'static {
    type T: UsesGenealogyDiagramUsecase + Sized;
    fn genealogy_diagram_usecase(&self) -> &Self::T;
}

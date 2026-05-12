pub mod get_genealogy_overview;

use async_trait::async_trait;
use creation_service::{
    repository::diagram::ProvidesDiagramRepository,
    service::{
        entity::ProvidesEntityService,
        person::ProvidesPersonService,
        relationship::ProvidesRelationshipService,
        world::ProvidesWorldService,
    },
};
use thiserror::Error;

pub use get_genealogy_overview::{GetGenealogyOverviewUsecaseError, UsesGetGenealogyOverviewUsecase};

#[async_trait]
pub trait GenealogyOverviewUsecase:
    ProvidesWorldService
    + ProvidesDiagramRepository
    + ProvidesEntityService
    + ProvidesPersonService
    + ProvidesRelationshipService
{
}

#[derive(Debug, Error)]
pub enum GenealogyOverviewUsecaseError {
    #[error(transparent)]
    GetGenealogyOverviewUsecaseError(#[from] GetGenealogyOverviewUsecaseError),
}

#[async_trait]
pub trait UsesGenealogyOverviewUsecase: UsesGetGenealogyOverviewUsecase {
    async fn get_genealogy_overview(
        &self,
        body: creation_service::model::genealogy_overview::GetGenealogyOverviewSchema,
    ) -> Result<creation_service::model::genealogy_overview::GenealogyOverview, GetGenealogyOverviewUsecaseError> {
        UsesGetGenealogyOverviewUsecase::get_genealogy_overview(self, body).await
    }
}

impl<T> UsesGenealogyOverviewUsecase for T where T: UsesGetGenealogyOverviewUsecase {}

pub trait ProvidesGenealogyOverviewUsecase: Send + Sync + 'static {
    type T: UsesGenealogyOverviewUsecase + Sized;
    fn genealogy_overview_usecase(&self) -> &Self::T;
}

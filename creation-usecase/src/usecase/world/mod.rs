pub mod create_world;
pub mod delete_world;
pub mod get_world;
pub mod get_worlds;
pub mod update_world;

use async_trait::async_trait;
use creation_service::service::world::ProvidesWorldService;

use super::map_usecase_result;
use super::map_usecase_result_unit;

pub use create_world::{CreateWorldUsecaseError, UsesCreateWorldUsecase};
pub use delete_world::{DeleteWorldUsecaseError, UsesDeleteWorldUsecase};
pub use get_world::{GetWorldUsecaseError, UsesGetWorldUsecase};
pub use get_worlds::{GetWorldsUsecaseError, UsesGetWorldsUsecase};
pub use update_world::{UpdateWorldUsecaseError, UsesUpdateWorldUsecase};

#[async_trait]
pub trait WorldUsecase: ProvidesWorldService {}

#[async_trait]
pub trait UsesWorldUsecase:
    UsesGetWorldsUsecase + UsesGetWorldUsecase + UsesCreateWorldUsecase + UsesUpdateWorldUsecase + UsesDeleteWorldUsecase
{
    async fn get_worlds(&self, body: creation_service::model::world::GetWorldsSchema) -> Result<Vec<creation_service::model::world::World>, GetWorldsUsecaseError> {
        UsesGetWorldsUsecase::get_worlds(self, body).await
    }

    async fn get_world(&self, body: creation_service::model::world::GetWorldSchema) -> Result<creation_service::model::world::World, GetWorldUsecaseError> {
        UsesGetWorldUsecase::get_world(self, body).await
    }

    async fn create_world(&self, body: creation_service::model::world::CreateWorldSchema) -> Result<creation_service::model::world::World, CreateWorldUsecaseError> {
        UsesCreateWorldUsecase::create_world(self, body).await
    }

    async fn update_world(&self, body: creation_service::model::world::UpdateWorldSchema) -> Result<creation_service::model::world::World, UpdateWorldUsecaseError> {
        UsesUpdateWorldUsecase::update_world(self, body).await
    }

    async fn delete_world(&self, body: creation_service::model::world::DeleteWorldSchema) -> Result<(), DeleteWorldUsecaseError> {
        UsesDeleteWorldUsecase::delete_world(self, body).await
    }
}

impl<T> UsesWorldUsecase for T where
    T: UsesGetWorldsUsecase
        + UsesGetWorldUsecase
        + UsesCreateWorldUsecase
        + UsesUpdateWorldUsecase
        + UsesDeleteWorldUsecase
{
}

pub trait ProvidesWorldUsecase: Send + Sync + 'static {
    type T: UsesWorldUsecase + Sized;
    fn world_usecase(&self) -> &Self::T;
}

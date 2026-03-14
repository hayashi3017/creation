use config::Config;
use creation_adapter::{
    model::{diagram::DiagramTable, entity::EntityTable, user::UserTable},
    repository::RepositoryImpl,
};
use creation_service::{
    repository::{
        diagram::ProvidesDiagramRepository, entity::ProvidesEntityRepository,
        user::ProvidesUserRepository,
    },
    service::{
        diagram::{DiagramService, ProvidesDiagramService},
        entity::{EntityService, ProvidesEntityService},
        user::{ProvidesUserService, UserService},
    },
};
use creation_usecase::usecase::{
    diagram::{DiagramUsecase, ProvidesDiagramUsecase},
    entity::{EntityUsecase, ProvidesEntityUsecase},
    user::{ProvidesUserUsecase, UserUsecase},
};
use sqlx::{Pool, Postgres};

// FIXME: pub?
pub mod config;
mod handler;
mod jwt_auth;
pub mod middleware;
mod response;
pub mod route;
pub mod utils;

pub struct AppState {
    pub driver: AppModule,
    pub env: Config,
}

#[derive(Clone)]
pub struct AppModule {
    pub user_repository: RepositoryImpl<UserTable>,
    pub diagram_repository: RepositoryImpl<DiagramTable>,
    pub entity_repository: RepositoryImpl<EntityTable>,
}

impl AppModule {
    pub async fn new() -> Self {
        AppModule {
            user_repository: RepositoryImpl::<UserTable>::new().await,
            diagram_repository: RepositoryImpl::<DiagramTable>::new().await,
            entity_repository: RepositoryImpl::<EntityTable>::new().await,
        }
    }
    pub async fn new_test(pool: Pool<Postgres>) -> Self {
        // FIXME: pass refs instead of clone.
        AppModule {
            user_repository: RepositoryImpl::<UserTable>::new_test(pool.clone()).await,
            diagram_repository: RepositoryImpl::<DiagramTable>::new_test(pool.clone()).await,
            entity_repository: RepositoryImpl::<EntityTable>::new_test(pool.clone()).await,
        }
    }
}

impl ProvidesUserRepository for AppModule {
    type T = RepositoryImpl<UserTable>;

    fn user_repository(&self) -> &Self::T {
        &self.user_repository
    }
}

impl ProvidesDiagramRepository for AppModule {
    type T = RepositoryImpl<DiagramTable>;

    fn diagram_repository(&self) -> &Self::T {
        &self.diagram_repository
    }
}

impl ProvidesEntityRepository for AppModule {
    type T = RepositoryImpl<EntityTable>;

    fn entity_repository(&self) -> &Self::T {
        &self.entity_repository
    }
}

impl UserService for AppModule {}
impl DiagramService for AppModule {}
impl EntityService for AppModule {}

impl ProvidesUserService for AppModule {
    type T = Self;

    fn user_service(&self) -> &Self::T {
        self
    }
}

impl ProvidesDiagramService for AppModule {
    type T = Self;

    fn diagram_service(&self) -> &Self::T {
        self
    }
}

impl ProvidesEntityService for AppModule {
    type T = Self;

    fn entity_service(&self) -> &Self::T {
        self
    }
}

impl UserUsecase for AppModule {}
impl DiagramUsecase for AppModule {}
impl EntityUsecase for AppModule {}

impl ProvidesUserUsecase for AppModule {
    type T = Self;

    fn user_usecase(&self) -> &Self::T {
        self
    }
}

impl ProvidesDiagramUsecase for AppModule {
    type T = Self;

    fn diagram_usecase(&self) -> &Self::T {
        self
    }
}

impl ProvidesEntityUsecase for AppModule {
    type T = Self;

    fn entity_usecase(&self) -> &Self::T {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::AppModule;
    use creation_service::{
        repository::{
            diagram::ProvidesDiagramRepository, entity::ProvidesEntityRepository,
            user::ProvidesUserRepository,
        },
        service::{
            diagram::ProvidesDiagramService, entity::ProvidesEntityService,
            user::ProvidesUserService,
        },
    };
    use creation_usecase::usecase::{
        diagram::UsesDiagramUsecase, entity::UsesEntityUsecase, user::UsesUserUsecase,
    };

    trait UsesMultipleRepositories:
        ProvidesUserRepository + ProvidesDiagramRepository + ProvidesEntityRepository
    {
    }
    impl<T> UsesMultipleRepositories for T where
        T: ProvidesUserRepository + ProvidesDiagramRepository + ProvidesEntityRepository
    {
    }

    trait UsesMultipleServices:
        ProvidesUserService + ProvidesDiagramService + ProvidesEntityService
    {
    }
    impl<T> UsesMultipleServices for T where
        T: ProvidesUserService + ProvidesDiagramService + ProvidesEntityService
    {
    }

    fn assert_module_supports_multi_dependencies<T>()
    where
        T: UsesMultipleRepositories
            + UsesMultipleServices
            + UsesUserUsecase
            + UsesDiagramUsecase
            + UsesEntityUsecase,
    {
    }

    #[test]
    fn app_module_implements_multi_dependency_traits() {
        assert_module_supports_multi_dependencies::<AppModule>();
    }
}

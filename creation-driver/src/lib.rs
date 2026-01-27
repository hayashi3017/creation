use config::Config;
use creation_adapter::{
    model::{diagram::DiagramTable, user::UserTable},
    repository::RepositoryImpl,
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
}

impl AppModule {
    pub async fn new() -> Self {
        AppModule {
            user_repository: RepositoryImpl::<UserTable>::new().await,
            diagram_repository: RepositoryImpl::<DiagramTable>::new().await,
        }
    }
    pub async fn new_test(pool: Pool<Postgres>) -> Self {
        // FIXME: pass refs instead of clone.
        AppModule {
            user_repository: RepositoryImpl::<UserTable>::new_test(pool.clone()).await,
            diagram_repository: RepositoryImpl::<DiagramTable>::new_test(pool.clone()).await,
        }
    }
}

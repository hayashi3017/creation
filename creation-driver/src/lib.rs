use config::Config;
use creation_adapter::{model::user::UserTable, repository::RepositoryImpl};
use sqlx::{Pool, Postgres};

// FIXME: pub?
pub mod config;
mod handler;
mod jwt_auth;
pub mod middleware;
mod response;
pub mod route;

pub struct AppState {
    pub driver: AppModule,
    pub env: Config,
}

#[derive(Clone)]
pub struct AppModule {
    pub user_repository: RepositoryImpl<UserTable>,
}

impl AppModule {
    pub async fn new() -> Self {
        AppModule {
            user_repository: RepositoryImpl::<UserTable>::new().await,
        }
    }
    pub async fn new_test(pool: Pool<Postgres>) -> Self {
        AppModule {
            user_repository: RepositoryImpl::<UserTable>::new_test(pool).await,
        }
    }
}

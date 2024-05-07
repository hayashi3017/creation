use config::Config;
use creation_adapter::{model::user::UserTable, repository::mysql::RepositoryImpl};

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
}

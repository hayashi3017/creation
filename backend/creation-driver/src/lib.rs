use config::Config;
use creation_adapter::repository::mysql::DatabaseImpl;

// FIXME: pub?
pub mod config;
mod handler;
mod jwt_auth;
pub mod middleware;
mod response;
pub mod route;

pub struct AppState {
    pub db: DatabaseImpl,
    pub env: Config,
}

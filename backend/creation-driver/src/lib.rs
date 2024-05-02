use config::Config;
use sqlx::{MySql, Pool};

pub mod config;
mod jwt_auth;
mod response;
pub mod route;
mod handler;

pub struct AppState {
    pub db: Pool<MySql>,
    pub env: Config,
}

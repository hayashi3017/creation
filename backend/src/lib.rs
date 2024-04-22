use config::Config;
use sqlx::{MySql, Pool};

pub mod config;
mod handler;
mod jwt_auth;
mod model;
mod response;
pub mod route;

pub struct AppState {
    pub db: Pool<MySql>,
    pub env: Config,
}

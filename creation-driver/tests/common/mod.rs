use std::sync::Arc;

use axum::Router;
use creation_driver::{
    config::Config, middleware::cors::setup_cors, route::create_router, AppModule, AppState,
};
use dotenvy::dotenv;
use sqlx::{Pool, Postgres};
use tower::ServiceBuilder;

pub async fn setup_router(pool: Pool<Postgres>) -> Router {
    dotenv().ok();
    let config = Config::init();

    let module = AppModule::new_test(pool).await;
    let cors = setup_cors();

    create_router(Arc::new(AppState {
        driver: module.clone(),
        env: config.clone(),
    }))
    .layer(ServiceBuilder::new().layer(cors))
}

use std::sync::Arc;

use axum::Router;
use creation_driver::{
    config::Config, middleware::cors::setup_cors, route::create_router, utils::get_port, AppModule,
    AppState,
};
use dotenvy::dotenv;
use sqlx::{Pool, Postgres};
use tower::ServiceBuilder;

pub async fn setup_router(pool: Pool<Postgres>) -> Router {
    dotenv().ok();
    let config = Config::init();

    let module = AppModule::new_test(pool).await;
    let port = get_port(config.runtime_mode);
    let cors = setup_cors(&port);

    create_router(Arc::new(AppState {
        driver: module.clone(),
        env: config.clone(),
    }))
    .layer(ServiceBuilder::new().layer(cors))
}

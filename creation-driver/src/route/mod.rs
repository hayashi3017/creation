use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    handler::{
        diagram::{create_diagram, delete_diagram, get_diagrams, update_diagram},
        entity::{create_entity, delete_entity, get_entities, update_entity},
        health_check::health_checker_handler,
        user::{get_me_handler, login_user_handler, logout_handler, register_user_handler},
    },
    jwt_auth::auth,
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/auth/register", post(register_user_handler))
        .route("/api/auth/login", post(login_user_handler))
        .route(
            "/api/auth/logout",
            get(logout_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/users/me",
            get(get_me_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/diagrams",
            get(get_diagrams).route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/diagrams/create",
            post(create_diagram)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/diagrams/update",
            post(update_diagram)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/diagrams/delete",
            post(delete_diagram)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/entities",
            get(get_entities).route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/entities/create",
            post(create_entity)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/entities/update",
            post(update_entity)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/entities/delete",
            post(delete_entity)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state)
}

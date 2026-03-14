use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};

use crate::{
    handler::{
        diagram::{create_diagram, delete_diagram_by_id, get_diagrams, update_diagram_by_id},
        entity::{
            create_entity_in_diagram, delete_entity_by_id, get_entities_by_diagram,
            update_entity_by_id,
        },
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
            "/api/diagrams/update/{id}",
            patch(update_diagram_by_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/diagrams/delete/{id}",
            axum::routing::delete(delete_diagram_by_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/entities/create",
            post(create_entity_in_diagram)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/entities",
            get(get_entities_by_diagram)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/entities/update/{id}",
            patch(update_entity_by_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/entities/{id}",
            axum::routing::delete(delete_entity_by_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state)
}

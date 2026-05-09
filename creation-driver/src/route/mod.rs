use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};

use crate::{
    handler::{
        diagram::{
            create_diagram, create_diagram_entity_membership, delete_diagram_by_diagram_id,
            get_diagrams, sync_diagram_entity_memberships, update_diagram_by_diagram_id,
        },
        genealogy_diagram::get_genealogy_diagram_by_diagram_id,
        genealogy_world::get_genealogy_world,
        health_check::health_checker_handler,
        person::{
            create_person, delete_person_by_entity_id, get_persons_by_diagram,
            update_person_by_entity_id,
        },
        relationship::{
            create_relationship, delete_relationship_by_relationship_id,
            get_relationships_by_diagram, update_relationship_by_relationship_id,
        },
        user::{get_me_handler, login_user_handler, logout_handler, register_user_handler},
        world::{
            create_world, delete_world_by_id, get_world_by_id, get_worlds, update_world_by_id,
        },
    },
    jwt_auth::auth,
    openapi::{openapi_json, swagger_ui_html},
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/swagger-ui", get(swagger_ui_html))
        .route("/swagger-ui/", get(swagger_ui_html))
        .route("/api-docs/openapi.json", get(openapi_json))
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
            "/api/worlds",
            get(get_worlds).route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/worlds/:world_id",
            get(get_world_by_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/worlds/create",
            post(create_world).route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/worlds/update/:world_id",
            patch(update_world_by_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/worlds/delete/:world_id",
            delete(delete_world_by_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/genealogy/diagram/:diagram_id",
            get(get_genealogy_diagram_by_diagram_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/genealogy/world/:world_id",
            get(get_genealogy_world)
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
            "/api/diagrams/entities/create",
            post(create_diagram_entity_membership)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/diagrams/:diagram_id/entities/sync",
            post(sync_diagram_entity_memberships)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/diagrams/update/:diagram_id",
            patch(update_diagram_by_diagram_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/diagrams/delete/:diagram_id",
            axum::routing::delete(delete_diagram_by_diagram_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/persons",
            get(get_persons_by_diagram)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/persons/create",
            post(create_person)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/persons/update/:entity_id",
            patch(update_person_by_entity_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/persons/delete/:entity_id",
            axum::routing::delete(delete_person_by_entity_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/relationships",
            get(get_relationships_by_diagram)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/relationships/create",
            post(create_relationship)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/relationships/update/:relationship_id",
            patch(update_relationship_by_relationship_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/relationships/delete/:relationship_id",
            axum::routing::delete(delete_relationship_by_relationship_id)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .with_state(app_state)
}

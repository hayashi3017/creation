use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use creation_service::{
    model::world::{
        CreateWorldSchema, DeleteWorldSchema, GetWorldSchema, GetWorldsSchema, UpdateWorldSchema,
    },
    repository::world::{
        CreateWorldRepositoryError, DeleteWorldRepositoryError, GetWorldRepositoryError,
        GetWorldsRepositoryError, UpdateWorldRepositoryError,
    },
    service::world::{
        CreateWorldServiceError, DeleteWorldServiceError, GetWorldServiceError,
        GetWorldsServiceError, UpdateWorldServiceError,
    },
};
use creation_usecase::usecase::world::{
    CreateWorldUsecaseError, DeleteWorldUsecaseError, GetWorldUsecaseError, GetWorldsUsecaseError,
    UpdateWorldUsecaseError, UsesWorldUsecase,
};
use http::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    response::{ErrorResponse, WorldListResponse, WorldResponse},
    AppState,
};

type JsonError = (StatusCode, Json<ErrorResponse>);

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateWorldRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/worlds",
    tag = "Worlds",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    responses(
        (status = 200, description = "All active worlds.", body = WorldListResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 500, description = "The worlds could not be loaded.", body = ErrorResponse)
    )
)]
pub async fn get_worlds(State(data): State<Arc<AppState>>) -> Result<impl IntoResponse, JsonError> {
    match data.driver.get_worlds(GetWorldsSchema {}).await {
        Ok(ret) => Ok(Json(WorldListResponse {
            status: "success".to_string(),
            data: ret,
        })),
        Err(GetWorldsUsecaseError::GetWorldsServiceError(
            GetWorldsServiceError::GetWorldsRepositoryError(GetWorldsRepositoryError::Db(err)),
        )) => Err(internal_server_error(format!("Database error: {}", err))),
    }
}

#[utoipa::path(
    post,
    path = "/api/worlds",
    tag = "Worlds",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    request_body = CreateWorldSchema,
    responses(
        (status = 200, description = "The world was created successfully.", body = WorldResponse),
        (status = 400, description = "The request was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 500, description = "The world could not be created.", body = ErrorResponse)
    )
)]
pub async fn create_world(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateWorldSchema>,
) -> Result<impl IntoResponse, JsonError> {
    match data.driver.create_world(body).await {
        Ok(world) => Ok(Json(WorldResponse {
            status: "success".to_string(),
            data: world,
        })),
        Err(CreateWorldUsecaseError::CreateWorldServiceError(err)) => match err {
            CreateWorldServiceError::InvalidParams => {
                Err(bad_request_error("Invalid Parameter".to_string()))
            }
            CreateWorldServiceError::CreateWorldRepositoryError(
                CreateWorldRepositoryError::Db(err),
            ) => Err(internal_server_error(format!("Database error: {}", err))),
        },
    }
}

#[utoipa::path(
    get,
    path = "/api/worlds/{world_id}",
    tag = "Worlds",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("world_id" = usize, Path, description = "World identifier.")),
    responses(
        (status = 200, description = "The active world.", body = WorldResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The world was not found.", body = ErrorResponse),
        (status = 500, description = "The world could not be loaded.", body = ErrorResponse)
    )
)]
pub async fn get_world_by_id(
    Path(world_id): Path<usize>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    match data.driver.get_world(GetWorldSchema { world_id }).await {
        Ok(world) => Ok(Json(WorldResponse {
            status: "success".to_string(),
            data: world,
        })),
        Err(GetWorldUsecaseError::NotFound) => Err(not_found_error()),
        Err(GetWorldUsecaseError::GetWorldServiceError(err)) => match err {
            GetWorldServiceError::NotFound => Err(not_found_error()),
            GetWorldServiceError::GetWorldRepositoryError(GetWorldRepositoryError::Db(err)) => {
                Err(internal_server_error(format!("Database error: {}", err)))
            }
        },
    }
}

#[utoipa::path(
    patch,
    path = "/api/worlds/{world_id}",
    tag = "Worlds",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("world_id" = usize, Path, description = "World identifier.")),
    request_body = UpdateWorldRequest,
    responses(
        (status = 200, description = "The world was updated successfully.", body = WorldResponse),
        (status = 400, description = "The request was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The world was not found.", body = ErrorResponse),
        (status = 500, description = "The world could not be updated.", body = ErrorResponse)
    )
)]
pub async fn update_world_by_id(
    Path(world_id): Path<usize>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateWorldRequest>,
) -> Result<impl IntoResponse, JsonError> {
    let body = UpdateWorldSchema {
        world_id,
        name: body.name,
        description: body.description,
    };

    match data.driver.update_world(body).await {
        Ok(world) => Ok(Json(WorldResponse {
            status: "success".to_string(),
            data: world,
        })),
        Err(UpdateWorldUsecaseError::NotFound) => Err(not_found_error()),
        Err(UpdateWorldUsecaseError::UpdateWorldServiceError(err)) => match err {
            UpdateWorldServiceError::InvalidParams => {
                Err(bad_request_error("Invalid Parameter".to_string()))
            }
            UpdateWorldServiceError::NotFound => Err(not_found_error()),
            UpdateWorldServiceError::UpdateWorldRepositoryError(err) => match err {
                UpdateWorldRepositoryError::Db(err) => {
                    Err(internal_server_error(format!("Database error: {}", err)))
                }
                UpdateWorldRepositoryError::NotFound => Err(not_found_error()),
            },
        },
    }
}

#[utoipa::path(
    delete,
    path = "/api/worlds/{world_id}",
    tag = "Worlds",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("world_id" = usize, Path, description = "World identifier.")),
    responses(
        (status = 200, description = "The world was deleted successfully."),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The world was not found.", body = ErrorResponse),
        (status = 500, description = "The world could not be deleted.", body = ErrorResponse)
    )
)]
pub async fn delete_world_by_id(
    Path(world_id): Path<usize>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .delete_world(DeleteWorldSchema { world_id })
        .await
    {
        Ok(()) => Ok(()),
        Err(DeleteWorldUsecaseError::NotFound) => Err(not_found_error()),
        Err(DeleteWorldUsecaseError::DeleteWorldServiceError(err)) => match err {
            DeleteWorldServiceError::InvalidParams => {
                Err(bad_request_error("Invalid Parameter".to_string()))
            }
            DeleteWorldServiceError::NotFound => Err(not_found_error()),
            DeleteWorldServiceError::DeleteWorldRepositoryError(err) => match err {
                DeleteWorldRepositoryError::Db(err) => {
                    Err(internal_server_error(format!("Database error: {}", err)))
                }
                DeleteWorldRepositoryError::NotFound => Err(not_found_error()),
            },
        },
    }
}

fn bad_request_error(message: String) -> JsonError {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            status: "fail".to_string(),
            message,
        }),
    )
}

fn not_found_error() -> JsonError {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            status: "fail".to_string(),
            message: "Not Found".to_string(),
        }),
    )
}

fn internal_server_error(message: String) -> JsonError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            status: "fail".to_string(),
            message,
        }),
    )
}

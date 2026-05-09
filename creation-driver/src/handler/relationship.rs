use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use creation_service::model::relationship::{
    CreateRelationshipSchema, DeleteRelationshipSchema, GetRelationshipsSchema, RelationshipKind,
    UpdateRelationshipSchema,
};
use creation_usecase::usecase::relationship::{
    CreateRelationshipUsecaseError, DeleteRelationshipUsecaseError, GetRelationshipsUsecaseError,
    UpdateRelationshipUsecaseError, UsesRelationshipUsecase,
};
use http::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    response::{ErrorResponse, RelationshipListResponse},
    AppState,
};

type JsonError = (StatusCode, Json<ErrorResponse>);

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRelationshipRequest {
    pub source_entity_id: usize,
    pub target_entity_id: usize,
    pub kind: RelationshipKind,
    #[serde(default)]
    pub start_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub end_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub end_reason: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[doc = include_str!("../openapi_docs/en/operations/get_relationships_by_diagram.md")]
#[utoipa::path(
    get,
    path = "/api/relationships",
    tag = "Relationships",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    request_body = GetRelationshipsSchema,
    responses(
        (status = 200, description = "All relationships in the requested world.", body = RelationshipListResponse),
        (status = 400, description = "The request payload was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The world was not found.", body = ErrorResponse),
        (status = 500, description = "The relationships could not be loaded.", body = ErrorResponse)
    )
)]
pub async fn get_relationships_by_diagram(
    State(data): State<Arc<AppState>>,
    Json(body): Json<GetRelationshipsSchema>,
) -> Result<impl IntoResponse, JsonError> {
    match data.driver.get_relationships(body).await {
        Ok(ret) => Ok(Json(RelationshipListResponse {
            status: "success".to_string(),
            data: ret,
        })),
        Err(GetRelationshipsUsecaseError::InvalidParams) => Err(invalid_parameter_error()),
        Err(GetRelationshipsUsecaseError::NotFound) => Err(not_found_error()),
        Err(err) => Err(internal_server_error(err)),
    }
}

#[doc = include_str!("../openapi_docs/en/operations/create_relationship.md")]
#[utoipa::path(
    post,
    path = "/api/relationships/create",
    tag = "Relationships",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    request_body = CreateRelationshipSchema,
    responses(
        (status = 200, description = "The relationship was created successfully."),
        (status = 400, description = "The request payload was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The world was not found.", body = ErrorResponse),
        (status = 500, description = "The create operation failed.", body = ErrorResponse)
    )
)]
pub async fn create_relationship(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateRelationshipSchema>,
) -> Result<impl IntoResponse, JsonError> {
    match data.driver.create_relationship(body).await {
        Ok(()) => Ok(()),
        Err(CreateRelationshipUsecaseError::InvalidParams) => Err(invalid_parameter_error()),
        Err(CreateRelationshipUsecaseError::NotFound) => Err(not_found_error()),
        Err(err) => Err(internal_server_error(err)),
    }
}

#[doc = include_str!("../openapi_docs/en/operations/update_relationship_by_id.md")]
#[utoipa::path(
    patch,
    path = "/api/relationships/update/{relationship_id}",
    tag = "Relationships",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("relationship_id" = usize, Path, description = "Relationship identifier.")),
    request_body = UpdateRelationshipRequest,
    responses(
        (status = 200, description = "The relationship was updated successfully."),
        (status = 400, description = "The request payload was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The relationship or world was not found.", body = ErrorResponse),
        (status = 500, description = "The update operation failed.", body = ErrorResponse)
    )
)]
pub async fn update_relationship_by_relationship_id(
    Path(relationship_id): Path<usize>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateRelationshipRequest>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .update_relationship(UpdateRelationshipSchema {
            relationship_id,
            source_entity_id: body.source_entity_id,
            target_entity_id: body.target_entity_id,
            kind: body.kind,
            start_date: body.start_date,
            end_date: body.end_date,
            end_reason: body.end_reason,
            notes: body.notes,
        })
        .await
    {
        Ok(()) => Ok(()),
        Err(UpdateRelationshipUsecaseError::InvalidParams) => Err(invalid_parameter_error()),
        Err(UpdateRelationshipUsecaseError::NotFound) => Err(not_found_error()),
        Err(err) => Err(internal_server_error(err)),
    }
}

#[doc = include_str!("../openapi_docs/en/operations/delete_relationship_by_id.md")]
#[utoipa::path(
    delete,
    path = "/api/relationships/delete/{relationship_id}",
    tag = "Relationships",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("relationship_id" = usize, Path, description = "Relationship identifier.")),
    responses(
        (status = 200, description = "The relationship was deleted successfully."),
        (status = 400, description = "The request payload was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The relationship or world was not found.", body = ErrorResponse),
        (status = 500, description = "The delete operation failed.", body = ErrorResponse)
    )
)]
pub async fn delete_relationship_by_relationship_id(
    Path(relationship_id): Path<usize>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .delete_relationship(DeleteRelationshipSchema { relationship_id })
        .await
    {
        Ok(()) => Ok(()),
        Err(DeleteRelationshipUsecaseError::InvalidParams) => Err(invalid_parameter_error()),
        Err(DeleteRelationshipUsecaseError::NotFound) => Err(not_found_error()),
        Err(err) => Err(internal_server_error(err)),
    }
}

fn invalid_parameter_error() -> JsonError {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            status: "fail".to_string(),
            message: "Invalid Parameter".to_string(),
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

fn internal_server_error(err: impl std::fmt::Display) -> JsonError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            status: "fail".to_string(),
            message: format!("Internal error: {}", err),
        }),
    )
}

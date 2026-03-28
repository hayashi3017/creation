use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use creation_service::model::family_tree::GetFamilyTreeSchema;
use creation_usecase::usecase::family_tree::{GetFamilyTreeUsecaseError, UsesFamilyTreeUsecase};
use http::StatusCode;

use crate::{
    response::{ErrorResponse, FamilyTreeResponse},
    AppState,
};

type JsonError = (StatusCode, Json<ErrorResponse>);

#[doc = include_str!("../openapi_docs/en/operations/get_family_tree_by_diagram_id.md")]
#[utoipa::path(
    get,
    path = "/api/family-trees/{diagram_id}",
    tag = "FamilyTrees",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("diagram_id" = usize, Path, description = "Diagram identifier.")),
    responses(
        (status = 200, description = "Normalized family-tree projection for the requested diagram.", body = FamilyTreeResponse),
        (status = 400, description = "The diagram id was invalid or the diagram kind is not family_tree.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The diagram was not found.", body = ErrorResponse),
        (status = 500, description = "The family tree could not be loaded.", body = ErrorResponse)
    )
)]
pub async fn get_family_tree_by_diagram_id(
    Path(diagram_id): Path<usize>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .get_family_tree(GetFamilyTreeSchema { diagram_id })
        .await
    {
        Ok(ret) => Ok(Json(FamilyTreeResponse {
            status: "success".to_string(),
            data: ret,
        })),
        Err(GetFamilyTreeUsecaseError::InvalidParams) => {
            Err(bad_request_error("Invalid Parameter"))
        }
        Err(GetFamilyTreeUsecaseError::InvalidDiagramKind) => {
            Err(bad_request_error("Diagram kind must be family_tree"))
        }
        Err(GetFamilyTreeUsecaseError::NotFound) => Err(not_found_error()),
        Err(err) => Err(internal_server_error(err)),
    }
}

fn bad_request_error(message: impl Into<String>) -> JsonError {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            status: "fail".to_string(),
            message: message.into(),
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

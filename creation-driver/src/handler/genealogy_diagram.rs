use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use creation_service::model::genealogy_diagram::GetGenealogyDiagramSchema;
use creation_usecase::usecase::genealogy_diagram::{
    GetGenealogyDiagramUsecaseError, UsesGenealogyDiagramUsecase,
};
use http::StatusCode;

use crate::{
    response::{ErrorResponse, GenealogyDiagramResponse},
    AppState,
};

type JsonError = (StatusCode, Json<ErrorResponse>);

#[doc = include_str!("../openapi_docs/en/operations/get_genealogy_diagram_by_diagram_id.md")]
#[utoipa::path(
    get,
    path = "/api/genealogy/diagram/{diagram_id}",
    tag = "Genealogy",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("diagram_id" = usize, Path, description = "Diagram identifier.")),
    responses(
        (status = 200, description = "Normalized genealogy projection for the requested diagram.", body = GenealogyDiagramResponse),
        (status = 400, description = "The diagram id was invalid or the diagram kind cannot be rendered as genealogy.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The diagram was not found.", body = ErrorResponse),
        (status = 500, description = "The genealogy diagram could not be loaded.", body = ErrorResponse)
    )
)]
pub async fn get_genealogy_diagram_by_diagram_id(
    Path(diagram_id): Path<usize>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .get_genealogy_diagram(GetGenealogyDiagramSchema { diagram_id })
        .await
    {
        Ok(ret) => Ok(Json(GenealogyDiagramResponse {
            status: "success".to_string(),
            data: ret,
        })),
        Err(GetGenealogyDiagramUsecaseError::InvalidParams) => {
            Err(bad_request_error("Invalid Parameter"))
        }
        Err(GetGenealogyDiagramUsecaseError::InvalidDiagramKind) => {
            Err(bad_request_error("Diagram kind must be family_tree"))
        }
        Err(GetGenealogyDiagramUsecaseError::NotFound) => Err(not_found_error()),
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

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use creation_service::{
    model::diagram::{
        CreateDiagramSchema, DeleteDiagramSchema, DiagramKind, GetDiagramsSchema,
        UpdateDiagramSchema,
    },
    repository::diagram::{
        CreateDiagramRepositoryError, DeleteDiagramRepositoryError, GetDiagramsRepositoryError,
        UpdateDiagramRepositoryError,
    },
    service::diagram::{
        CreateDiagramServiceError, DeleteDiagramServiceError, GetDiagramsServiceError,
        UpdateDiagramServiceError,
    },
};
use creation_usecase::usecase::diagram::{
    CreateDiagramUsecaseError, DeleteDiagramUsecaseError, GetDiagramsUsecaseError,
    UpdateDiagramUsecaseError, UsesDiagramUsecase,
};
use http::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    response::{DiagramListResponse, ErrorResponse},
    AppState,
};

type JsonError = (StatusCode, Json<ErrorResponse>);

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDiagramRequest {
    pub name: String,
    pub kind: DiagramKind,
    #[serde(default)]
    pub description: Option<String>,
}

#[doc = include_str!("../openapi_docs/en/operations/get_diagrams.md")]
#[utoipa::path(
    get,
    path = "/api/diagrams",
    tag = "Diagrams",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    responses(
        (status = 200, description = "All accessible diagrams.", body = DiagramListResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 500, description = "The diagrams could not be loaded.", body = ErrorResponse)
    )
)]
pub async fn get_diagrams(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    let body = GetDiagramsSchema {};
    let query_result = data.driver.get_diagrams(body).await;

    match query_result {
        Ok(ret) => Ok(Json(DiagramListResponse {
            status: "success".to_string(),
            data: ret,
        })),
        Err(err) => match err {
            GetDiagramsUsecaseError::GetDiagramsServiceError(err) => match err {
                GetDiagramsServiceError::GetDiagramsRepositoryError(err) => match err {
                    GetDiagramsRepositoryError::Db(err) => {
                        Err(internal_server_error(format!("Database error: {}", err)))
                    }
                },
            },
        },
    }
}

#[doc = include_str!("../openapi_docs/en/operations/create_diagram.md")]
#[utoipa::path(
    post,
    path = "/api/diagrams/create",
    tag = "Diagrams",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    request_body = CreateDiagramSchema,
    responses(
        (status = 200, description = "The diagram was created successfully."),
        (status = 400, description = "The request was invalid or duplicated an existing diagram.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse)
    )
)]
pub async fn create_diagram(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateDiagramSchema>,
) -> Result<impl IntoResponse, JsonError> {
    let query_result = data.driver.create_diagram(body).await;

    match query_result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            CreateDiagramUsecaseError::CreateDiagramServiceError(err) => match err {
                CreateDiagramServiceError::InvalidParams => {
                    Err(bad_request_error("Invalid Parameter".to_string()))
                }
                CreateDiagramServiceError::DuplicateDiagram => {
                    Err(bad_request_error("Duplicate".to_string()))
                }
                CreateDiagramServiceError::CreateDiagramRepositoryError(err) => match err {
                    CreateDiagramRepositoryError::Db(err) => {
                        Err(bad_request_error(format!("Database error: {}", err)))
                    }
                },
            },
        },
    }
}

#[doc = include_str!("../openapi_docs/en/operations/update_diagram_by_id.md")]
#[utoipa::path(
    patch,
    path = "/api/diagrams/update/{id}",
    tag = "Diagrams",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("id" = usize, Path, description = "Diagram identifier.")),
    request_body = UpdateDiagramRequest,
    responses(
        (status = 200, description = "The diagram was updated successfully."),
        (status = 400, description = "The request was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The diagram was not found.", body = ErrorResponse),
        (status = 500, description = "The update failed due to a server-side error.", body = ErrorResponse)
    )
)]
pub async fn update_diagram_by_id(
    Path(id): Path<usize>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateDiagramRequest>,
) -> Result<impl IntoResponse, JsonError> {
    update_diagram_inner(
        data,
        UpdateDiagramSchema {
            id,
            name: body.name,
            kind: body.kind,
            description: body.description,
        },
    )
    .await
}

async fn update_diagram_inner(
    data: Arc<AppState>,
    body: UpdateDiagramSchema,
) -> Result<(), JsonError> {
    let query_result = data.driver.update_diagram(body).await;

    match query_result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            UpdateDiagramUsecaseError::NotFound => Err(not_found_error()),
            UpdateDiagramUsecaseError::UpdateDiagramServiceError(err) => match err {
                UpdateDiagramServiceError::InvalidParams => {
                    Err(bad_request_error("Invalid Parameter".to_string()))
                }
                UpdateDiagramServiceError::NotFound => Err(not_found_error()),
                UpdateDiagramServiceError::UpdateDiagramRepositoryError(err) => match err {
                    UpdateDiagramRepositoryError::Db(err) => {
                        Err(internal_server_error(format!("Database error: {}", err)))
                    }
                    UpdateDiagramRepositoryError::NotFound => Err(not_found_error()),
                },
            },
        },
    }
}

#[doc = include_str!("../openapi_docs/en/operations/delete_diagram_by_id.md")]
#[utoipa::path(
    delete,
    path = "/api/diagrams/delete/{id}",
    tag = "Diagrams",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("id" = usize, Path, description = "Diagram identifier.")),
    responses(
        (status = 200, description = "The diagram was deleted successfully."),
        (status = 400, description = "The request was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The diagram was not found.", body = ErrorResponse),
        (status = 500, description = "The delete operation failed.", body = ErrorResponse)
    )
)]
pub async fn delete_diagram_by_id(
    Path(id): Path<usize>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    delete_diagram_inner(data, DeleteDiagramSchema { id }).await
}

async fn delete_diagram_inner(
    data: Arc<AppState>,
    body: DeleteDiagramSchema,
) -> Result<(), JsonError> {
    let query_result = data.driver.delete_diagram(body).await;

    match query_result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            DeleteDiagramUsecaseError::NotFound => Err(not_found_error()),
            DeleteDiagramUsecaseError::DeleteDiagramServiceError(err) => match err {
                DeleteDiagramServiceError::InvalidParams => {
                    Err(bad_request_error("Invalid Parameter".to_string()))
                }
                DeleteDiagramServiceError::NotFound => Err(not_found_error()),
                DeleteDiagramServiceError::DeleteDiagramRepositoryError(err) => match err {
                    DeleteDiagramRepositoryError::Db(err) => {
                        Err(internal_server_error(format!("Database error: {}", err)))
                    }
                    DeleteDiagramRepositoryError::NotFound => Err(not_found_error()),
                },
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

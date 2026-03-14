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

use crate::AppState;

type JsonError = (StatusCode, Json<serde_json::Value>);

#[derive(Debug, Deserialize)]
pub struct UpdateDiagramRequest {
    pub name: String,
    pub kind: DiagramKind,
    pub description: String,
}

pub async fn get_diagrams(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    let body = GetDiagramsSchema {};
    let query_result = data.driver.get_diagrams(body).await;

    match query_result {
        Ok(ret) => {
            let res = serde_json::json!({
                "status": "success",
                "data": ret
            });

            Ok(Json(res))
        }
        Err(err) => match err {
            GetDiagramsUsecaseError::GetDiagramsServiceError(err) => match err {
                GetDiagramsServiceError::GetDiagramsRepositoryError(err) => match err {
                    GetDiagramsRepositoryError::Db(err) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "status": "fail",
                            "message": format!("Database error: {}", err)
                        })),
                    )),
                },
            },
        },
    }
}

pub async fn create_diagram(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateDiagramSchema>,
) -> Result<impl IntoResponse, JsonError> {
    let query_result = data.driver.create_diagram(body).await;

    match query_result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            CreateDiagramUsecaseError::CreateDiagramServiceError(err) => match err {
                CreateDiagramServiceError::InvalidParams => Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": format!("Invalid Parameter")
                    })),
                )),
                CreateDiagramServiceError::DuplicateDiagram => Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": format!("Duplicate")
                    })),
                )),
                CreateDiagramServiceError::CreateDiagramRepositoryError(err) => match err {
                    CreateDiagramRepositoryError::Db(err) => Err((
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "status": "fail",
                            "message": format!("Database error: {}", err)
                        })),
                    )),
                },
            },
        },
    }
}

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
            UpdateDiagramUsecaseError::UpdateDiagramServiceError(err) => match err {
                UpdateDiagramServiceError::InvalidParams => Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": "Invalid Parameter"
                    })),
                )),
                UpdateDiagramServiceError::UpdateDiagramRepositoryError(err) => match err {
                    UpdateDiagramRepositoryError::Db(err) => Err((
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "status": "fail",
                            "message": format!("Database error: {}", err)
                        })),
                    )),
                },
            },
        },
    }
}

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
            DeleteDiagramUsecaseError::DeleteDiagramServiceError(err) => match err {
                DeleteDiagramServiceError::InvalidParams => Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": "Invalid Parameter"
                    })),
                )),
                DeleteDiagramServiceError::DeleteDiagramRepositoryError(err) => match err {
                    DeleteDiagramRepositoryError::Db(err) => Err((
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "status": "fail",
                            "message": format!("Database error: {}", err)
                        })),
                    )),
                },
            },
        },
    }
}

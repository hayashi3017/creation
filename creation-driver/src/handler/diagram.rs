use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use creation_service::{
    model::diagram::{
        CreateDiagramSchema, DeleteDiagramSchema, GetDiagramsSchema, UpdateDiagramSchema,
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

use crate::AppState;

pub async fn get_diagrams(
    State(data): State<Arc<AppState>>,
    Json(body): Json<GetDiagramsSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // let query_result = data.driver.user_repository.regist_user(body).await;
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
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

pub async fn update_diagram(
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateDiagramSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

pub async fn delete_diagram(
    State(data): State<Arc<AppState>>,
    Json(body): Json<DeleteDiagramSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

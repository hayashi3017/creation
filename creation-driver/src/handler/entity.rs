use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use creation_service::{
    model::entity::{
        CreateEntitySchema, DeleteEntitySchema, GetEntitiesSchema, UpdateEntitySchema,
    },
    repository::entity::{
        CreateEntityRepositoryError, DeleteEntityRepositoryError, GetEntitiesRepositoryError,
        UpdateEntityRepositoryError,
    },
    service::entity::{
        CreateEntityServiceError, DeleteEntityServiceError, GetEntitiesServiceError,
        UpdateEntityServiceError,
    },
};
use creation_usecase::usecase::entity::{
    CreateEntityUsecaseError, DeleteEntityUsecaseError, GetEntitiesUsecaseError,
    UpdateEntityUsecaseError, UsesEntityUsecase,
};
use http::StatusCode;

use crate::AppState;

pub async fn get_entities(
    State(data): State<Arc<AppState>>,
    Json(body): Json<GetEntitiesSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = data.driver.get_entities(body).await;

    match query_result {
        Ok(ret) => {
            let res = serde_json::json!({
                "status": "success",
                "data": ret
            });

            Ok(Json(res))
        }
        Err(err) => match err {
            GetEntitiesUsecaseError::GetEntitiesServiceError(err) => match err {
                GetEntitiesServiceError::InvalidParams => Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": "Invalid Parameter"
                    })),
                )),
                GetEntitiesServiceError::GetEntitiesRepositoryError(err) => match err {
                    GetEntitiesRepositoryError::Db(err) => Err((
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

pub async fn create_entity(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateEntitySchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = data.driver.create_entity(body).await;

    match query_result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            CreateEntityUsecaseError::CreateEntityServiceError(err) => match err {
                CreateEntityServiceError::InvalidParams => Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": "Invalid Parameter"
                    })),
                )),
                CreateEntityServiceError::CreateEntityRepositoryError(err) => match err {
                    CreateEntityRepositoryError::Db(err) => Err((
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

pub async fn update_entity(
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateEntitySchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = data.driver.update_entity(body).await;

    match query_result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            UpdateEntityUsecaseError::UpdateEntityServiceError(err) => match err {
                UpdateEntityServiceError::InvalidParams => Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": "Invalid Parameter"
                    })),
                )),
                UpdateEntityServiceError::UpdateEntityRepositoryError(err) => match err {
                    UpdateEntityRepositoryError::Db(err) => Err((
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

pub async fn delete_entity(
    State(data): State<Arc<AppState>>,
    Json(body): Json<DeleteEntitySchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = data.driver.delete_entity(body).await;

    match query_result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            DeleteEntityUsecaseError::DeleteEntityServiceError(err) => match err {
                DeleteEntityServiceError::InvalidParams => Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": "Invalid Parameter"
                    })),
                )),
                DeleteEntityServiceError::DeleteEntityRepositoryError(err) => match err {
                    DeleteEntityRepositoryError::Db(err) => Err((
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

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
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

type JsonError = (StatusCode, Json<serde_json::Value>);

pub async fn get_entities_by_diagram(
    State(data): State<Arc<AppState>>,
    Json(body): Json<GetEntitiesSchema>,
) -> Result<impl IntoResponse, JsonError> {
    get_entities_inner(data, body).await
}

async fn get_entities_inner(
    data: Arc<AppState>,
    body: GetEntitiesSchema,
) -> Result<Json<serde_json::Value>, JsonError> {
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

pub async fn create_entity_in_diagram(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateEntitySchema>,
) -> Result<impl IntoResponse, JsonError> {
    create_entity_inner(data, body).await
}

async fn create_entity_inner(
    data: Arc<AppState>,
    body: CreateEntitySchema,
) -> Result<(), JsonError> {
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

pub async fn update_entity_by_id(
    Path(id): Path<usize>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateEntitySchema>,
) -> Result<impl IntoResponse, JsonError> {
    update_entity_inner(
        data,
        UpdateEntitySchema {
            id,
            diagram_id: body.diagram_id,
            kind: body.kind,
            name: body.name,
            description: body.description,
        },
    )
    .await
}

async fn update_entity_inner(
    data: Arc<AppState>,
    body: UpdateEntitySchema,
) -> Result<(), JsonError> {
    let query_result = data.driver.update_entity(body).await;

    match query_result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            UpdateEntityUsecaseError::NotFound => Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": "Not Found"
                })),
            )),
            UpdateEntityUsecaseError::UpdateEntityServiceError(err) => match err {
                UpdateEntityServiceError::InvalidParams => Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": "Invalid Parameter"
                    })),
                )),
                UpdateEntityServiceError::NotFound => Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": "Not Found"
                    })),
                )),
                UpdateEntityServiceError::UpdateEntityRepositoryError(err) => match err {
                    UpdateEntityRepositoryError::Db(err) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "status": "fail",
                            "message": format!("Database error: {}", err)
                        })),
                    )),
                    UpdateEntityRepositoryError::NotFound => Err((
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({
                            "status": "fail",
                            "message": "Not Found"
                        })),
                    )),
                },
            },
        },
    }
}

pub async fn delete_entity_by_id(
    Path(id): Path<usize>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    delete_entity_inner(data, DeleteEntitySchema { id }).await
}

async fn delete_entity_inner(
    data: Arc<AppState>,
    body: DeleteEntitySchema,
) -> Result<(), JsonError> {
    let query_result = data.driver.delete_entity(body).await;

    match query_result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            DeleteEntityUsecaseError::NotFound => Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "status": "fail",
                    "message": "Not Found"
                })),
            )),
            DeleteEntityUsecaseError::DeleteEntityServiceError(err) => match err {
                DeleteEntityServiceError::InvalidParams => Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": "Invalid Parameter"
                    })),
                )),
                DeleteEntityServiceError::NotFound => Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "status": "fail",
                        "message": "Not Found"
                    })),
                )),
                DeleteEntityServiceError::DeleteEntityRepositoryError(err) => match err {
                    DeleteEntityRepositoryError::Db(err) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "status": "fail",
                            "message": format!("Database error: {}", err)
                        })),
                    )),
                    DeleteEntityRepositoryError::NotFound => Err((
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({
                            "status": "fail",
                            "message": "Not Found"
                        })),
                    )),
                },
            },
        },
    }
}

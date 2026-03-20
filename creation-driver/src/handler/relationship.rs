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

use crate::AppState;

type JsonError = (StatusCode, Json<serde_json::Value>);

#[derive(Debug, Deserialize)]
pub struct UpdateRelationshipRequest {
    pub source_entity_id: usize,
    pub target_entity_id: usize,
    pub kind: RelationshipKind,
    #[serde(default)]
    pub start_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub end_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub notes: Option<String>,
}

pub async fn get_relationships_by_diagram(
    State(data): State<Arc<AppState>>,
    Json(body): Json<GetRelationshipsSchema>,
) -> Result<impl IntoResponse, JsonError> {
    match data.driver.get_relationships(body).await {
        Ok(ret) => Ok(Json(serde_json::json!({
            "status": "success",
            "data": ret
        }))),
        Err(GetRelationshipsUsecaseError::InvalidParams) => Err(invalid_parameter_error()),
        Err(GetRelationshipsUsecaseError::NotFound) => Err(not_found_error()),
        Err(err) => Err(internal_server_error(err)),
    }
}

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

pub async fn update_relationship_by_id(
    Path(id): Path<usize>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateRelationshipRequest>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .update_relationship(UpdateRelationshipSchema {
            id,
            source_entity_id: body.source_entity_id,
            target_entity_id: body.target_entity_id,
            kind: body.kind,
            start_date: body.start_date,
            end_date: body.end_date,
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

pub async fn delete_relationship_by_id(
    Path(id): Path<usize>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .delete_relationship(DeleteRelationshipSchema { id })
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
        Json(serde_json::json!({
            "status": "fail",
            "message": "Invalid Parameter"
        })),
    )
}

fn not_found_error() -> JsonError {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "status": "fail",
            "message": "Not Found"
        })),
    )
}

fn internal_server_error(err: impl std::fmt::Display) -> JsonError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "status": "fail",
            "message": format!("Internal error: {}", err)
        })),
    )
}

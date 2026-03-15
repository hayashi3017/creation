use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use creation_service::model::person::{
    CreatePersonSchema, DeletePersonSchema, GenderKind, GetPersonsSchema, UpdatePersonSchema,
};
use creation_usecase::usecase::person::{
    CreatePersonUsecaseError, DeletePersonUsecaseError, GetPersonsUsecaseError,
    UpdatePersonUsecaseError, UsesPersonUsecase,
};
use http::StatusCode;
use serde::Deserialize;

use crate::AppState;

type JsonError = (StatusCode, Json<serde_json::Value>);

#[derive(Debug, Deserialize)]
pub struct UpdatePersonRequest {
    pub diagram_id: usize,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub gender: Option<GenderKind>,
    #[serde(default)]
    pub birth_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub death_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub birthplace: Option<String>,
    #[serde(default)]
    pub residence: Option<String>,
    #[serde(default)]
    pub photo_url: Option<String>,
}

pub async fn get_persons_by_diagram(
    State(data): State<Arc<AppState>>,
    Json(body): Json<GetPersonsSchema>,
) -> Result<impl IntoResponse, JsonError> {
    match data.driver.get_persons(body).await {
        Ok(ret) => Ok(Json(serde_json::json!({
            "status": "success",
            "data": ret
        }))),
        Err(GetPersonsUsecaseError::InvalidParams) => Err(invalid_parameter_error()),
        Err(err) => Err(internal_server_error(err)),
    }
}

pub async fn create_person(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreatePersonSchema>,
) -> Result<impl IntoResponse, JsonError> {
    match data.driver.create_person(body).await {
        Ok(()) => Ok(()),
        Err(CreatePersonUsecaseError::InvalidParams) => Err(invalid_parameter_error()),
        Err(err) => Err(internal_server_error(err)),
    }
}

pub async fn update_person_by_entity_id(
    Path(entity_id): Path<usize>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdatePersonRequest>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .update_person(UpdatePersonSchema {
            entity_id,
            diagram_id: body.diagram_id,
            name: body.name,
            description: body.description,
            gender: body.gender,
            birth_date: body.birth_date,
            death_date: body.death_date,
            birthplace: body.birthplace,
            residence: body.residence,
            photo_url: body.photo_url,
        })
        .await
    {
        Ok(()) => Ok(()),
        Err(UpdatePersonUsecaseError::InvalidParams) => Err(invalid_parameter_error()),
        Err(UpdatePersonUsecaseError::NotFound) => Err(not_found_error()),
        Err(err) => Err(internal_server_error(err)),
    }
}

pub async fn delete_person_by_entity_id(
    Path(entity_id): Path<usize>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .delete_person(DeletePersonSchema { entity_id })
        .await
    {
        Ok(()) => Ok(()),
        Err(DeletePersonUsecaseError::InvalidParams) => Err(invalid_parameter_error()),
        Err(DeletePersonUsecaseError::NotFound) => Err(not_found_error()),
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

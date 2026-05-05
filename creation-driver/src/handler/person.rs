use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
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
use utoipa::ToSchema;

use crate::{
    response::{ErrorResponse, PersonListResponse},
    AppState,
};

type JsonError = (StatusCode, Json<ErrorResponse>);

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePersonRequest {
    pub world_id: usize,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub middle_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub first_name_kana: Option<String>,
    #[serde(default)]
    pub middle_name_kana: Option<String>,
    #[serde(default)]
    pub last_name_kana: Option<String>,
    #[serde(default)]
    pub first_name_romaji: Option<String>,
    #[serde(default)]
    pub middle_name_romaji: Option<String>,
    #[serde(default)]
    pub last_name_romaji: Option<String>,
    #[serde(default)]
    pub gender: Option<GenderKind>,
    #[serde(default)]
    pub birth_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub death_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub birthplace: Option<String>,
    #[serde(default)]
    pub deathplace: Option<String>,
    #[serde(default)]
    pub residence: Option<String>,
    #[serde(default)]
    pub photo_url: Option<String>,
    #[serde(default)]
    pub profile_text: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeletePersonQuery {
    pub world_id: usize,
}

#[doc = include_str!("../openapi_docs/en/operations/get_persons_by_diagram.md")]
#[utoipa::path(
    get,
    path = "/api/persons",
    tag = "Persons",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("world_id" = usize, Query, description = "World identifier.")),
    responses(
        (status = 200, description = "All person nodes in the requested world.", body = PersonListResponse),
        (status = 400, description = "The request was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 500, description = "The persons could not be loaded.", body = ErrorResponse)
    )
)]
pub async fn get_persons_by_diagram(
    Query(body): Query<GetPersonsSchema>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    match data.driver.get_persons(body).await {
        Ok(ret) => Ok(Json(PersonListResponse {
            status: "success".to_string(),
            data: ret,
        })),
        Err(GetPersonsUsecaseError::InvalidParams) => Err(invalid_parameter_error()),
        Err(err) => Err(internal_server_error(err)),
    }
}

#[doc = include_str!("../openapi_docs/en/operations/create_person.md")]
#[utoipa::path(
    post,
    path = "/api/persons/create",
    tag = "Persons",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    request_body = CreatePersonSchema,
    responses(
        (status = 200, description = "The person was created successfully."),
        (status = 400, description = "The request payload was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 500, description = "The create operation failed.", body = ErrorResponse)
    )
)]
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

#[doc = include_str!("../openapi_docs/en/operations/update_person_by_entity_id.md")]
#[utoipa::path(
    patch,
    path = "/api/persons/update/{entity_id}",
    tag = "Persons",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("entity_id" = usize, Path, description = "Entity identifier of the person.")),
    request_body = UpdatePersonRequest,
    responses(
        (status = 200, description = "The person was updated successfully."),
        (status = 400, description = "The request payload was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The person was not found.", body = ErrorResponse),
        (status = 500, description = "The update operation failed.", body = ErrorResponse)
    )
)]
pub async fn update_person_by_entity_id(
    Path(entity_id): Path<usize>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdatePersonRequest>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .update_person(UpdatePersonSchema {
            entity_id,
            world_id: body.world_id,
            name: body.name,
            description: body.description,
            first_name: body.first_name,
            middle_name: body.middle_name,
            last_name: body.last_name,
            first_name_kana: body.first_name_kana,
            middle_name_kana: body.middle_name_kana,
            last_name_kana: body.last_name_kana,
            first_name_romaji: body.first_name_romaji,
            middle_name_romaji: body.middle_name_romaji,
            last_name_romaji: body.last_name_romaji,
            gender: body.gender,
            birth_date: body.birth_date,
            death_date: body.death_date,
            birthplace: body.birthplace,
            deathplace: body.deathplace,
            residence: body.residence,
            photo_url: body.photo_url,
            profile_text: body.profile_text,
        })
        .await
    {
        Ok(()) => Ok(()),
        Err(UpdatePersonUsecaseError::InvalidParams) => Err(invalid_parameter_error()),
        Err(UpdatePersonUsecaseError::NotFound) => Err(not_found_error()),
        Err(err) => Err(internal_server_error(err)),
    }
}

#[doc = include_str!("../openapi_docs/en/operations/delete_person_by_entity_id.md")]
#[utoipa::path(
    delete,
    path = "/api/persons/delete/{entity_id}",
    tag = "Persons",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(
        ("entity_id" = usize, Path, description = "Entity identifier of the person."),
        ("world_id" = usize, Query, description = "World identifier.")
    ),
    responses(
        (status = 200, description = "The person was deleted successfully."),
        (status = 400, description = "The request payload was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The person was not found.", body = ErrorResponse),
        (status = 500, description = "The delete operation failed.", body = ErrorResponse)
    )
)]
pub async fn delete_person_by_entity_id(
    Path(entity_id): Path<usize>,
    Query(query): Query<DeletePersonQuery>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .delete_person(DeletePersonSchema {
            entity_id,
            world_id: query.world_id,
        })
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

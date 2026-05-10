use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use creation_service::{
    model::diagram::{
        CreateDiagramSchema, DeleteDiagramSchema, DiagramKind, GetDiagramsSchema,
        UpdateDiagramSchema,
    },
    model::entity::{CreateDiagramEntityMembershipSchema, SyncDiagramEntityMembershipsSchema},
    repository::diagram::{
        CreateDiagramRepositoryError, DeleteDiagramRepositoryError, GetDiagramsRepositoryError,
        UpdateDiagramRepositoryError,
    },
    repository::entity::CreateDiagramEntityMembershipRepositoryError,
    service::diagram::{
        CreateDiagramServiceError, DeleteDiagramServiceError, GetDiagramsServiceError,
        UpdateDiagramServiceError,
    },
    service::entity::CreateDiagramEntityMembershipServiceError,
};
use creation_usecase::usecase::{
    diagram::{
        CreateDiagramUsecaseError, DeleteDiagramUsecaseError, GetDiagramsUsecaseError,
        UpdateDiagramUsecaseError, UsesDiagramUsecase,
    },
    entity::{
        CreateDiagramEntityMembershipUsecaseError, SyncDiagramEntityMembershipsUsecaseError,
        UsesEntityUsecase,
    },
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
pub struct GetDiagramsQuery {
    pub world_id: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDiagramRequest {
    pub world_id: usize,
    pub name: String,
    pub kind: DiagramKind,
    #[serde(default = "default_true")]
    pub genealogy_overview_enabled: bool,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SyncDiagramEntityMembershipsRequest {
    pub entity_ids: Vec<usize>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteDiagramQuery {
    pub world_id: usize,
}

fn default_true() -> bool {
    true
}

#[doc = include_str!("../openapi_docs/en/operations/get_diagrams.md")]
#[utoipa::path(
    get,
    path = "/api/diagrams",
    tag = "Diagrams",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("world_id" = usize, Query, description = "World identifier.")),
    responses(
        (status = 200, description = "All accessible diagrams.", body = DiagramListResponse),
        (status = 400, description = "The request was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 500, description = "The diagrams could not be loaded.", body = ErrorResponse)
    )
)]
pub async fn get_diagrams(
    Query(query): Query<GetDiagramsQuery>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    let body = GetDiagramsSchema {
        world_id: query.world_id,
    };
    let query_result = data.driver.get_diagrams(body).await;

    match query_result {
        Ok(ret) => Ok(Json(DiagramListResponse {
            status: "success".to_string(),
            data: ret,
        })),
        Err(err) => match err {
            GetDiagramsUsecaseError::GetDiagramsServiceError(err) => match err {
                GetDiagramsServiceError::InvalidParams => {
                    Err(bad_request_error("Invalid Parameter".to_string()))
                }
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

#[utoipa::path(
    post,
    path = "/api/diagrams/entities/create",
    tag = "Diagrams",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    request_body = CreateDiagramEntityMembershipSchema,
    responses(
        (status = 200, description = "The entity was added to the diagram successfully."),
        (status = 400, description = "The request was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The diagram or entity was not found in the same world.", body = ErrorResponse),
        (status = 500, description = "The membership could not be created.", body = ErrorResponse)
    )
)]
pub async fn create_diagram_entity_membership(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateDiagramEntityMembershipSchema>,
) -> Result<impl IntoResponse, JsonError> {
    let query_result = data.driver.create_diagram_entity_membership(body).await;

    match query_result {
        Ok(()) => Ok(()),
        Err(err) => match err {
            CreateDiagramEntityMembershipUsecaseError::InvalidParams => {
                Err(bad_request_error("Invalid Parameter".to_string()))
            }
            CreateDiagramEntityMembershipUsecaseError::NotFound => Err(not_found_error()),
            CreateDiagramEntityMembershipUsecaseError::CreateDiagramEntityMembershipServiceError(
                err,
            ) => match err {
                CreateDiagramEntityMembershipServiceError::InvalidParams => {
                    Err(bad_request_error("Invalid Parameter".to_string()))
                }
                CreateDiagramEntityMembershipServiceError::NotFound => Err(not_found_error()),
                CreateDiagramEntityMembershipServiceError::CreateDiagramEntityMembershipRepositoryError(
                    err,
                ) => match err {
                    CreateDiagramEntityMembershipRepositoryError::Db(err) => {
                        Err(internal_server_error(format!("Database error: {}", err)))
                    }
                    CreateDiagramEntityMembershipRepositoryError::NotFound => Err(not_found_error()),
                },
            },
        },
    }
}

#[utoipa::path(
    post,
    path = "/api/diagrams/{diagram_id}/entities/sync",
    tag = "Diagrams",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("diagram_id" = usize, Path, description = "Diagram identifier.")),
    request_body = SyncDiagramEntityMembershipsRequest,
    responses(
        (status = 200, description = "The entity memberships were synchronized successfully."),
        (status = 400, description = "The request was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The entity or one of the diagrams was not found in the same world.", body = ErrorResponse),
        (status = 500, description = "The memberships could not be synchronized.", body = ErrorResponse)
    )
)]
pub async fn sync_diagram_entity_memberships(
    Path(diagram_id): Path<usize>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<SyncDiagramEntityMembershipsRequest>,
) -> Result<impl IntoResponse, JsonError> {
    match data
        .driver
        .sync_diagram_entity_memberships(SyncDiagramEntityMembershipsSchema {
            diagram_id,
            entity_ids: body.entity_ids,
        })
        .await
    {
        Ok(()) => Ok(()),
        Err(SyncDiagramEntityMembershipsUsecaseError::InvalidParams) => {
            Err(bad_request_error("Invalid Parameter".to_string()))
        }
        Err(SyncDiagramEntityMembershipsUsecaseError::NotFound) => Err(not_found_error()),
        Err(err) => Err(internal_server_error(format!("Database error: {}", err))),
    }
}

#[doc = include_str!("../openapi_docs/en/operations/update_diagram_by_id.md")]
#[utoipa::path(
    patch,
    path = "/api/diagrams/update/{diagram_id}",
    tag = "Diagrams",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(("diagram_id" = usize, Path, description = "Diagram identifier.")),
    request_body = UpdateDiagramRequest,
    responses(
        (status = 200, description = "The diagram was updated successfully."),
        (status = 400, description = "The request was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The diagram was not found.", body = ErrorResponse),
        (status = 500, description = "The update failed due to a server-side error.", body = ErrorResponse)
    )
)]
pub async fn update_diagram_by_diagram_id(
    Path(diagram_id): Path<usize>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateDiagramRequest>,
) -> Result<impl IntoResponse, JsonError> {
    update_diagram_inner(
        data,
        UpdateDiagramSchema {
            diagram_id,
            world_id: body.world_id,
            name: body.name,
            kind: body.kind,
            genealogy_overview_enabled: body.genealogy_overview_enabled,
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
    path = "/api/diagrams/delete/{diagram_id}",
    tag = "Diagrams",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(
        ("diagram_id" = usize, Path, description = "Diagram identifier."),
        ("world_id" = usize, Query, description = "World identifier.")
    ),
    responses(
        (status = 200, description = "The diagram was deleted successfully."),
        (status = 400, description = "The request was invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The diagram was not found.", body = ErrorResponse),
        (status = 500, description = "The delete operation failed.", body = ErrorResponse)
    )
)]
pub async fn delete_diagram_by_diagram_id(
    Path(diagram_id): Path<usize>,
    Query(query): Query<DeleteDiagramQuery>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, JsonError> {
    delete_diagram_inner(
        data,
        DeleteDiagramSchema {
            diagram_id,
            world_id: query.world_id,
        },
    )
    .await
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
            err => Err(internal_server_error(format!("Delete error: {}", err))),
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

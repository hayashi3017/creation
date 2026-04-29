use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use creation_service::model::genealogy_overview::GetGenealogyOverviewSchema;
use creation_usecase::usecase::genealogy_overview::{
    GetGenealogyOverviewUsecaseError, UsesGenealogyOverviewUsecase,
};
use http::StatusCode;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    response::{ErrorResponse, GenealogyOverviewResponse},
    AppState,
};

type JsonError = (StatusCode, Json<ErrorResponse>);

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetGenealogyOverviewQuery {
    #[serde(default)]
    pub center_entity_id: Option<usize>,
    #[serde(default)]
    pub ancestor_depth: Option<usize>,
    #[serde(default)]
    pub descendant_depth: Option<usize>,
    #[serde(default)]
    pub diagram_ids: Option<String>,
    #[serde(default)]
    pub as_of: Option<chrono::NaiveDate>,
}

#[utoipa::path(
    get,
    path = "/api/genealogy/world/{world_id}",
    tag = "Genealogy",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    params(
        ("world_id" = usize, Path, description = "World identifier."),
        ("center_entity_id" = Option<usize>, Query, description = "Optional center entity for a scoped overview."),
        ("ancestor_depth" = Option<usize>, Query, description = "Optional ancestor depth when center_entity_id is set."),
        ("descendant_depth" = Option<usize>, Query, description = "Optional descendant depth when center_entity_id is set."),
        ("diagram_ids" = Option<String>, Query, description = "Optional comma-separated diagram filter, e.g. diagram_ids=1,2."),
        ("as_of" = Option<chrono::NaiveDate>, Query, description = "Optional as-of date in YYYY-MM-DD format.")
    ),
    responses(
        (status = 200, description = "Merged world-scoped genealogy overview.", body = GenealogyOverviewResponse),
        (status = 400, description = "The request parameters were invalid.", body = ErrorResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 404, description = "The world was not found.", body = ErrorResponse),
        (status = 409, description = "No enabled genealogy diagrams are visible for overview output.", body = ErrorResponse),
        (status = 500, description = "The genealogy overview could not be loaded.", body = ErrorResponse)
    )
)]
pub async fn get_genealogy_overview(
    State(data): State<Arc<AppState>>,
    Path(world_id): Path<usize>,
    Query(query): Query<GetGenealogyOverviewQuery>,
) -> Result<impl IntoResponse, JsonError> {
    let body = query.try_into_schema(world_id)?;

    match data.driver.get_genealogy_overview(body).await {
        Ok(ret) => Ok(Json(GenealogyOverviewResponse {
            status: "success".to_string(),
            data: ret,
        })),
        Err(GetGenealogyOverviewUsecaseError::InvalidParams) => {
            Err(error(StatusCode::BAD_REQUEST, "Invalid Parameter"))
        }
        Err(GetGenealogyOverviewUsecaseError::NotFound) => {
            Err(error(StatusCode::NOT_FOUND, "Not Found"))
        }
        Err(GetGenealogyOverviewUsecaseError::NoVisibleGenealogyDiagrams) => {
            Err(error(StatusCode::CONFLICT, "NO_VISIBLE_GENEALOGY_DIAGRAMS"))
        }
        Err(err) => Err(error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal error: {}", err),
        )),
    }
}

impl GetGenealogyOverviewQuery {
    fn try_into_schema(self, world_id: usize) -> Result<GetGenealogyOverviewSchema, JsonError> {
        Ok(GetGenealogyOverviewSchema {
            world_id,
            center_entity_id: self.center_entity_id,
            ancestor_depth: self.ancestor_depth,
            descendant_depth: self.descendant_depth,
            diagram_ids: parse_diagram_ids(self.diagram_ids)?,
            as_of: self.as_of,
        })
    }
}

fn parse_diagram_ids(diagram_ids: Option<String>) -> Result<Option<Vec<usize>>, JsonError> {
    let Some(diagram_ids) = diagram_ids else {
        return Ok(None);
    };

    if diagram_ids.trim().is_empty() {
        return Ok(None);
    }

    diagram_ids
        .split(',')
        .map(|diagram_id| {
            diagram_id
                .trim()
                .parse::<usize>()
                .map_err(|_| error(StatusCode::BAD_REQUEST, "Invalid Parameter"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn error(status: StatusCode, message: impl Into<String>) -> JsonError {
    (
        status,
        Json(ErrorResponse {
            status: "fail".to_string(),
            message: message.into(),
        }),
    )
}

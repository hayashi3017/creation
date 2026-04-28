use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use creation_service::model::genealogy_overview::GetGenealogyOverviewSchema;
use creation_usecase::usecase::genealogy_overview::{
    GetGenealogyOverviewUsecaseError, UsesGenealogyOverviewUsecase,
};
use http::StatusCode;

use crate::{
    response::{ErrorResponse, GenealogyOverviewResponse},
    AppState,
};

type JsonError = (StatusCode, Json<ErrorResponse>);

#[utoipa::path(
    post,
    path = "/api/genealogy/overview",
    tag = "Genealogy",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    request_body = GetGenealogyOverviewSchema,
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
    Json(body): Json<GetGenealogyOverviewSchema>,
) -> Result<impl IntoResponse, JsonError> {
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

fn error(status: StatusCode, message: impl Into<String>) -> JsonError {
    (
        status,
        Json(ErrorResponse {
            status: "fail".to_string(),
            message: message.into(),
        }),
    )
}

use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use creation_adapter::model::user::UserTable;
use creation_service::{
    model::user::{FilteredUser, LoginUserSchema, RegisterUserSchema, TokenClaims},
    repository::user::{
        UserConfirmRepositoryError, UserLoginRepositoryError, UserResistRepositoryError,
    },
    service::user::{UserLoginServiceError, UserRegistServiceError},
};
use creation_usecase::usecase::user::{
    UserLoginUsecaseError, UserRegistUsecaseError, UsesUserUsecase,
};
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::{
    response::{
        ErrorResponse, LoginUserResponse, OperationResultData, RegisterUserResponse,
        StatusResponse, UserData, UserResponse,
    },
    AppState,
};

type JsonError = (StatusCode, Json<ErrorResponse>);

#[doc = include_str!("../openapi_docs/en/operations/register_user_handler.md")]
#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "Auth",
    request_body = RegisterUserSchema,
    responses(
        (status = 200, description = "User registration succeeded.", body = RegisterUserResponse),
        (status = 409, description = "A user with the same email already exists.", body = ErrorResponse),
        (status = 500, description = "The registration request failed.", body = ErrorResponse)
    )
)]
pub async fn register_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, JsonError> {
    let query_result = data.driver.regist_user(body).await;

    match query_result {
        Ok(()) => Ok(Json(RegisterUserResponse {
            status: "success".to_string(),
            data: OperationResultData {
                response: "ok".to_string(),
            },
        })),
        Err(err) => match err {
            UserRegistUsecaseError::UserRegistServiceError(err) => match err {
                UserRegistServiceError::UserConfirmRepositoryError(err) => match err {
                    UserConfirmRepositoryError::Db(e) => {
                        Err(internal_server_error(format!("Database error: {}", e)))
                    }
                },
                UserRegistServiceError::DubpicateUser => Err(conflict_error(
                    "User with that email already exists".to_string(),
                )),
                UserRegistServiceError::UserResistRepositoryError(err) => match err {
                    UserResistRepositoryError::Db(e) => {
                        Err(internal_server_error(format!("Database error: {}", e)))
                    }
                    UserResistRepositoryError::HashingPassword(e) => Err(internal_server_error(
                        format!("Error while hashing password: {}", e),
                    )),
                },
            },
        },
    }
}

#[doc = include_str!("../openapi_docs/en/operations/login_user_handler.md")]
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    request_body = LoginUserSchema,
    responses(
        (
            status = 200,
            description = "The user was authenticated successfully.",
            body = LoginUserResponse,
            headers(("set-cookie" = String, description = "HttpOnly session cookie named `token`."))
        ),
        (status = 400, description = "The email or password is invalid.", body = ErrorResponse),
        (status = 500, description = "The authentication request failed.", body = ErrorResponse)
    )
)]
pub async fn login_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, JsonError> {
    let query_result = data.driver.login_user(body).await;

    match query_result {
        Ok(user) => {
            let now = chrono::Utc::now();
            let iat = now.timestamp() as usize;
            let exp = (now + chrono::Duration::try_minutes(60).unwrap()).timestamp() as usize;
            let claims: TokenClaims = TokenClaims {
                sub: user.user_id.to_string(),
                exp,
                iat,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
            )
            .unwrap();

            let cookie = Cookie::build(("token", token.to_owned()))
                .path("/")
                .max_age(time::Duration::hours(1))
                .same_site(SameSite::Lax)
                .http_only(true);

            let mut response = Json(LoginUserResponse {
                status: "success".to_string(),
                token,
            })
            .into_response();
            response
                .headers_mut()
                .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
            Ok(response)
        }
        Err(err) => match err {
            UserLoginUsecaseError::UserLoginServiceError(err) => match err {
                UserLoginServiceError::UserLoginRepositoryError(err) => match err {
                    UserLoginRepositoryError::Db(e) => {
                        Err(internal_server_error(format!("Database error: {}", e)))
                    }
                    UserLoginRepositoryError::WrongUser => {
                        Err(bad_request_error("Invalid email or password".to_string()))
                    }
                    UserLoginRepositoryError::WrongPassword => {
                        Err(bad_request_error("Wrong password".to_string()))
                    }
                },
            },
        },
    }
}

#[doc = include_str!("../openapi_docs/en/operations/logout_handler.md")]
#[utoipa::path(
    get,
    path = "/api/auth/logout",
    tag = "Auth",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    responses(
        (
            status = 200,
            description = "The authentication cookie was cleared.",
            body = StatusResponse,
            headers(("set-cookie" = String, description = "Expired session cookie named `token`."))
        ),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 500, description = "Authentication middleware failed.", body = ErrorResponse)
    )
)]
pub async fn logout_handler() -> Result<impl IntoResponse, JsonError> {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    let mut response = Json(StatusResponse {
        status: "success".to_string(),
    })
    .into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}

#[doc = include_str!("../openapi_docs/en/operations/get_me_handler.md")]
#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "Users",
    security(("cookie_auth" = []), ("bearer_auth" = [])),
    responses(
        (status = 200, description = "The authenticated user profile.", body = UserResponse),
        (status = 401, description = "Authentication is required.", body = ErrorResponse),
        (status = 500, description = "Authentication or data loading failed.", body = ErrorResponse)
    )
)]
pub async fn get_me_handler(
    Extension(user): Extension<UserTable>,
) -> Result<impl IntoResponse, JsonError> {
    Ok(Json(UserResponse {
        status: "success".to_string(),
        data: UserData {
            user: filter_user_record(&user),
        },
    }))
}

pub fn filter_user_record(user: &UserTable) -> FilteredUser {
    FilteredUser {
        user_id: user.user_id.to_string(),
        email: user.email.to_owned(),
        name: user.name.to_owned(),
        photo: user.photo.to_owned(),
        role: user.role.to_owned(),
        createdAt: user.created_at,
        updatedAt: user.updated_at,
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

fn conflict_error(message: String) -> JsonError {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            status: "fail".to_string(),
            message,
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

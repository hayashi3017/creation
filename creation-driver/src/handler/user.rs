use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::State,
    http::{header, Response, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use creation_adapter::model::user::UserTable;
use creation_service::{
    model::user::{LoginUserSchema, RegisterUserSchema, TokenClaims},
    repository::user::UserRepositoryError,
    service::user::UserServiceError,
};
use creation_usecase::usecase::user::{UserUsecaseError, UsesUserUsecase};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;

use crate::{response::FilteredUser, AppState};

pub async fn register_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = data.driver.user_repository.regist_user(body).await;

    match query_result {
        Ok(()) => {
            let user_response = serde_json::json!({
                "status": "success",
                "data": serde_json::json!({"response": "ok"})
            });

            Ok(Json(user_response))
        }
        Err(err) => match err {
            UserUsecaseError::UserServiceError(err) => match err {
                UserServiceError::UserRepositoryError(err) => match err {
                    UserRepositoryError::Db(e) => {
                        let error_response = serde_json::json!({
                            "status": "fail",
                            "message": format!("Database error: {}", e),
                        });
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                    }
                    UserRepositoryError::DubpicateUser => {
                        let error_response = serde_json::json!({
                            "status": "fail",
                            "message": "User with that email already exists",
                        });
                        Err((StatusCode::CONFLICT, Json(error_response)))
                    }
                    UserRepositoryError::HashingPassword(e) => {
                        let error_response = serde_json::json!({
                            "status": "fail",
                            "message": format!("Error while hashing password: {}", e),
                        });
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                    }
                },
            },
        },
    }
}

pub async fn login_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user = sqlx::query_as!(
        UserTable,
        r#"
            SELECT
                id as `id: _`, 
                name, email, photo,
                password,
                role,
                created_at,
                tz_created_at as `tz_created_at: _`,
                updated_at,
                tz_updated_at as `tz_updated_at: _`
            FROM users WHERE email = ?
        "#,
        body.email.to_ascii_lowercase()
    )
    .fetch_optional(&data.driver.user_repository.pool.0)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?
    .ok_or_else(|| {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Invalid email or password",
        });
        (StatusCode::BAD_REQUEST, Json(error_response))
    })?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Invalid email or password"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::try_minutes(60).unwrap()).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: user.id.to_string(),
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

    let mut response = Response::new(json!({"status": "success", "token": token}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}

pub async fn logout_handler() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    let mut response = Response::new(json!({"status": "success"}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}

pub async fn get_me_handler(
    Extension(user): Extension<UserTable>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let json_response = serde_json::json!({
        "status":  "success",
        "data": serde_json::json!({
            "user": filter_user_record(&user)
        })
    });

    Ok(Json(json_response))
}

fn filter_user_record(user: &UserTable) -> FilteredUser {
    FilteredUser {
        id: user.id.to_string(),
        email: user.email.to_owned(),
        name: user.name.to_owned(),
        photo: user.photo.to_owned(),
        role: user.role.to_owned(),
        createdAt: user.created_at.unwrap(),
        updatedAt: user.updated_at.unwrap(),
    }
}

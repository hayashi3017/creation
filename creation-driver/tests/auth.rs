mod common;

use axum::body::Body;
use creation_service::model::user::TokenClaims;
use http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::setup_router;

#[sqlx::test]
async fn health_checker_returns_ok(db: PgPool) {
    set_test_env();

    let mut router = setup_router(db).await;
    let resp = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/healthchecker")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "success");
}

#[sqlx::test(fixtures("auth"))]
async fn login_user_returns_token_and_cookie(db: PgPool) {
    set_test_env();

    let mut router = setup_router(db).await;
    let resp = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "email": "login@example.com",
                        "password": "test_password"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    assert!(set_cookie.contains("token="));

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "success");
    assert!(json["token"].as_str().unwrap_or("").len() > 10);
}

#[sqlx::test(fixtures("auth"))]
async fn login_user_rejects_wrong_password(db: PgPool) {
    set_test_env();

    let mut router = setup_router(db).await;
    let resp = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "email": "badpass@example.com",
                        "password": "wrong_password"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "fail");
    assert_eq!(json["message"], "Wrong password");
}

#[sqlx::test(fixtures("auth"))]
async fn get_me_returns_user(db: PgPool) {
    set_test_env();

    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000020").unwrap();
    let token = create_token(user_id, "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/users/me")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["user"]["email"], "me@example.com");
}

#[sqlx::test(fixtures("auth"))]
async fn logout_clears_cookie(db: PgPool) {
    set_test_env();

    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000021").unwrap();
    let token = create_token(user_id, "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/auth/logout")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    assert!(set_cookie.contains("token="));
    assert!(set_cookie.contains("Max-Age=-3600"));
}

fn set_test_env() {
    std::env::set_var("JWT_SECRET", "test_secret");
    std::env::set_var("JWT_EXPIRED_IN", "60m");
    std::env::set_var("JWT_MAXAGE", "60");
    std::env::set_var("RUNTIME_MODE", "debug");
}

fn create_token(user_id: Uuid, secret: &str) -> String {
    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::try_minutes(60).unwrap()).timestamp() as usize;
    let claims = TokenClaims {
        sub: user_id.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}

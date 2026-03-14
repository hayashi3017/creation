mod common;

use std::borrow::BorrowMut;

use axum::body::Body;
use creation_service::model::user::TokenClaims;
use http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use sqlx::{PgPool, Row};
use tower::ServiceExt;

use crate::common::setup_router;

#[sqlx::test(fixtures("entity"))]
async fn get_entities_returns_list(db: PgPool) {
    set_test_env();

    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/entities")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"diagram_id":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert!(json["data"].is_array());
    assert_eq!(json["data"].as_array().unwrap().len(), 2);
}

#[sqlx::test(fixtures("entity"))]
async fn get_entities_rejects_zero_diagram_id(db: PgPool) {
    set_test_env();

    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/entities")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"diagram_id":0}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("entity"))]
async fn create_entity_returns_ok(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/entities/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "diagram_id": 1,
                        "kind": "person",
                        "name": "New Entity",
                        "description": "created from test"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*) FROM entity WHERE name = $1 AND diagram_id = $2
        "#,
    )
    .bind("New Entity")
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(count, 1);
}

#[sqlx::test(fixtures("entity"))]
async fn create_entity_rejects_empty_name(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/entities/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "diagram_id": 1,
                        "kind": "person",
                        "name": "",
                        "description": "invalid"
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
    assert_eq!(json["message"], "Invalid Parameter");
}

#[sqlx::test(fixtures("entity"))]
async fn update_entity_returns_ok(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/entities/update/1")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "diagram_id": 2,
                        "kind": "person",
                        "name": "Updated Entity",
                        "description": "updated from handler test"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let row = sqlx::query(
        r#"
            SELECT diagram_id, name, description FROM entity WHERE id = $1
        "#,
    )
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(row.get::<i64, _>("diagram_id"), 2);
    assert_eq!(row.get::<String, _>("name"), "Updated Entity");
    assert_eq!(
        row.get::<Option<String>, _>("description").as_deref(),
        Some("updated from handler test")
    );
}

#[sqlx::test(fixtures("entity"))]
async fn update_entity_rejects_empty_name(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/entities/update/1")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "diagram_id": 1,
                        "kind": "person",
                        "name": "",
                        "description": "invalid"
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
    assert_eq!(json["message"], "Invalid Parameter");
}

#[sqlx::test(fixtures("entity"))]
async fn update_entity_returns_not_found_for_missing_id(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/entities/update/999")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "diagram_id": 1,
                        "kind": "person",
                        "name": "Missing Entity",
                        "description": "missing"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "fail");
    assert_eq!(json["message"], "Not Found");
}

#[sqlx::test(fixtures("entity"))]
async fn delete_entity_returns_ok(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/entities/2")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let deleted_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        r#"
            SELECT deleted_at FROM entity WHERE id = $1
        "#,
    )
    .bind(2_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert!(deleted_at.is_some());
}

#[sqlx::test(fixtures("entity"))]
async fn delete_entity_returns_not_found_when_already_deleted(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;

    let first = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/entities/2")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(first.status(), StatusCode::OK);

    let second = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/entities/2")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(second.status(), StatusCode::NOT_FOUND);

    let body = second.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "fail");
    assert_eq!(json["message"], "Not Found");
}

#[sqlx::test(fixtures("entity"))]
async fn delete_entity_rejects_zero_id(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/entities/0")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "fail");
    assert_eq!(json["message"], "Invalid Parameter");
}

fn set_test_env() {
    std::env::set_var("JWT_SECRET", "test_secret");
    std::env::set_var("JWT_EXPIRED_IN", "60m");
    std::env::set_var("JWT_MAXAGE", "60");
    std::env::set_var("RUNTIME_MODE", "debug");
}

fn create_token(user_id: &str, secret: &str) -> String {
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

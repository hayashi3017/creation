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

#[sqlx::test(fixtures("world"))]
async fn create_world_normalizes_name_and_blank_description(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/worlds/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "name": "  Created World  ",
                        "description": "   "
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["name"], "Created World");
    assert!(json["data"]["description"].is_null());
    assert!(json["data"]["world_id"].as_u64().unwrap() > 0);
    assert!(json["data"]["created_at"].is_string());
    assert!(json["data"]["updated_at"].is_string());
}

#[sqlx::test(fixtures("world"))]
async fn get_worlds_excludes_deleted_worlds(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/worlds")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let names: Vec<&str> = json["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|world| world["name"].as_str().unwrap())
        .collect();

    assert_eq!(names, vec!["Second World", "Active World"]);
}

#[sqlx::test(fixtures("world"))]
async fn get_world_returns_active_world_and_404_for_deleted(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let active = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/worlds/1")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(active.status(), StatusCode::OK);

    let deleted = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/worlds/3")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(deleted.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("world"))]
async fn update_world_updates_name_and_description(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/worlds/update/1")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "name": "  Updated World  ",
                        "description": "  updated description  "
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let row = sqlx::query("SELECT name, description FROM world WHERE world_id = 1")
        .fetch_one(&db)
        .await
        .unwrap();

    assert_eq!(row.get::<String, _>("name"), "Updated World");
    assert_eq!(
        row.get::<Option<String>, _>("description").as_deref(),
        Some("updated description")
    );
}

#[sqlx::test(fixtures("world"))]
async fn delete_world_soft_deletes_owned_records_and_removes_tree_paths(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/worlds/delete/1")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    for table in [
        "world",
        "diagram",
        "entity",
        "person",
        "relationship",
        "diagram_entity",
    ] {
        let count: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM {} WHERE deleted_at IS NOT NULL",
            table
        ))
        .fetch_one(&db)
        .await
        .unwrap();

        assert!(count > 0, "{} should have soft-deleted rows", table);
    }

    let tree_path_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tree_path")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(tree_path_count, 0);

    let get_deleted = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/worlds/1")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_deleted.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("world"))]
async fn diagram_create_rejects_deleted_world_id(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/diagrams/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "world_id": 3,
                        "name": "Invalid Diagram",
                        "kind": "family_tree"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
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

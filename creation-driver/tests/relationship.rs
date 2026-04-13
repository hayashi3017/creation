mod common;

use std::borrow::BorrowMut;

use axum::body::Body;
use creation_service::model::{relationship::RelationshipKind, user::TokenClaims};
use http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use sqlx::{PgPool, Row};
use tower::ServiceExt;

use crate::common::setup_router;

#[sqlx::test(fixtures("relationship"))]
async fn get_relationships_returns_list(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/relationships")
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
    assert_eq!(json["data"].as_array().unwrap().len(), 2);
    assert_eq!(json["data"][0]["relationship_id"], 1);
    assert!(json["data"][0]["id"].is_null());
    assert_eq!(json["data"][0]["kind"], "parent");
}

#[sqlx::test(fixtures("relationship"))]
async fn get_relationships_returns_not_found_for_soft_deleted_diagram(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/relationships")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"diagram_id":3}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("relationship"))]
async fn create_relationship_rebuilds_tree_path(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/relationships/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "diagram_id": 1,
                        "source_entity_id": 3,
                        "target_entity_id": 6,
                        "kind": "parent",
                        "notes": "created from API"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let relationship_count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*)
            FROM relationship
            WHERE
                diagram_id = $1
                AND source_entity_id = $2
                AND target_entity_id = $3
                AND deleted_at IS NULL
        "#,
    )
    .bind(1_i64)
    .bind(3_i64)
    .bind(6_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(relationship_count, 1);

    let depth: i32 = sqlx::query_scalar(
        r#"
            SELECT depth
            FROM tree_path
            WHERE ancestor_id = $1
                AND descendant_id = $2
        "#,
    )
    .bind(1_i64)
    .bind(6_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(depth, 3);
}

#[sqlx::test(fixtures("relationship"))]
async fn create_relationship_returns_not_found_for_soft_deleted_diagram(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/relationships/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "diagram_id": 3,
                        "source_entity_id": 8,
                        "target_entity_id": 9,
                        "kind": "parent"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("relationship"))]
async fn create_relationship_rejects_cycle_and_rolls_back(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/relationships/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "diagram_id": 1,
                        "source_entity_id": 3,
                        "target_entity_id": 1,
                        "kind": "parent"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let relationship_count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*)
            FROM relationship
            WHERE
                diagram_id = $1
                AND source_entity_id = $2
                AND target_entity_id = $3
                AND deleted_at IS NULL
        "#,
    )
    .bind(1_i64)
    .bind(3_i64)
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(relationship_count, 0);
}

#[sqlx::test(fixtures("relationship"))]
async fn create_relationship_rejects_non_lineage_kind(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/relationships/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "diagram_id": 1,
                        "source_entity_id": 1,
                        "target_entity_id": 2,
                        "kind": "spouse"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("relationship"))]
async fn update_relationship_rebuilds_tree_path(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/relationships/update/2")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "source_entity_id": 2,
                        "target_entity_id": 6,
                        "kind": "parent",
                        "notes": "updated from API"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let updated = sqlx::query(
        r#"
            SELECT target_entity_id, kind, notes
            FROM relationship
            WHERE relationship_id = $1
        "#,
    )
    .bind(2_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(updated.get::<i64, _>("target_entity_id"), 6);
    assert_eq!(
        updated.get::<RelationshipKind, _>("kind"),
        RelationshipKind::Parent
    );
    assert_eq!(
        updated.get::<Option<String>, _>("notes").as_deref(),
        Some("updated from API")
    );

    let removed_count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*)
            FROM tree_path
            WHERE ancestor_id = $1
                AND descendant_id = $2
        "#,
    )
    .bind(1_i64)
    .bind(3_i64)
    .fetch_one(&db)
    .await
    .unwrap();
    let new_depth: i32 = sqlx::query_scalar(
        r#"
            SELECT depth
            FROM tree_path
            WHERE ancestor_id = $1
                AND descendant_id = $2
        "#,
    )
    .bind(1_i64)
    .bind(6_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(removed_count, 0);
    assert_eq!(new_depth, 2);
}

#[sqlx::test(fixtures("relationship"))]
async fn update_relationship_returns_not_found_for_deleted_row(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/relationships/update/4")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "source_entity_id": 1,
                        "target_entity_id": 6,
                        "kind": "parent"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("relationship"))]
async fn update_relationship_returns_not_found_for_soft_deleted_diagram(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/relationships/update/5")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "source_entity_id": 8,
                        "target_entity_id": 9,
                        "kind": "parent"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("relationship"))]
async fn delete_relationship_rebuilds_tree_path(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/relationships/delete/2")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let deleted_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        r#"
            SELECT deleted_at
            FROM relationship
            WHERE relationship_id = $1
        "#,
    )
    .bind(2_i64)
    .fetch_one(&db)
    .await
    .unwrap();
    let removed_count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*)
            FROM tree_path
            WHERE ancestor_id = $1
                AND descendant_id = $2
        "#,
    )
    .bind(1_i64)
    .bind(3_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert!(deleted_at.is_some());
    assert_eq!(removed_count, 0);
}

#[sqlx::test(fixtures("relationship"))]
async fn delete_relationship_returns_not_found_for_deleted_row(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/relationships/delete/4")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("relationship"))]
async fn delete_relationship_returns_not_found_for_soft_deleted_diagram(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/relationships/delete/5")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
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

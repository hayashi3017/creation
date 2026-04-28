mod common;

use std::borrow::BorrowMut;

use axum::body::Body;
use creation_service::model::user::TokenClaims;
use http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::PgPool;
use tower::ServiceExt;

use crate::common::setup_router;

#[sqlx::test(fixtures("genealogy_overview"))]
async fn get_genealogy_overview_merges_visible_world_diagrams(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/genealogy/overview")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"world_id":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["world"]["world_id"], 1);
    assert_eq!(json["data"]["diagram_ids"], serde_json::json!([1, 2]));
    assert_eq!(json["data"]["stats"]["diagram_count"], 2);
    assert_eq!(json["data"]["stats"]["node_count"], 4);
    assert_eq!(json["data"]["stats"]["edge_count"], 4);
    assert_eq!(json["data"]["root_entity_ids"], serde_json::json!([1, 4]));

    let nodes = json["data"]["nodes"].as_array().unwrap();
    let shared = nodes
        .iter()
        .find(|node| node["entity_id"] == 1)
        .expect("shared node");
    assert_eq!(shared["source_diagram_ids"], serde_json::json!([1, 2]));
    assert!(nodes.iter().all(|node| node["entity_id"] != 5));
    assert!(nodes.iter().all(|node| node["entity_id"] != 6));
    assert!(nodes.iter().all(|node| node["entity_id"] != 7));

    let edges = json["data"]["edges"].as_array().unwrap();
    let deduped_parent = edges
        .iter()
        .find(|edge| {
            edge["from_entity_id"] == 1 && edge["to_entity_id"] == 2 && edge["kind"] == "parent"
        })
        .expect("deduped parent edge");
    assert_eq!(
        deduped_parent["source_relationship_ids"],
        serde_json::json!([1, 2])
    );
    assert_eq!(
        deduped_parent["source_diagram_ids"],
        serde_json::json!([1, 2])
    );
    assert!(edges
        .iter()
        .all(|edge| edge["source_diagram_ids"] != serde_json::json!([3])));
    assert!(edges
        .iter()
        .all(|edge| edge["source_diagram_ids"] != serde_json::json!([4])));
}

#[sqlx::test(fixtures("genealogy_overview"))]
async fn get_genealogy_overview_returns_conflict_when_requested_diagrams_are_disabled(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/genealogy/overview")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"world_id":1,"diagram_ids":[3]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CONFLICT);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["message"], "NO_VISIBLE_GENEALOGY_DIAGRAMS");
}

#[sqlx::test(fixtures("genealogy_overview"))]
async fn get_genealogy_overview_applies_as_of_filters(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/genealogy/overview")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"world_id":1,"as_of":"2000-01-01"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let nodes = json["data"]["nodes"].as_array().unwrap();
    let edges = json["data"]["edges"].as_array().unwrap();

    assert!(nodes.iter().all(|node| node["entity_id"] != 3));
    assert!(edges
        .iter()
        .all(|edge| { edge["from_entity_id"] != 2 || edge["to_entity_id"] != 3 }));
    assert!(edges
        .iter()
        .all(|edge| { edge["from_entity_id"] != 4 || edge["to_entity_id"] != 2 }));
}

#[sqlx::test(fixtures("genealogy_overview"))]
async fn get_genealogy_overview_requires_authentication(db: PgPool) {
    set_test_env();

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/genealogy/overview")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"world_id":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
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

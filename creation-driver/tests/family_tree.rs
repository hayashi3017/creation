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

#[sqlx::test(fixtures("family_tree"))]
async fn get_family_tree_returns_normalized_projection(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/family-trees/1")
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
    assert_eq!(json["data"]["diagram"]["diagram_id"], 1);
    assert!(json["data"]["diagram"]["id"].is_null());
    assert_eq!(json["data"]["diagram"]["kind"], "family_tree");
    assert_eq!(json["data"]["root_entity_ids"], serde_json::json!([1, 4]));
    assert_eq!(json["data"]["stats"]["person_count"], 5);
    assert_eq!(json["data"]["stats"]["edge_count"], 3);
    assert_eq!(json["data"]["stats"]["root_count"], 2);

    let nodes = json["data"]["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 5);
    assert_eq!(nodes[0]["entity_id"], 1);
    assert_eq!(nodes[0]["parent_entity_ids"], serde_json::json!([]));
    assert_eq!(nodes[0]["child_entity_ids"], serde_json::json!([2]));
    assert_eq!(nodes[0]["is_root"], true);
    assert_eq!(nodes[1]["entity_id"], 2);
    assert_eq!(nodes[1]["parent_entity_ids"], serde_json::json!([1]));
    assert_eq!(nodes[1]["child_entity_ids"], serde_json::json!([3]));
    assert_eq!(nodes[1]["is_root"], false);
    assert_eq!(nodes[3]["entity_id"], 4);
    assert_eq!(nodes[3]["parent_entity_ids"], serde_json::json!([]));
    assert_eq!(nodes[3]["child_entity_ids"], serde_json::json!([5]));
    assert_eq!(nodes[3]["is_root"], true);

    let edges = json["data"]["edges"].as_array().unwrap();
    assert_eq!(edges.len(), 3);
    assert_eq!(edges[0]["relationship_id"], 1);
    assert_eq!(edges[0]["parent_entity_id"], 1);
    assert_eq!(edges[0]["child_entity_id"], 2);
    assert_eq!(edges[1]["relationship_id"], 2);
    assert_eq!(edges[1]["parent_entity_id"], 2);
    assert_eq!(edges[1]["child_entity_id"], 3);
    assert_eq!(edges[1]["kind"], "parent");
    assert_eq!(edges[2]["relationship_id"], 3);
    assert_eq!(edges[2]["parent_entity_id"], 4);
    assert_eq!(edges[2]["child_entity_id"], 5);

    assert!(nodes.iter().all(|node| node["entity_id"] != 6));
    assert!(edges.iter().all(|edge| edge["relationship_id"] != 4));
    assert!(edges.iter().all(|edge| edge["relationship_id"] != 5));
}

#[sqlx::test(fixtures("family_tree"))]
async fn get_family_tree_returns_bad_request_for_zero_diagram_id(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/family-trees/0")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("family_tree"))]
async fn get_family_tree_returns_bad_request_for_correlation_diagram(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/family-trees/2")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("family_tree"))]
async fn get_family_tree_returns_not_found_for_soft_deleted_diagram(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/family-trees/3")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("family_tree"))]
async fn get_family_tree_requires_authentication(db: PgPool) {
    set_test_env();

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/family-trees/1")
                .body(Body::empty())
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

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
async fn get_genealogy_diagram_returns_normalized_projection(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/genealogy/diagram/1")
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
    assert_eq!(json["data"]["context"]["kind"], "diagram");
    assert_eq!(json["data"]["context"]["diagram_id"], 1);
    assert_eq!(json["data"]["context"]["world_id"], 1);
    assert_eq!(
        json["data"]["context"]["diagram_ids"],
        serde_json::json!([1])
    );
    assert!(json["data"]["as_of"].is_null());
    assert!(json["data"]["center_entity_id"].is_null());
    assert_eq!(json["data"]["root_entity_ids"], serde_json::json!([1, 4]));
    assert_eq!(json["data"]["stats"]["node_count"], 5);
    assert_eq!(json["data"]["stats"]["edge_count"], 4);
    assert_eq!(json["data"]["stats"]["root_count"], 2);
    assert_eq!(json["data"]["stats"]["diagram_count"], 1);

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
    assert_eq!(edges.len(), 4);
    assert_eq!(edges[0]["edge_id"], "diagram:1:edge:1");
    assert_eq!(edges[0]["source_relationship_ids"], serde_json::json!([1]));
    assert_eq!(edges[0]["source_confidence"], "confirmed");
    assert_eq!(edges[0]["source_entity_id"], 1);
    assert_eq!(edges[0]["target_entity_id"], 2);
    assert_eq!(edges[1]["source_relationship_ids"], serde_json::json!([2]));
    assert_eq!(edges[1]["source_entity_id"], 2);
    assert_eq!(edges[1]["target_entity_id"], 3);
    assert_eq!(edges[1]["kind"], "parent");
    assert_eq!(edges[2]["source_relationship_ids"], serde_json::json!([5]));
    assert_eq!(edges[2]["source_entity_id"], 2);
    assert_eq!(edges[2]["target_entity_id"], 5);
    assert_eq!(edges[2]["kind"], "spouse");
    assert_eq!(edges[3]["source_relationship_ids"], serde_json::json!([3]));
    assert_eq!(edges[3]["source_entity_id"], 4);
    assert_eq!(edges[3]["target_entity_id"], 5);

    assert!(nodes.iter().all(|node| node["entity_id"] != 6));
    assert!(edges
        .iter()
        .all(|edge| edge["source_relationship_ids"] != serde_json::json!([4])));
}

#[sqlx::test(fixtures("family_tree"))]
async fn get_genealogy_diagram_applies_as_of_projection(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/genealogy/diagram/1?as_of=1995-01-01")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"]["as_of"], "1995-01-01");
    assert_eq!(json["data"]["root_entity_ids"], serde_json::json!([1, 4]));
    assert_eq!(json["data"]["stats"]["node_count"], 4);
    assert_eq!(json["data"]["stats"]["edge_count"], 3);
    assert_eq!(json["data"]["stats"]["root_count"], 2);

    let nodes = json["data"]["nodes"].as_array().unwrap();
    assert!(nodes.iter().all(|node| node["entity_id"] != 3));
    let other_root_child = nodes
        .iter()
        .find(|node| node["entity_id"] == 5)
        .expect("other root child remains visible before death");
    assert_eq!(
        other_root_child["parent_entity_ids"],
        serde_json::json!([4])
    );

    let edges = json["data"]["edges"].as_array().unwrap();
    assert!(edges
        .iter()
        .all(|edge| edge["source_relationship_ids"] != serde_json::json!([2])));
    assert!(edges
        .iter()
        .any(|edge| edge["source_relationship_ids"] == serde_json::json!([3])));
    assert!(edges
        .iter()
        .any(|edge| edge["source_relationship_ids"] == serde_json::json!([5])));
}

#[sqlx::test(fixtures("family_tree"))]
async fn get_genealogy_diagram_applies_center_metadata(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/genealogy/diagram/1?center_entity_id=2&ancestor_depth=1&descendant_depth=1")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let node_ids = json["data"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["entity_id"].as_i64().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(node_ids, vec![1, 2, 3, 4, 5]);
    assert_eq!(json["data"]["root_entity_ids"], serde_json::json!([1, 4]));
    assert_eq!(json["data"]["center_entity_id"], 2);
    assert_eq!(json["data"]["stats"]["node_count"], 5);
    assert_eq!(json["data"]["stats"]["edge_count"], 4);
    assert_eq!(json["data"]["stats"]["root_count"], 2);

    let nodes = json["data"]["nodes"].as_array().unwrap();
    let center = nodes
        .iter()
        .find(|node| node["entity_id"] == serde_json::json!(2))
        .unwrap();
    assert_eq!(center["relation_to_center"], "self");
    assert_eq!(center["relation_path_to_center"], serde_json::json!([]));
    assert_eq!(center["generation_offset_from_center"], 0);

    let parent = nodes
        .iter()
        .find(|node| node["entity_id"] == serde_json::json!(1))
        .unwrap();
    assert_eq!(parent["relation_to_center"], "parent");
    assert_eq!(parent["generation_offset_from_center"], -1);
    assert_eq!(
        parent["relation_path_to_center"].as_array().unwrap().len(),
        1
    );

    let in_law = nodes
        .iter()
        .find(|node| node["entity_id"] == serde_json::json!(4))
        .unwrap();
    assert_eq!(in_law["relation_to_center"], "in_law");
    assert_eq!(in_law["generation_offset_from_center"], -1);
    assert_eq!(
        in_law["relation_path_to_center"].as_array().unwrap().len(),
        2
    );

    let spouse = nodes
        .iter()
        .find(|node| node["entity_id"] == serde_json::json!(5))
        .unwrap();
    assert_eq!(spouse["relation_to_center"], "spouse");
    assert_eq!(spouse["generation_offset_from_center"], 0);

    let child = nodes
        .iter()
        .find(|node| node["entity_id"] == serde_json::json!(3))
        .unwrap();
    assert_eq!(child["relation_to_center"], "child");
    assert_eq!(child["generation_offset_from_center"], 1);
    assert_eq!(
        child["relation_path_to_center"].as_array().unwrap().len(),
        1
    );

    let edges = json["data"]["edges"].as_array().unwrap();
    assert!(edges
        .iter()
        .any(|edge| edge["source_relationship_ids"] == serde_json::json!([1])));
    assert!(edges
        .iter()
        .any(|edge| edge["source_relationship_ids"] == serde_json::json!([2])));
    assert!(edges
        .iter()
        .any(|edge| edge["source_relationship_ids"] == serde_json::json!([5])));
}

#[sqlx::test(fixtures("family_tree"))]
async fn get_genealogy_diagram_returns_bad_request_for_zero_diagram_id(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/genealogy/diagram/0")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("family_tree"))]
async fn get_genealogy_diagram_returns_bad_request_for_correlation_diagram(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/genealogy/diagram/2")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("family_tree"))]
async fn get_genealogy_diagram_returns_not_found_for_soft_deleted_diagram(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/genealogy/diagram/3")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("family_tree"))]
async fn get_genealogy_diagram_requires_authentication(db: PgPool) {
    set_test_env();

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/genealogy/diagram/1")
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

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

#[sqlx::test(fixtures("genealogy_world"))]
async fn get_genealogy_world_merges_visible_world_diagrams(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/genealogy/world/1")
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
    assert_eq!(json["data"]["context"]["kind"], "world");
    assert_eq!(json["data"]["context"]["world_id"], 1);
    assert_eq!(
        json["data"]["context"]["diagram_ids"],
        serde_json::json!([1, 2])
    );
    assert_eq!(json["data"]["stats"]["diagram_count"], 2);
    assert_eq!(json["data"]["stats"]["node_count"], 4);
    assert_eq!(json["data"]["stats"]["edge_count"], 5);
    assert_eq!(json["data"]["root_entity_ids"], serde_json::json!([1]));

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
            edge["source_entity_id"] == 1
                && edge["target_entity_id"] == 2
                && edge["kind"] == "parent"
        })
        .expect("deduped parent edge");
    assert_eq!(
        deduped_parent["source_relationship_ids"],
        serde_json::json!([1])
    );
    assert_eq!(deduped_parent["edge_id"], "world:1:edge:1");
    assert_eq!(deduped_parent["source_confidence"], "confirmed");
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

#[sqlx::test(fixtures("genealogy_world"))]
async fn get_genealogy_world_applies_center_depth_filters(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(
                    "/api/genealogy/world/1?center_entity_id=3&ancestor_depth=1&descendant_depth=1",
                )
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

    assert_eq!(node_ids, vec![2, 3]);
    assert_eq!(json["data"]["stats"]["node_count"], 2);
    assert_eq!(json["data"]["stats"]["edge_count"], 1);
    assert_eq!(json["data"]["root_entity_ids"], serde_json::json!([2]));

    let nodes = json["data"]["nodes"].as_array().unwrap();
    let center = nodes
        .iter()
        .find(|node| node["entity_id"] == serde_json::json!(3))
        .unwrap();
    assert_eq!(center["relation_to_center"], "self");
    assert_eq!(center["relation_path_to_center"], serde_json::json!([]));
    assert_eq!(center["generation_offset_from_center"], 0);

    let parent = nodes
        .iter()
        .find(|node| node["entity_id"] == serde_json::json!(2))
        .unwrap();
    assert_eq!(parent["relation_to_center"], "parent");
    assert_eq!(parent["generation_offset_from_center"], -1);
    assert_eq!(
        parent["relation_path_to_center"].as_array().unwrap().len(),
        1
    );
}

#[sqlx::test(fixtures("genealogy_world"))]
async fn get_genealogy_world_returns_conflict_when_requested_diagrams_are_disabled(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/genealogy/world/1?diagram_ids=3")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CONFLICT);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["message"], "NO_VISIBLE_GENEALOGY_DIAGRAMS");
}

#[sqlx::test(fixtures("genealogy_world"))]
async fn get_genealogy_world_applies_as_of_filters(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/genealogy/world/1?as_of=2000-01-01")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
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
        .all(|edge| { edge["source_entity_id"] != 2 || edge["target_entity_id"] != 3 }));
    assert!(edges
        .iter()
        .all(|edge| { edge["source_entity_id"] != 4 || edge["target_entity_id"] != 2 }));
}

#[sqlx::test(fixtures("genealogy_world"))]
async fn get_genealogy_world_requires_authentication(db: PgPool) {
    set_test_env();

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/genealogy/world/1")
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

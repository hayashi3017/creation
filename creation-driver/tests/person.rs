mod common;

use std::borrow::BorrowMut;

use axum::body::Body;
use creation_service::{model::person::GenderKind, model::user::TokenClaims};
use http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use sqlx::{PgPool, Row};
use tower::ServiceExt;

use crate::common::setup_router;

#[sqlx::test(fixtures("person"))]
async fn get_persons_returns_list(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/persons")
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
    assert_eq!(json["data"][0]["name"], "Test Person 1");
}

#[sqlx::test(fixtures("person"))]
async fn get_persons_rejects_zero_diagram_id(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/persons")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"diagram_id":0}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("person"))]
async fn create_person_returns_ok(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/persons/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "world_id": 1,
                        "diagram_id": 1,
                        "name": "Created Person",
                        "description": "created from test",
                        "gender": "male",
                        "birth_date": "2001-01-01",
                        "birthplace": "Yokohama",
                        "residence": "Kobe",
                        "photo_url": "https://example.com/create.png"
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
            SELECT de.diagram_id, e.name, e.description, p.gender, p.birthplace
            FROM entity AS e
            INNER JOIN diagram_entity AS de
                ON de.entity_id = e.entity_id
            INNER JOIN person AS p ON p.entity_id = e.entity_id
            WHERE e.name = $1
        "#,
    )
    .bind("Created Person")
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(row.get::<i64, _>("diagram_id"), 1);
    assert_eq!(
        row.get::<Option<String>, _>("description").as_deref(),
        Some("created from test")
    );
    assert_eq!(row.get::<GenderKind, _>("gender"), GenderKind::Male);
    assert_eq!(
        row.get::<Option<String>, _>("birthplace").as_deref(),
        Some("Yokohama")
    );
}

#[sqlx::test(fixtures("person"))]
async fn create_person_without_diagram_creates_world_person_only(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/persons/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "world_id": 1,
                        "name": "World Only Person"
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
            SELECT e.world_id, p.entity_id
            FROM entity AS e
            INNER JOIN person AS p ON p.entity_id = e.entity_id
            WHERE e.name = $1
        "#,
    )
    .bind("World Only Person")
    .fetch_one(&db)
    .await
    .unwrap();

    let entity_id = row.get::<i64, _>("entity_id");
    assert_eq!(row.get::<i64, _>("world_id"), 1);

    let membership_count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*)
            FROM diagram_entity
            WHERE entity_id = $1
        "#,
    )
    .bind(entity_id)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(membership_count, 0);
}

#[sqlx::test(fixtures("person"))]
async fn create_diagram_entity_membership_adds_existing_person_to_diagram(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/diagrams/entities/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "diagram_id": 1,
                        "entity_id": 3
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
            SELECT COUNT(*)
            FROM diagram_entity
            WHERE diagram_id = $1 AND entity_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(1_i64)
    .bind(3_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(count, 1);
}

#[sqlx::test(fixtures("person"))]
async fn create_person_normalizes_name_description_and_blank_optional_fields(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/persons/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "world_id": 1,
                        "diagram_id": 1,
                        "name": "  Normalized Person  ",
                        "description": "   ",
                        "birthplace": "  ",
                        "residence": "",
                        "photo_url": " "
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
            SELECT e.name, e.description, p.birthplace, p.residence, p.photo_url
            FROM entity AS e
            INNER JOIN person AS p ON p.entity_id = e.entity_id
            WHERE e.name = $1
        "#,
    )
    .bind("Normalized Person")
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(row.get::<String, _>("name"), "Normalized Person");
    assert!(row.get::<Option<String>, _>("description").is_none());
    assert!(row.get::<Option<String>, _>("birthplace").is_none());
    assert!(row.get::<Option<String>, _>("residence").is_none());
    assert!(row.get::<Option<String>, _>("photo_url").is_none());
}

#[sqlx::test(fixtures("person"))]
async fn create_person_rejects_empty_name(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/persons/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "world_id": 1,
                        "diagram_id": 1,
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
}

#[sqlx::test(fixtures("person"))]
async fn create_person_rejects_too_long_birthplace(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");
    let too_long_birthplace = "a".repeat(256);

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/persons/create")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "world_id": 1,
                        "diagram_id": 1,
                        "name": "Too Long Birthplace",
                        "birthplace": too_long_birthplace
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("person"))]
async fn update_person_returns_ok(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/persons/update/1")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "name": "Updated Person",
                        "description": "updated from API",
                        "gender": "unknown",
                        "birth_date": "1996-04-01",
                        "death_date": "2024-04-01",
                        "birthplace": "Fukuoka",
                        "residence": "Sendai",
                        "photo_url": "https://example.com/update.png"
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
            SELECT de.diagram_id, e.name, e.description, p.gender, p.death_date, p.birthplace, p.residence
            FROM entity AS e
            INNER JOIN diagram_entity AS de
                ON de.entity_id = e.entity_id
            INNER JOIN person AS p ON p.entity_id = e.entity_id
            WHERE e.entity_id = $1
        "#,
    )
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(row.get::<i64, _>("diagram_id"), 1);
    assert_eq!(row.get::<String, _>("name"), "Updated Person");
    assert_eq!(row.get::<GenderKind, _>("gender"), GenderKind::Unknown);
    assert_eq!(
        row.get::<Option<chrono::NaiveDate>, _>("death_date"),
        Some("2024-04-01".parse().unwrap())
    );
}

#[sqlx::test(fixtures("person"))]
async fn update_person_rejects_empty_name(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/persons/update/1")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "name": ""
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("person"))]
async fn update_person_returns_not_found_for_deleted_entity(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri("/api/persons/update/4")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "name": "Missing Person"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("person"))]
async fn delete_person_returns_ok(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/persons/delete/2")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let row = sqlx::query(
        r#"
            SELECT e.deleted_at AS entity_deleted_at, p.deleted_at AS person_deleted_at
            FROM entity AS e
            INNER JOIN person AS p ON p.entity_id = e.entity_id
            WHERE e.entity_id = $1
        "#,
    )
    .bind(2_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert!(row
        .get::<Option<chrono::DateTime<chrono::Utc>>, _>("entity_deleted_at")
        .is_some());
    assert!(row
        .get::<Option<chrono::DateTime<chrono::Utc>>, _>("person_deleted_at")
        .is_some());

    let relationship_deleted_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        r#"
            SELECT deleted_at
            FROM relationship
            WHERE relationship_id = $1
        "#,
    )
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert!(relationship_deleted_at.is_some());

    let tree_path_count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*)
            FROM tree_path
            WHERE ancestor_id = $1 OR descendant_id = $1
        "#,
    )
    .bind(2_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(tree_path_count, 0);
}

#[sqlx::test(fixtures("person"))]
async fn delete_person_returns_not_found_for_deleted_person_row(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/persons/delete/5")
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

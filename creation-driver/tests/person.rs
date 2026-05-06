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
                .uri("/api/persons?world_id=1")
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
    assert!(json["data"].is_array());
    assert_eq!(json["data"].as_array().unwrap().len(), 3);
    assert_eq!(json["data"][0]["name"], "Test Person 1");
    assert!(json["data"][0]["diagram_id"].is_null());
    assert_eq!(json["data"][0]["diagram_ids"], serde_json::json!([1]));
    assert_eq!(json["data"][2]["name"], "Other Diagram Person");
    assert_eq!(json["data"][2]["diagram_ids"], serde_json::json!([2]));
}

#[sqlx::test(fixtures("person"))]
async fn get_persons_rejects_missing_world_id(db: PgPool) {
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
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("person"))]
async fn get_persons_rejects_zero_world_id(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/persons?world_id=0")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
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
                        "diagram_ids": [1, 2],
                        "name": "Created Person",
                        "description": "created from test",
                        "first_name": "Created",
                        "last_name": "Person",
                        "first_name_kana": "CreatedKana",
                        "last_name_romaji": "Person",
                        "gender": "male",
                        "birth_date": "2001-01-01",
                        "birthplace": "Yokohama",
                        "deathplace": "Naha",
                        "residence": "Kobe",
                        "photo_url": "https://example.com/create.png",
                        "profile_text": "created profile"
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
            SELECT e.entity_id, e.name, e.description, p.first_name, p.last_name, p.first_name_kana, p.last_name_romaji, p.gender, p.birthplace, p.deathplace, p.profile_text
            FROM entity AS e
            INNER JOIN person AS p ON p.entity_id = e.entity_id
            WHERE e.name = $1
        "#,
    )
    .bind("Created Person")
    .fetch_one(&db)
    .await
    .unwrap();

    let entity_id = row.get::<i64, _>("entity_id");
    let diagram_ids = sqlx::query_scalar::<_, i64>(
        r#"
            SELECT diagram_id
            FROM diagram_entity
            WHERE entity_id = $1 AND deleted_at IS NULL
            ORDER BY diagram_id
        "#,
    )
    .bind(entity_id)
    .fetch_all(&db)
    .await
    .unwrap();

    assert_eq!(diagram_ids, vec![1, 2]);
    assert_eq!(
        row.get::<Option<String>, _>("description").as_deref(),
        Some("created from test")
    );
    assert_eq!(
        row.get::<Option<String>, _>("first_name").as_deref(),
        Some("Created")
    );
    assert_eq!(
        row.get::<Option<String>, _>("last_name").as_deref(),
        Some("Person")
    );
    assert_eq!(
        row.get::<Option<String>, _>("first_name_kana").as_deref(),
        Some("CreatedKana")
    );
    assert_eq!(
        row.get::<Option<String>, _>("last_name_romaji").as_deref(),
        Some("Person")
    );
    assert_eq!(row.get::<GenderKind, _>("gender"), GenderKind::Male);
    assert_eq!(
        row.get::<Option<String>, _>("birthplace").as_deref(),
        Some("Yokohama")
    );
    assert_eq!(
        row.get::<Option<String>, _>("deathplace").as_deref(),
        Some("Naha")
    );
    assert_eq!(
        row.get::<Option<String>, _>("profile_text").as_deref(),
        Some("created profile")
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
async fn sync_diagram_entity_memberships_links_entities_to_diagram(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/diagrams/2/entities/sync")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "entity_ids": [1, 2]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let entity_ids = sqlx::query_scalar::<_, i64>(
        r#"
            SELECT entity_id
            FROM diagram_entity
            WHERE diagram_id = $1 AND deleted_at IS NULL
            ORDER BY entity_id
        "#,
    )
    .bind(2_i64)
    .fetch_all(&db)
    .await
    .unwrap();

    assert_eq!(entity_ids, vec![1, 2, 3]);
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
                        "diagram_ids": [1],
                        "name": "  Normalized Person  ",
                        "description": "   ",
                        "first_name": "  Normalized  ",
                        "last_name": " ",
                        "birthplace": "  ",
                        "deathplace": "",
                        "residence": "",
                        "photo_url": " ",
                        "profile_text": "  "
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
            SELECT e.name, e.description, p.first_name, p.last_name, p.birthplace, p.deathplace, p.residence, p.photo_url, p.profile_text
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
    assert_eq!(
        row.get::<Option<String>, _>("first_name").as_deref(),
        Some("Normalized")
    );
    assert!(row.get::<Option<String>, _>("last_name").is_none());
    assert!(row.get::<Option<String>, _>("birthplace").is_none());
    assert!(row.get::<Option<String>, _>("deathplace").is_none());
    assert!(row.get::<Option<String>, _>("residence").is_none());
    assert!(row.get::<Option<String>, _>("photo_url").is_none());
    assert!(row.get::<Option<String>, _>("profile_text").is_none());
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
                        "diagram_ids": [1],
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
                        "diagram_ids": [1],
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
                        "world_id": 1,
                        "diagram_ids": [2],
                        "name": "Updated Person",
                        "description": "updated from API",
                        "first_name": "Updated",
                        "last_name": "Person",
                        "first_name_romaji": "Updated",
                        "last_name_romaji": "Person",
                        "gender": "unknown",
                        "birth_date": "1996-04-01",
                        "death_date": "2024-04-01",
                        "birthplace": "Fukuoka",
                        "deathplace": "Nagasaki",
                        "residence": "Sendai",
                        "photo_url": "https://example.com/update.png",
                        "profile_text": "updated profile"
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
            SELECT e.name, e.description, p.first_name, p.last_name, p.first_name_romaji, p.last_name_romaji, p.gender, p.death_date, p.birthplace, p.deathplace, p.residence, p.profile_text
            FROM entity AS e
            INNER JOIN person AS p ON p.entity_id = e.entity_id
            WHERE e.entity_id = $1
        "#,
    )
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    let diagram_ids = sqlx::query_scalar::<_, i64>(
        r#"
            SELECT diagram_id
            FROM diagram_entity
            WHERE entity_id = $1 AND deleted_at IS NULL
            ORDER BY diagram_id
        "#,
    )
    .bind(1_i64)
    .fetch_all(&db)
    .await
    .unwrap();

    assert_eq!(diagram_ids, vec![2]);
    assert_eq!(row.get::<String, _>("name"), "Updated Person");
    assert_eq!(
        row.get::<Option<String>, _>("first_name").as_deref(),
        Some("Updated")
    );
    assert_eq!(
        row.get::<Option<String>, _>("last_name").as_deref(),
        Some("Person")
    );
    assert_eq!(
        row.get::<Option<String>, _>("first_name_romaji").as_deref(),
        Some("Updated")
    );
    assert_eq!(row.get::<GenderKind, _>("gender"), GenderKind::Unknown);
    assert_eq!(
        row.get::<Option<chrono::NaiveDate>, _>("death_date"),
        Some("2024-04-01".parse().unwrap())
    );
    assert_eq!(
        row.get::<Option<String>, _>("deathplace").as_deref(),
        Some("Nagasaki")
    );
    assert_eq!(
        row.get::<Option<String>, _>("profile_text").as_deref(),
        Some("updated profile")
    );
}

#[sqlx::test(fixtures("person"))]
async fn update_person_with_empty_diagram_ids_removes_all_memberships(db: PgPool) {
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
                        "world_id": 1,
                        "diagram_ids": [],
                        "name": "Unlinked Person"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let membership_count: i64 = sqlx::query_scalar(
        r#"
            SELECT COUNT(*)
            FROM diagram_entity
            WHERE entity_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(1_i64)
    .fetch_one(&db)
    .await
    .unwrap();

    assert_eq!(membership_count, 0);
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
                        "world_id": 1,
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
                        "world_id": 1,
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
async fn update_person_returns_not_found_for_wrong_world(db: PgPool) {
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
                        "world_id": 999,
                        "name": "Wrong World Person"
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
                .uri("/api/persons/delete/2?world_id=1")
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
async fn delete_person_returns_not_found_for_wrong_world(db: PgPool) {
    set_test_env();
    let token = create_token("00000000-0000-0000-0000-000000000001", "test_secret");

    let mut router = setup_router(db.clone()).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/persons/delete/2?world_id=999")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

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
        .is_none());
    assert!(row
        .get::<Option<chrono::DateTime<chrono::Utc>>, _>("person_deleted_at")
        .is_none());
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
                .uri("/api/persons/delete/5?world_id=1")
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

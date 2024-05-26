mod common;

use std::borrow::BorrowMut;

use crate::common::setup_router;
use axum::{body::Body, http::Request};
use http::{Method, StatusCode};
use serde_json::json;
use sqlx::MySqlPool;
use tower::ServiceExt;

#[sqlx::test(fixtures("user"))]
async fn regist_new_user(db: MySqlPool) {
    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "email": "test4@example.com",
                        "name": "test4_user",
                        "password": "test4_password",
                        "passwordConfirm": "test4_password",
                        "photo": "default.png"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("user"))]
async fn duplicate_regist_email(db: MySqlPool) {
    let mut router = setup_router(db).await;
    let resp = router
        .borrow_mut()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "email": "test1@example.com",
                        "name": "test1_user",
                        "password": "test1_password",
                        "passwordConfirm": "test1_password",
                        "photo": "default.png"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

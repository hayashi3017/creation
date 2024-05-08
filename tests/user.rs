use std::sync::Arc;

use axum::{body::Body, http::Request};
use backend::{config::Config, route::create_router, AppState};
use http::StatusCode;
use serde_json::json;
use sqlx::MySqlPool;

use tower::ServiceExt;

#[sqlx::test]
async fn duplicate_regist_email(db: MySqlPool) {
    let app_state = AppState {
        db,
        env: Config::init(),
    };
    let router = create_router(Arc::new(app_state));
    let resp = router
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/auth/register")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "email": "test1@example.com",
                        "password": "test1_password",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR)
}

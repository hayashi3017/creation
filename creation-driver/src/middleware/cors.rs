use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn setup_cors(port: &str) -> CorsLayer {
    let allowed_origins = [
        format!("http://localhost:{port}")
            .parse::<HeaderValue>()
            .unwrap(),
        format!("http://127.0.0.1:{port}")
            .parse::<HeaderValue>()
            .unwrap(),
        format!("http://0.0.0.0:{port}")
            .parse::<HeaderValue>()
            .unwrap(),
        "http://localhost:4321".parse::<HeaderValue>().unwrap(),
        "http://127.0.0.1:4321".parse::<HeaderValue>().unwrap(),
    ];

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{
            header::{
                ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_METHODS,
                ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_REQUEST_HEADERS,
                ACCESS_CONTROL_REQUEST_METHOD, ORIGIN,
            },
            HeaderValue, Method, Request, StatusCode,
        },
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    use super::setup_cors;

    #[tokio::test]
    async fn allows_simple_request_from_local_frontend_origin() {
        let app = Router::new()
            .route("/api/healthchecker", get(|| async { StatusCode::OK }))
            .layer(setup_cors("8001"));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/healthchecker")
                    .header(ORIGIN, "http://localhost:4321")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&HeaderValue::from_static("http://localhost:4321"))
        );
        assert_eq!(
            response.headers().get(ACCESS_CONTROL_ALLOW_CREDENTIALS),
            Some(&HeaderValue::from_static("true"))
        );
    }

    #[tokio::test]
    async fn allows_preflight_request_from_local_frontend_origin() {
        let app = Router::new()
            .route("/api/healthchecker", get(|| async { StatusCode::OK }))
            .layer(setup_cors("8001"));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/api/healthchecker")
                    .header(ORIGIN, "http://localhost:4321")
                    .header(ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .header(ACCESS_CONTROL_REQUEST_HEADERS, "authorization,content-type")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&HeaderValue::from_static("http://localhost:4321"))
        );
        assert_eq!(
            response.headers().get(ACCESS_CONTROL_ALLOW_METHODS),
            Some(&HeaderValue::from_static("GET,POST,PATCH,DELETE,OPTIONS"))
        );
    }
}

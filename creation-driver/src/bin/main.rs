use std::sync::Arc;

use creation_driver::{
    config::Config, middleware::cors::setup_cors, route::create_router, utils::get_port, AppModule,
    AppState,
};
use dotenvy::dotenv;

use listenfd::ListenFd;
use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = Config::init();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_line_number(true)
                .with_file(true)
                .json(),
        )
        .init();

    let module = AppModule::new().await;
    let port = get_port(config.runtime_mode);
    let addr = format!("{}:{}", "0.0.0.0", port);
    let cors = setup_cors(&port);

    let app = create_router(Arc::new(AppState {
        driver: module.clone(),
        env: config.clone(),
    }))
    .layer(ServiceBuilder::new().layer(cors));

    let mut listenfd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0) {
        Ok(Some(std_listener)) => {
            println!("✅ Received socket from systemfd!");
            std_listener
                .set_nonblocking(true)
                .expect("Cannot set non-blocking");
            TcpListener::from_std(std_listener).expect("Failed to convert to Tokio TcpListener")
        }
        Ok(None) => {
            println!("⚠️ No socket received. Binding manually to {}", &addr);
            TcpListener::bind(&addr).await.expect("Failed to bind")
        }
        Err(e) => {
            println!("listenfd error: {e:?}, falling back to manual bind");
            TcpListener::bind(&addr).await.expect("Failed to bind")
        }
    };
    println!("listener local addr: {:?}", listener.local_addr());
    println!("🚀 Server started successfully on {}", addr);
    println!(
        "OpenAPI JSON: http://localhost:{}/api-docs/openapi.json",
        port
    );
    println!("Swagger UI: http://localhost:{}/swagger-ui", port);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        })
        .await
        .unwrap();
}

use axum::{Router, routing::get};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use voxnote_server::routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .nest("/api/v1", api_router())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = "0.0.0.0:8080";
    info!("VoxNote server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn api_router() -> Router {
    Router::new()
        .nest("/auth", routes::auth::router())
        .nest("/sync", routes::sync::router())
        .nest("/license", routes::license::router())
        .nest("/models", routes::models::router())
        .nest("/user", routes::users::router())
        .route("/health", get(routes::health::health_check))
}

pub mod docs;
pub mod error;
use axum;
use axum::routing::get;
use axum::Router;

pub fn health_check_routes() -> Router {
    Router::new()
        .route(
            "/actuator/health",
            get(|| async { "OK" }),
        )
}
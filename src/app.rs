use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

use crate::handlers;

/// App state for routers.
#[derive(Clone)]
pub struct AppState {
    /// Application's base URL
    pub base_url: String,

    /// Database pool
    pub pool: SqlitePool,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::index))
        .route("/", post(handlers::shorten))
        .route("/:link", get(handlers::resolve))
        .nest_service("/resources", ServeDir::new("resources/public"))
        .layer(CompressionLayer::new())
        .with_state(state)
    // .fallback(handlers::404)
}

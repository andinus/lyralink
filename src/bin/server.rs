use axum::{
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

use lyralink::{handlers, DB_URL};

#[tokio::main]
async fn main() {
    // connect to the database.
    let pool = SqlitePool::connect(DB_URL)
        .await
        .unwrap_or_else(|_| panic!("connect to sqlite db: {}", DB_URL));

    // define routes & start axum server.
    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/", post(handlers::shorten))
        .route("/:link", get(handlers::resolve))
        .nest_service("/resources", ServeDir::new("resources/public"))
        .layer(CompressionLayer::new())
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

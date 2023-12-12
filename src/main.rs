use axum::{
    routing::{get, post},
    Router,
};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

mod handlers;
mod utils;

const DB_URL: &str = "sqlite://lyralink.db";
const BASE_URL: &str = "https://ll.unfla.me";

#[tokio::main]
async fn main() {
    // create sqlite database if it doesn't exist.
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        Sqlite::create_database(DB_URL)
            .await
            .expect("create sqlite db");
    }

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

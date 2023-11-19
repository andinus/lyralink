use std::net::SocketAddr;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::{
    Sqlite,
    SqlitePool,
    migrate::MigrateDatabase
};

mod controllers;
use crate::controllers::short_url;

const DB_URL: &str = "sqlite://lyralink.db";

#[tokio::main]
async fn main() {
    // create sqlite database if it doesn't exist.
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        Sqlite::create_database(DB_URL)
            .await
            .expect("create sqlite db");
    }

    // connect to the database & run migrations.
    let pool = SqlitePool::connect(DB_URL)
        .await
        .expect(&format!("connect to sqlite db: {}", DB_URL));

    // sqlx::migrate!("migrations")
    //     .run(&pool)
    //     .await
    //     .expect("run db migrations");

    // define routes & start axum server.
    let app = Router::new()
        .route("/", post(short_url::create))
        .route("/:link", get(short_url::resolve))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0 , 0, 1], 8000));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::net::SocketAddr;
use std::str::FromStr;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

pub mod handlers;
pub mod utils;

/// Server for lyralink URL shortener
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address to listen on
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Port to listen on
    #[arg(short, long, default_value_t = 12032)]
    port: u16,

    /// SQLite database URL
    #[arg(long, default_value = "sqlite://lyralink.db")]
    database: String,

    /// Application's base URL
    #[arg(long, default_value = "https://ll.unfla.me")]
    base_url: String,
}

/// App state for routers.
#[derive(Clone)]
pub struct AppState {
    /// Application's base URL
    base_url: String,

    /// Database pool
    pool: SqlitePool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // create sqlite database if it doesn't exist.
    if !Sqlite::database_exists(&args.database)
        .await
        .unwrap_or(false)
    {
        Sqlite::create_database(&args.database)
            .await
            .expect("create sqlite db");
    }

    // connect to the database.
    let pool = SqlitePool::connect(&args.database)
        .await
        .unwrap_or_else(|_| panic!("connect to sqlite db: {}", args.database));

    // run migrations.
    sqlx::migrate!()
        .run(&pool)
        .await
        .unwrap_or_else(|_| panic!("running sqlx migrations: {}", args.database));

    let app_state = AppState {
        base_url: args.base_url,
        pool,
    };

    // define routes & start axum server.
    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/", post(handlers::shorten))
        .route("/:link", get(handlers::resolve))
        .nest_service("/resources", ServeDir::new("resources/public"))
        .layer(CompressionLayer::new())
        .with_state(app_state);

    let addr = SocketAddr::from_str(&format!("{}:{}", args.address, args.port)).unwrap();
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

use clap::Parser;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

mod app;
mod handlers;
mod utils;

use app::{app, AppState};

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

    // bind to port and serve the app.
    let listener = tokio::net::TcpListener::bind(&format!("{}:{}", args.address, args.port))
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    let state = AppState {
        base_url: args.base_url,
        pool,
    };
    axum::serve(listener, app(state)).await.unwrap();
}

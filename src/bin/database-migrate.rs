use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use lyralink::DB_URL;

/// run database migrations, creates the database if it does not exist.
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

    // run migrations.
    sqlx::migrate!()
        .run(&pool)
        .await
        .unwrap_or_else(|_| panic!("running sqlx migrations: {}", DB_URL));

    // close the connection.
    pool.close();

    println!("done!");
}

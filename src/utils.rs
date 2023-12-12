use rand::distributions::{Alphanumeric, DistString};
use sqlx::{Error::RowNotFound, SqlitePool};

/// Generates a unique short link for the given url. It needs the pool
/// connection to check if the generated url is unique.
pub async fn unique_short_url(pool: SqlitePool, original_url: &str) -> Result<String, sqlx::Error> {
    // return short_url if it exists already or create one.
    match sqlx::query!(
        "SELECT short_url FROM lyralink WHERE long_url = $1;",
        original_url
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => return Ok(row.short_url),
        Err(RowNotFound) => {}
        Err(err) => return Err(err),
    }

    // generate an unique short_url, if it exists then we try incrementing
    // the length.
    let mut length = 3;
    let mut short_url;
    loop {
        short_url = Alphanumeric.sample_string(&mut rand::thread_rng(), length);
        if let Err(RowNotFound) =
            sqlx::query!("SELECT id FROM lyralink WHERE short_url = $1;", short_url)
                .fetch_one(&pool)
                .await
        {
            break;
        }

        length += 1;
    }

    match sqlx::query!(
        "INSERT INTO lyralink (short_url, long_url) VALUES ($1, $2);",
        short_url,
        original_url
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(short_url),
        Err(err) => Err(err),
    }
}

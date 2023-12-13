use chrono::{DateTime, Utc};
use rand::distributions::{Alphanumeric, DistString};
use sqlx::{Error::RowNotFound, SqlitePool};

/// Generates a unique short link for the given url. It needs the pool
/// connection to check if the generated url is unique. It returns the short url
/// and the time it was created at, the timezone will be UTC.
pub async fn unique_short_url(
    pool: SqlitePool,
    original_url: &str,
) -> Result<(String, DateTime<Utc>), sqlx::Error> {
    // return short_url if it exists already or create one.
    match sqlx::query!(
        "SELECT short_url, created_at FROM lyralink WHERE long_url = $1;",
        original_url
    )
    .fetch_one(&pool)
    .await
    {
        Ok(row) => return Ok((row.short_url, row.created_at.and_utc())),
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

    let created_at = Utc::now();

    match sqlx::query!(
        "INSERT INTO lyralink (short_url, long_url, created_at) VALUES ($1, $2, $3);",
        short_url,
        original_url,
        created_at
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Ok((short_url, created_at)),
        Err(err) => Err(err),
    }
}

pub mod short_url {
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::{IntoResponse, Redirect, Response},
        Form,
    };
    use rand::distributions::{Alphanumeric, DistString};
    use sqlx::{Error::RowNotFound, SqlitePool};

    #[derive(serde::Deserialize)]
    pub struct FormData {
        url: String,
    }

    /// Handles POST request to create a new short url, it expects "url" in a
    /// formdata.
    pub async fn create(State(pool): State<SqlitePool>, Form(form): Form<FormData>) -> Response {
        // return short_url if it exists already or create one.
        match sqlx::query!(
            "SELECT short_url FROM lyralink WHERE long_url = $1;",
            form.url
        )
        .fetch_one(&pool)
        .await
        {
            Ok(row) => return row.short_url.into_response(),
            Err(RowNotFound) => {}
            Err(err) => panic!("{}", err),
        }

        // generate an unique short_url, if it exists then we try incrementing
        // the length.
        let mut length = 1;
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
            form.url
        )
        .execute(&pool)
        .await
        {
            Ok(_) => short_url.into_response(),
            Err(err) => panic!("{}", err),
        }
    }

    /// Handles GET request to resolve a short url, client is redirected if we
    /// have the original url, if not then 404 is returned.
    pub async fn resolve(State(pool): State<SqlitePool>, Path(link): Path<String>) -> Response {
        match sqlx::query!("SELECT long_url FROM lyralink WHERE short_url = $1;", link)
            .fetch_one(&pool)
            .await
        {
            Ok(row) => Redirect::to(&row.long_url).into_response(),
            Err(RowNotFound) => StatusCode::NOT_FOUND.into_response(),
            Err(err) => panic!("{}", err),
        }
    }
}

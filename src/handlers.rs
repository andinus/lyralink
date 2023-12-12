use askama::Template;
use axum::{
    extract::{Path, State},
    http::{header::HeaderMap, header::ACCEPT, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use sqlx::{Error::RowNotFound, SqlitePool};

use crate::utils::unique_short_url;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "result.html")]
struct ResultTemplate {
    short_url: String,
}

pub async fn index() -> Html<String> {
    let page = IndexTemplate;
    Html(page.render().unwrap())
}

#[derive(serde::Deserialize)]
pub struct FormData {
    url: String,
}

/// Handles POST request to create a new short url, it expects "url" in a
/// formdata.
pub async fn shorten(
    State(pool): State<SqlitePool>,
    headers: HeaderMap,
    Form(form): Form<FormData>,
) -> Response {
    match unique_short_url(pool, &form.url).await {
        Ok(short_url) => {
            if headers
                .get(ACCEPT)
                .map_or(false, |x| x.to_str().unwrap().contains("text/html"))
            {
                let page = ResultTemplate { short_url };
                Html(page.render().unwrap()).into_response()
            } else {
                format!("{}/{}\n", crate::BASE_URL, short_url).into_response()
            }
        }
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

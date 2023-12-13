use askama::Template;
use axum::{
    extract::{Path, State},
    http::{header::HeaderMap, header::ACCEPT, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use chrono::{Duration, DurationRound};
use sqlx::Error::RowNotFound;

use crate::utils::unique_short_url;
use crate::AppState;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    base_url: String,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate {
    base_url: String,
}

#[derive(Template)]
#[template(path = "result.html")]
struct ResultTemplate {
    short_url: String,
    base_url: String,
    original_url: String,
    validity: String,
}

#[derive(Template)]
#[template(path = "result-invalid.html")]
struct InvalidResultTemplate {
    base_url: String,
}

pub async fn index(State(AppState { base_url, .. }): State<AppState>) -> Html<String> {
    let page = IndexTemplate { base_url };
    Html(page.render().unwrap())
}

pub async fn not_found(State(AppState { base_url, .. }): State<AppState>) -> Html<String> {
    let page = NotFoundTemplate { base_url };
    Html(page.render().unwrap())
}

#[derive(serde::Deserialize)]
pub struct FormData {
    url: String,
}

/// Handles POST request to create a new short url, it expects "url" in a
/// formdata.
pub async fn shorten(
    State(AppState { pool, base_url, .. }): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<FormData>,
) -> Response {
    match unique_short_url(pool, &form.url).await {
        Ok((short_url, created_at)) => {
            if headers
                .get(ACCEPT)
                .map_or(false, |x| x.to_str().unwrap().contains("text/html"))
            {
                // validity will be 1 day from when it's created.
                let validity = (created_at + Duration::days(1))
                    .duration_trunc(Duration::seconds(1))
                    .unwrap()
                    .to_string();

                let page = ResultTemplate {
                    short_url,
                    base_url,
                    validity,
                    original_url: form.url,
                };
                Html(page.render().unwrap()).into_response()
            } else {
                format!("{}/{}\n", base_url, short_url).into_response()
            }
        }
        Err(err) => panic!("{}", err),
    }
}

/// Handles GET request to resolve a short url, client is redirected if we
/// have the original url, if not then 404 is returned.
pub async fn resolve(
    State(AppState { pool, base_url, .. }): State<AppState>,
    headers: HeaderMap,
    Path(link): Path<String>,
) -> Response {
    // URLs are valid for 24 hours only.
    match sqlx::query!("SELECT long_url FROM lyralink WHERE short_url = $1;", link)
        .fetch_one(&pool)
        .await
    {
        Ok(row) => Redirect::to(&row.long_url).into_response(),
        Err(RowNotFound) => {
            if headers
                .get(ACCEPT)
                .map_or(false, |x| x.to_str().unwrap().contains("text/html"))
            {
                let page = InvalidResultTemplate { base_url };
                (StatusCode::NOT_FOUND, Html(page.render().unwrap())).into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
        Err(err) => panic!("{}", err),
    }
}

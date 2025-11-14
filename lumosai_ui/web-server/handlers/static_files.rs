use web_assets::files;
use axum::body::Body;
use axum::http::{header, HeaderValue, Response, StatusCode};
use axum::response::IntoResponse;
use axum_extra::routing::TypedPath;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/static/{*path}")]
pub struct StaticFilePath {
    pub path: String,
}

pub async fn static_path(StaticFilePath { path }: StaticFilePath) -> impl IntoResponse {
    let path = format!("/static/{}", path);

    if let Some(data) = files::get(&path) {
        Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static(data.mime_type),
            )
            .header(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000"),
            )
            .body(Body::from(data.content.to_vec()))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}

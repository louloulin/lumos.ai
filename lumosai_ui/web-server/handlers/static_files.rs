use web_assets::files::{StaticFile};
use axum::body::Body;
use axum::http::{header, HeaderValue, Response, StatusCode};
use axum::response::IntoResponse;
use axum_extra::routing::TypedPath;
use serde::Deserialize;
use tokio_util::io::ReaderStream;

#[derive(TypedPath, Deserialize)]
#[typed_path("/static/{*path}")]
pub struct StaticFilePath {
    pub path: String,
}

pub async fn static_path(StaticFilePath { path }: StaticFilePath) -> impl IntoResponse {
    let path = format!("/static/{}", path);
    
    tracing::debug!("Serving static file: {}", path);
    
    let data = StaticFile::get(&path);
    
    if let Some(data) = data {
        match tokio::fs::File::open(&data.file_name).await {
            Ok(file) => {
                let stream = ReaderStream::new(file);
                
                Response::builder()
                    .status(StatusCode::OK)
                    .header(
                        header::CONTENT_TYPE,
                        HeaderValue::from_str(data.mime.as_ref()).unwrap(),
                    )
                    .header(
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=31536000"), // 1 year cache
                    )
                    .body(Body::from_stream(stream))
                    .unwrap()
            }
            Err(e) => {
                tracing::error!("Failed to open static file {}: {}", data.file_name, e);
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap()
            }
        }
    } else {
        tracing::warn!("Static file not found: {}", path);
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}

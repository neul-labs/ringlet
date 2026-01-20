//! Embedded static assets for the web UI.
//!
//! This module uses rust-embed to bundle the Vue.js UI into the binary.

use axum::{
    body::Body,
    extract::Path,
    http::{header, Response, StatusCode},
    response::IntoResponse,
};
use rust_embed::Embed;

/// Embedded UI assets from the clown-ui/dist directory.
#[derive(Embed)]
#[folder = "../../clown-ui/dist"]
struct Assets;

/// Serve a static file from the embedded assets.
pub async fn serve_static(Path(path): Path<String>) -> impl IntoResponse {
    // The path parameter doesn't include "assets/", so we need to add it
    let full_path = format!("assets/{}", path);
    serve_file(&full_path)
}

/// Serve the index.html for SPA routing.
pub async fn serve_index() -> impl IntoResponse {
    serve_file("index.html")
}

/// Serve a file by path, with proper content type.
fn serve_file(path: &str) -> Response<Body> {
    // Try to get the file from embedded assets
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::CACHE_CONTROL, cache_control_for(path))
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => {
            // For SPA routing: if file not found and not an API/asset request,
            // serve index.html
            if !path.starts_with("api/") && !path.contains('.') {
                if let Some(content) = Assets::get("index.html") {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html")
                        .header(header::CACHE_CONTROL, "no-cache")
                        .body(Body::from(content.data.into_owned()))
                        .unwrap();
                }
            }

            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap()
        }
    }
}

/// Determine cache control header based on file type.
fn cache_control_for(path: &str) -> &'static str {
    // Assets with hashes in filename can be cached forever
    if path.starts_with("assets/") && (path.contains("-") || path.contains(".")) {
        "public, max-age=31536000, immutable"
    } else if path.ends_with(".html") {
        "no-cache"
    } else {
        "public, max-age=3600"
    }
}

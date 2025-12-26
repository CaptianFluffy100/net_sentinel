/// Monaco Editor language server for pseudo-code
/// Generates JavaScript code that defines syntax highlighting, autocomplete, and validation

use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};

/// Handler for serving the language server JavaScript
pub async fn language_server_handler() -> impl IntoResponse {
    let js = include_str!("../../public/code-server.js");
    
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/javascript; charset=utf-8"),
        ), (
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("public, max-age=3600"),
        )],
        js,
    )
}

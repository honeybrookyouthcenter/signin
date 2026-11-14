//! All the web routes: the API under `/api`, and the kiosk and admin pages.

pub mod admin;
pub mod kiosk;

use axum::http::{header, HeaderValue, StatusCode, Uri};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::Router;
use rust_embed::RustEmbed;

use crate::app::Shared;

pub fn router(state: Shared) -> Router {
    let api = Router::new()
        .nest("/api/kiosk", kiosk::router())
        .nest("/api/admin", admin::router(state.clone()))
        .layer(middleware::map_response(never_cache));
    api.fallback(web_file).with_state(state)
}

/// The API's answers change all the time, so the browser must always ask
/// again. Without this, the back button could show an old "you are logged in".
async fn never_cache(mut response: Response) -> Response {
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

/// The built web pages (`web/dist`), baked into the program when it is compiled.
#[derive(RustEmbed)]
#[folder = "../web/dist"]
struct WebFiles;

/// What kind of file this is, for the browser.
fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript",
        Some("css") => "text/css",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("webmanifest") => "application/manifest+json",
        _ => "application/octet-stream",
    }
}

async fn web_file(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.starts_with("api/") {
        return (StatusCode::NOT_FOUND, "No such API route.").into_response();
    }

    // "/" is the kiosk page and "/admin" is the admin page.
    let candidates = [path.to_string(), format!("{path}/index.html"), format!("{path}index.html")];
    for candidate in candidates {
        let candidate = candidate.trim_start_matches('/');
        if let Some(file) = WebFiles::get(candidate) {
            // Files under assets/ have a fingerprint in their name, so they can
            // be kept forever. Pages are always checked for a newer version.
            let cache = if candidate.starts_with("assets/") { "public, max-age=31536000, immutable" } else { "no-cache" };
            let headers = [(header::CONTENT_TYPE, content_type(candidate)), (header::CACHE_CONTROL, cache)];
            return (headers, file.data).into_response();
        }
    }
    (StatusCode::NOT_FOUND, "Page not found.").into_response()
}

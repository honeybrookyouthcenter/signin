//! All the web routes: the API under `/api`, and the kiosk and admin pages.

pub mod admin;
pub mod kiosk;

use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::http::{header, StatusCode, Uri};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use rust_embed::RustEmbed;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

use crate::app::Shared;

pub fn router(state: Shared) -> Router {
    Router::new()
        .nest("/api/kiosk", kiosk::router())
        .nest("/api/admin", admin::router(state.clone()))
        .route("/api/events", get(events))
        .fallback(web_file)
        .with_state(state)
}

/// A stream the pages keep open. We send "changed" whenever people or visits
/// change, and the page then reloads its lists. The event says nothing about
/// what changed, so it is safe to send to the kiosk. (It still needs some
/// data: browsers silently drop an event whose data is empty.)
async fn events(State(state): State<Shared>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let changes = BroadcastStream::new(state.changes.subscribe());
    // If a page falls behind it gets an error instead of a message. It still
    // means "something changed", so both become the same event.
    let events = changes.map(|_| Ok(Event::default().event("changed").data("changed")));
    Sse::new(events).keep_alive(KeepAlive::new().interval(Duration::from_secs(20)))
}

/// The built web pages (`web/dist`), baked into the program when it is compiled.
#[derive(RustEmbed)]
#[folder = "../web/dist"]
struct WebFiles;

async fn web_file(State(state): State<Shared>, uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.starts_with("api/") {
        return (StatusCode::NOT_FOUND, "No such API route.").into_response();
    }

    // A `logo.png` next to config.toml replaces the built-in logo.
    if path == "logo.png" {
        if let Ok(logo) = std::fs::read(state.paths.base.join("logo.png")) {
            return ([(header::CONTENT_TYPE, "image/png"), (header::CACHE_CONTROL, "no-cache")], logo).into_response();
        }
    }

    // "/" is the kiosk page and "/admin" is the admin page.
    let candidates = [path.to_string(), format!("{path}/index.html"), format!("{path}index.html")];
    for candidate in candidates {
        let candidate = candidate.trim_start_matches('/');
        if let Some(file) = WebFiles::get(candidate) {
            let mime = mime_guess::from_path(candidate).first_or_octet_stream();
            // Files under assets/ have a fingerprint in their name, so they can
            // be kept forever. Pages are always checked for a newer version.
            let cache = if candidate.starts_with("assets/") { "public, max-age=31536000, immutable" } else { "no-cache" };
            let headers = [(header::CONTENT_TYPE, mime.to_string()), (header::CACHE_CONTROL, cache.to_string())];
            return (headers, file.data).into_response();
        }
    }
    (StatusCode::NOT_FOUND, "Page not found.").into_response()
}

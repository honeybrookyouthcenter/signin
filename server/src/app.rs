//! The state every request handler shares, and the error type they return.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rusqlite::Connection;
use tokio::sync::{broadcast, Notify};

use crate::config::{Config, Paths};
use crate::graph::GraphState;

pub type Shared = Arc<AppState>;

pub struct AppState {
    pub config: Config,
    pub paths: Paths,
    db: Mutex<Connection>,
    /// For calls to Microsoft.
    pub http: reqwest::Client,
    /// A message is sent here whenever people or visits change, so open pages refresh.
    pub changes: broadcast::Sender<()>,
    /// Wakes the Outlook sync task up early.
    pub sync_now: Notify,
    /// Admin session cookies that are logged in.
    pub sessions: Mutex<HashSet<String>>,
    /// Wrong PIN tries in a row, and when the last one was.
    pub wrong_pins: Mutex<(u32, Instant)>,
    pub graph: Mutex<GraphState>,
    /// Held while an Outlook sync runs, so two never run at once.
    pub sync_lock: tokio::sync::Mutex<()>,
    pub kiosk: Mutex<KioskStatus>,
}

/// What the kiosk last told us about itself.
#[derive(Default, Clone)]
pub struct KioskStatus {
    pub last_seen: Option<Instant>,
    /// Sign-ins saved on the tablet that have not reached us yet.
    pub waiting: u32,
}

impl AppState {
    pub fn new(config: Config, paths: Paths, conn: Connection) -> Shared {
        let (changes, _) = broadcast::channel(16);
        Arc::new(AppState {
            config,
            paths,
            db: Mutex::new(conn),
            http: reqwest::Client::new(),
            changes,
            sync_now: Notify::new(),
            sessions: Mutex::new(HashSet::new()),
            wrong_pins: Mutex::new((0, Instant::now())),
            graph: Mutex::new(GraphState::default()),
            sync_lock: tokio::sync::Mutex::new(()),
            kiosk: Mutex::new(KioskStatus::default()),
        })
    }

    /// Locks the database. Keep the lock short, and never hold it across an `.await`.
    pub fn db(&self) -> MutexGuard<'_, Connection> {
        // If another thread panicked while holding the lock, the data is still fine to use.
        self.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Tells every open page that something changed.
    pub fn changed(&self) {
        // An error only means no page is listening.
        let _ = self.changes.send(());
    }
}

/// An error a handler sends back: a status code and a message for the screen.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> ApiError {
        ApiError { status, message: message.into() }
    }
    pub fn bad_request(message: impl Into<String>) -> ApiError {
        ApiError::new(StatusCode::BAD_REQUEST, message)
    }
    pub fn not_found(message: impl Into<String>) -> ApiError {
        ApiError::new(StatusCode::NOT_FOUND, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({ "error": self.message }));
        (self.status, body).into_response()
    }
}

/// Lets handlers use `?` on anything that fails. Unexpected errors are logged
/// and become a plain "500" for the browser.
impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> ApiError {
        tracing::error!("request failed: {error:#}");
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong on the office computer.")
    }
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;

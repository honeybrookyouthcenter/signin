//! The state every request handler shares, and the error type they return.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rusqlite::Connection;
use tokio::sync::Notify;

use crate::config::{Config, Paths};
use crate::graph::GraphState;

pub type Shared = Arc<AppState>;

pub struct AppState {
    pub config: Config,
    pub paths: Paths,
    db: Mutex<Connection>,
    memory: Mutex<Memory>,
    /// For calls to Microsoft.
    pub http: reqwest::Client,
    /// Wakes the Outlook sync task up early.
    pub sync_now: Notify,
    /// When this program started. The kiosk page reloads itself when this
    /// changes, which is how it picks up a new version after an update.
    pub started: String,
}

/// Things that only need remembering while the program runs.
#[derive(Default)]
pub struct Memory {
    /// Admin session cookies that are logged in.
    pub sessions: HashSet<String>,
    /// Wrong PIN tries in a row, and when the last one was.
    pub wrong_pins: u32,
    pub last_wrong_pin: Option<Instant>,
    /// When the kiosk last asked for the list of people.
    pub kiosk_last_seen: Option<Instant>,
    pub outlook: GraphState,
}

impl AppState {
    pub fn new(config: Config, paths: Paths, conn: Connection) -> Shared {
        Arc::new(AppState {
            config,
            paths,
            db: Mutex::new(conn),
            memory: Mutex::new(Memory::default()),
            http: reqwest::Client::new(),
            sync_now: Notify::new(),
            started: crate::db::time_text(crate::db::now()),
        })
    }

    /// Locks the database. Keep the lock short, and never hold it across an `.await`.
    pub fn db(&self) -> MutexGuard<'_, Connection> {
        // If another thread panicked while holding the lock, the data is still fine to use.
        self.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Locks the in-memory state. Same rules as `db()`.
    pub fn memory(&self) -> MutexGuard<'_, Memory> {
        self.memory.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
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

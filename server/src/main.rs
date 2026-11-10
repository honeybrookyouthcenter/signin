//! Youth Center Sign In: one program that serves the kiosk and admin web
//! pages, stores sign-ins in a SQLite database, and syncs the kids list with
//! Outlook contacts.
//!
//! Run it with no arguments to start the server, or
//! `signin import <folder> [--dry-run]` to import the old app's logs.

mod app;
mod backup;
mod config;
mod db;
mod graph;
mod import;
mod notes;
mod routes;
mod waivers;

#[cfg(test)]
mod tests;

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::app::AppState;
use crate::config::{Config, Paths};

#[tokio::main]
async fn main() -> Result<()> {
    let base = config::base_dir();
    let config = Config::load(&base)?;
    let paths = Paths::new(&base, &config);
    std::fs::create_dir_all(&paths.data).with_context(|| format!("could not create {}", paths.data.display()))?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => serve(config, paths).await,
        Some("import") => run_import(&paths, &args[1..]),
        Some(_) => {
            println!("Usage:");
            println!("  signin                              start the server");
            println!("  signin import <folder> [--dry-run]  import the old app's sign-in logs");
            Ok(())
        }
    }
}

async fn serve(config: Config, paths: Paths) -> Result<()> {
    // Log to the screen and to a new file each day in data/logs.
    let log_file = tracing_appender::rolling::daily(paths.logs(), "signin.log");
    let (log_writer, _keep_log_open) = tracing_appender::non_blocking(log_file);
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_ansi(false).with_writer(log_writer))
        .init();

    let conn = db::open(&paths.database())?;
    let address = SocketAddr::from(([0, 0, 0, 0], config.port));
    let state = AppState::new(config, paths, conn);

    tokio::spawn(graph::sync_loop(state.clone()));
    tokio::spawn(backup::housekeeping_loop(state.clone()));

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .with_context(|| format!("could not use port {}. Is the program already running?", address.port()))?;
    tracing::info!("database: {}", state.paths.database().display());
    tracing::info!("kiosk page: http://localhost:{}", address.port());
    tracing::info!("admin page: http://localhost:{}/admin", address.port());

    let router = routes::router(state);
    axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

fn run_import(paths: &Paths, args: &[String]) -> Result<()> {
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    let Some(folder) = args.iter().find(|arg| !arg.starts_with("--")) else {
        bail!("Which folder? Usage: signin import <folder> [--dry-run]");
    };

    let mut conn = db::open(&paths.database())?;
    let report = import::import(&mut conn, &PathBuf::from(folder), dry_run)?;
    report.print();
    Ok(())
}

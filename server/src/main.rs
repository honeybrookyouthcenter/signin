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
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

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
    // Log to a new file each day in data/logs. The window only shows problems,
    // so the welcome note stays in view.
    let log_file = tracing_appender::rolling::daily(paths.logs(), "signin.log");
    let (log_writer, _keep_log_open) = tracing_appender::non_blocking(log_file);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_timer(log_time())
                .with_ansi(false)
                .with_filter(LevelFilter::WARN),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_timer(log_time())
                .with_ansi(false)
                .with_writer(log_writer)
                .with_filter(LevelFilter::INFO),
        )
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
    print_welcome(address.port(), &state.paths.logs());

    let router = routes::router(state);
    axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

/// What the program's window shows, so anyone at the office computer knows
/// what it is and leaves it open.
fn print_welcome(port: u16, logs: &std::path::Path) {
    set_window_title("Youth Center Sign In - please leave open");

    println!(
        r#"
   ____  _               ___
  / ___|(_) __ _ _ __   |_ _|_ __
  \___ \| |/ _` | '_ \   | || '_ \
   ___) | | (_| | | | |  | || | | |
  |____/|_|\__, |_| |_| |___|_| |_|
           |___/
  Honey Brook Youth Center

  Hi! I'm the sign-in program for the tablet at the door.

  Every sign-in comes through me, so please don't close this window.
  Minimize it with the _ button at the top right instead.

  If I do get closed, restart the computer and I'll start again."#
    );
    println!();
    if let Ok(computer) = std::env::var("COMPUTERNAME") {
        println!("  Tablet address:  http://{computer}:{port}");
    }
    println!("  Staff page:      http://localhost:{port}/admin");
    println!();
    println!("  Anything that goes wrong shows up below. More detail is in");
    println!("  {}", logs.display());
    println!();
}

/// Names the window, so it is clear on the taskbar what it is.
#[cfg(windows)]
fn set_window_title(title: &str) {
    let text: Vec<u16> = title.encode_utf16().chain([0]).collect();
    // Windows wants the title as UTF-16 ending in a 0, which `text` is.
    unsafe {
        windows_sys::Win32::System::Console::SetConsoleTitleW(text.as_ptr());
    }
}

#[cfg(not(windows))]
fn set_window_title(_title: &str) {}

/// Log lines show local time, like `2026-11-10 18:42:05`.
fn log_time() -> tracing_subscriber::fmt::time::ChronoLocal {
    tracing_subscriber::fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string())
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

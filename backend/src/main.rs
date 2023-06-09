#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use axum::Router;
use std::net::SocketAddr;
use std::{sync::Arc};
use std::path::{PathBuf};
use tracing::log::warn;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod middlewares;
pub mod routes;
mod store;

// SETUP Constants
const WWW_DIR: &str = "./build";
const SERVER_PORT: &str = "8099";
const SERVER_HOST: &str = "0.0.0.0";
const API_FIXED_SECRET: &str = "123456789";
const TIME_SHEET_DIR: &str = "./data";

#[tokio::main]
async fn main() {
    // start tracing - level set by either RUST_LOG env variable or defaults to debug
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "timesheet_backend=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // configure server from environmental variables
    let (port, host, secret, timesheet_dir, www_dir) = from_env();

    let timesheet_dir = PathBuf::from(timesheet_dir);

    tokio::fs::create_dir_all(&timesheet_dir)
        .await
        .expect("failed to create `timesheet_dir` directory");

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Can not parse address and port");

    tracing::info!("listening on http://{}. (SERVER_HOST:SERVER_PORT). Secret: {} (SERVER_SECRET)", addr, &secret);

    if let Ok(files) = std::fs::read_dir(&timesheet_dir) {
        for file in files {
            if let Ok(file) = file {
                tracing::info!("Found data file: {}", file.file_name().to_str().unwrap_or_default());
            }
        }
    }

    let shared_state = Arc::new(store::Store::new(secret, timesheet_dir));

    // crate::routes::api::fetch_ics(State(shared_state));

    // combine the front and backend into server
    let app = Router::new()
        .merge(routes::front_public_route(&www_dir))
        .merge(routes::backend(shared_state));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

/// Tokio signal handler that will wait for a user to press CTRL+C.
/// We use this in our `Server` method `with_graceful_shutdown`.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Expect shutdown signal handler");
    println!("signal shutdown");
}

// Variables from Environment or default to configure server
// port, host, secret
fn from_env() -> (String, String, String, String, String) {
    (
        std::env::var("SERVER_PORT").ok().unwrap_or_else(|| SERVER_PORT.to_string()),
        std::env::var("SERVER_HOST").ok().unwrap_or_else(|| SERVER_HOST.to_string()),
        std::env::var("SERVER_SECRET").ok().unwrap_or_else(|| { API_FIXED_SECRET.to_string() }),
        std::env::var("TIME_SHEET_DIR").ok().unwrap_or_else(|| { TIME_SHEET_DIR.to_string() }),
        std::env::var("WWW_DIR").ok().unwrap_or_else(|| { WWW_DIR.to_string() }),
    )
}

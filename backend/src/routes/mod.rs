use axum::{
    http::StatusCode,
    middleware,
    routing::{get, post, delete},
    Router,
};
use std::{sync::Arc};
use axum::handler::HandlerWithoutStateExt;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_http::cors::{Any, CorsLayer};

pub mod api;

use crate::{
    middlewares,
    store::{Store},
};

/// Front end to server svelte build bundle, css and index.html from public folder
pub fn front_public_route(html_public_dir: &str) -> Router {
    Router::new()
        .nest_service("/", ServeDir::new(html_public_dir).not_found_service(handle_404.into_service()))
        .layer(TraceLayer::new_for_http())
}

async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

/// Back end server built form various routes that are either public, require auth, or secure login
pub fn backend(
    state: Arc<Store>
) -> Router {
    Router::new()
        .route("/api", get(api::handler))
        .route("/api/settings", get(api::get_settings))
        .route("/api/settings", post(api::set_settings))
        .route("/api/fetch_ics", get(api::fetch_ics_full))
        .route("/api/fetch_ics/:month", get(api::fetch_ics_month))
        .route("/api/fetch_ics/:month/:day", get(api::fetch_ics_month_day))
        .route("/api/timesheets", get(api::list_timesheets))
        .route("/api/timesheets/:date", get(api::get_timesheet))
        .route("/api/timesheets/:date", post(api::set_timesheet))
        .route("/api/timesheets/:date", delete(api::delete_timesheet))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            middlewares::auth,
        ))
        .route_layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_headers(Any)
            .allow_methods(Any))
        .with_state(state)
}

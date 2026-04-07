use axum::{http::StatusCode, routing::get, Router};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{modules, state::AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .merge(modules::auth::router())
        .merge(modules::admin::router())
        .merge(modules::leagues::router())
        .merge(modules::predictions::router())
        .merge(modules::standings::router())
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::OK
}

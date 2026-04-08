use axum::{Router, routing::get};

use crate::state::AppState;

pub(crate) mod db;
mod handlers;
pub mod models;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/leagues/{id}/standings", get(handlers::standings_page))
        .route(
            "/leagues/{id}/standings/leaderboard",
            get(handlers::leaderboard_fragment),
        )
        .route(
            "/leagues/{id}/standings/match/{match_id}",
            get(handlers::match_breakdown),
        )
        .route(
            "/leagues/{id}/standings/compare",
            get(handlers::compare_page),
        )
        .route("/leagues/{id}/fixtures", get(handlers::fixture_list))
        .route(
            "/leagues/{id}/members/{user_id}",
            get(handlers::member_stats),
        )
}

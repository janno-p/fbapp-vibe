use axum::{
    routing::{get, post},
    Router,
};

use crate::{modules::admin::AdminUser, state::AppState};

mod db;
mod handlers;
pub mod models;

pub use db::list_user_leagues;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/leagues", get(handlers::admin_list_leagues))
        .route("/admin/leagues", post(handlers::admin_create_league))
        .route("/leagues/{id}", get(handlers::league_overview))
        .route("/leagues/join/{token}", get(handlers::join_league))
}

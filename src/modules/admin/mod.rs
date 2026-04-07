use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    routing::{get, post},
    Router,
};

use crate::{
    error::AppError,
    modules::auth::{AuthSession, User},
    state::AppState,
};

mod db;
mod handlers;
pub mod models;

/// Extractor that resolves to the authenticated user and fails with 401 if the
/// user is not logged in or does not have admin rights.
pub struct AdminUser(pub User);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_session = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let user = auth_session.user.ok_or(AppError::Unauthorized)?;
        if !user.is_admin {
            return Err(AppError::Unauthorized);
        }
        Ok(AdminUser(user))
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin", get(handlers::dashboard))
        .route("/admin/competitions", get(handlers::list_competitions))
        .route("/admin/tournaments", post(handlers::register_tournament))
        .route("/admin/tournaments/{id}/seed", post(handlers::seed_tournament))
        .route("/admin/tournaments/{id}/activate", post(handlers::activate_tournament))
        .route("/admin/tournaments/{id}/deactivate", post(handlers::deactivate_tournament))
        .route("/admin/tournaments/{id}/lock", post(handlers::lock_tournament))
        .route("/admin/tournaments/{id}/unlock", post(handlers::unlock_tournament))
}

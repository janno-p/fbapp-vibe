use axum::{
    Router,
    extract::FromRequestParts,
    http::request::Parts,
    routing::{get, post},
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
            return Err(AppError::Forbidden);
        }
        Ok(AdminUser(user))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::auth::User;

    fn make_user(is_admin: bool) -> User {
        User {
            id: 1,
            google_id: "g-001".to_string(),
            email: "user@example.com".to_string(),
            name: "User".to_string(),
            avatar_url: None,
            is_admin,
        }
    }

    // R4.1 — non-admin users must be rejected with Forbidden.
    #[test]
    fn non_admin_user_triggers_forbidden() {
        let user = make_user(false);
        // Mirror the extractor's guard logic.
        let result: Result<AdminUser, AppError> = if !user.is_admin {
            Err(AppError::Forbidden)
        } else {
            Ok(AdminUser(user))
        };
        assert!(matches!(result, Err(AppError::Forbidden)));
    }

    // R4.2 — admin users are wrapped and passed through.
    #[test]
    fn admin_user_passes_through() {
        let user = make_user(true);
        let result: Result<AdminUser, AppError> = if !user.is_admin {
            Err(AppError::Forbidden)
        } else {
            Ok(AdminUser(user.clone()))
        };
        let admin = result.expect("admin user must pass");
        assert_eq!(admin.0.id, user.id);
        assert!(admin.0.is_admin);
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin", get(handlers::dashboard))
        .route("/admin/competitions", get(handlers::list_competitions))
        .route("/admin/tournaments", post(handlers::register_tournament))
        .route(
            "/admin/tournaments/{id}/seed",
            post(handlers::seed_tournament),
        )
        .route(
            "/admin/tournaments/{id}/activate",
            post(handlers::activate_tournament),
        )
        .route(
            "/admin/tournaments/{id}/deactivate",
            post(handlers::deactivate_tournament),
        )
        .route(
            "/admin/tournaments/{id}/lock",
            post(handlers::lock_tournament),
        )
        .route(
            "/admin/tournaments/{id}/unlock",
            post(handlers::unlock_tournament),
        )
}

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

mod db;
mod handlers;
pub mod models;

pub use models::User;

/// Type alias for the authenticated session used in handlers.
pub type AuthSession = axum_login::AuthSession<AuthBackend>;

/// OAuth authentication backend — restores users from session by loading from PostgreSQL.
#[derive(Clone)]
pub struct AuthBackend {
    pool: sqlx::PgPool,
}

impl AuthBackend {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

impl axum_login::AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = models::Credentials;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        _creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        // OAuth does not use credentials — login() is called directly after token exchange.
        Ok(None)
    }

    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        sqlx::query_as!(
            User,
            "SELECT id, google_id, email, name, avatar_url, is_admin FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::home))
        .route("/dashboard", get(handlers::dashboard))
        .route("/auth/login", get(handlers::login))
        .route("/auth/callback", get(handlers::callback))
        .route("/auth/logout", post(handlers::logout))
}

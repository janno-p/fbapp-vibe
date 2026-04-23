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

#[cfg(test)]
mod tests {
    use axum_login::AuthnBackend as _;
    use sqlx::PgPool;

    use super::*;

    // R1.5 — get_user() loads a user from the database by id.
    #[sqlx::test(migrations = "./migrations")]
    async fn get_user_returns_user_by_id(pool: PgPool) {
        let created = db::find_or_create_user(&pool, "g-100", "test@example.com", "Test", None)
            .await
            .expect("create user");

        let backend = AuthBackend::new(pool);
        let found = backend.get_user(&created.id).await.expect("get_user");

        assert!(found.is_some(), "get_user must return the user");
        assert_eq!(found.unwrap().email, "test@example.com");
    }

    // R1.5 — get_user() returns None for an id that does not exist.
    #[sqlx::test(migrations = "./migrations")]
    async fn get_user_returns_none_for_unknown_id(pool: PgPool) {
        let backend = AuthBackend::new(pool);
        let found = backend.get_user(&999_999_i64).await.expect("get_user");
        assert!(found.is_none(), "get_user must return None for unknown id");
    }

    // R1.3 — authenticate() always returns Ok(None) for the OAuth backend.
    #[sqlx::test(migrations = "./migrations")]
    async fn authenticate_always_returns_ok_none(pool: PgPool) {
        let backend = AuthBackend::new(pool);
        let result = backend
            .authenticate(models::Credentials)
            .await
            .expect("authenticate");
        assert!(
            result.is_none(),
            "OAuth backend authenticate() must return None"
        );
    }
}

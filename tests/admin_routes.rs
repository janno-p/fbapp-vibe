use axum_login::AuthManagerLayerBuilder;
use axum_test::TestServer;
use fbapp_vibe::{
    config::{Config, OAuthEndpoints},
    football_api::FootballApiClient,
    modules::auth::AuthBackend,
    routes,
    state::{AppState, OAuthClient},
};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use sqlx::PgPool;
use tower_sessions::{Expiry, SessionManagerLayer, cookie::SameSite};
use tower_sessions_sqlx_store::PostgresStore;

fn test_config() -> Config {
    Config {
        database_url: String::new(),
        host: "127.0.0.1".to_string(),
        port: 3000,
        google_client_id: "test-client-id".to_string(),
        google_client_secret: "test-client-secret".to_string(),
        google_redirect_url: "http://localhost:3000/auth/callback".to_string(),
        tls_cert_path: None,
        tls_key_path: None,
        football_api_key: "test-key".to_string(),
        poll_interval_secs: 120,
        poll_interval_live_secs: 30,
        session_duration_hours: 24,
    }
}

fn test_oauth_client() -> OAuthClient {
    BasicClient::new(ClientId::new("test-client-id".to_string()))
        .set_client_secret(ClientSecret::new("test-client-secret".to_string()))
        .set_auth_uri(
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .expect("auth url"),
        )
        .set_token_uri(
            TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).expect("token url"),
        )
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:3000/auth/callback".to_string())
                .expect("redirect url"),
        )
}

fn test_oauth_endpoints() -> OAuthEndpoints {
    OAuthEndpoints {
        auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        token_url: "https://oauth2.googleapis.com/token".to_string(),
        userinfo_url: "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
    }
}

async fn build_test_server(pool: PgPool) -> TestServer {
    let session_store = PostgresStore::new(pool.clone());
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(1)));

    let auth_backend = AuthBackend::new(pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    let football_api = FootballApiClient::new("test-key".to_string()).expect("football api client");
    let state = AppState::new(
        pool,
        test_config(),
        test_oauth_client(),
        test_oauth_endpoints(),
        football_api,
    );
    let app = routes::router(state).layer(auth_layer);

    TestServer::new(app)
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_dashboard_requires_auth(pool: PgPool) {
    let server = build_test_server(pool).await;
    server.get("/admin").await.assert_status_unauthorized();
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_competitions_requires_auth(pool: PgPool) {
    let server = build_test_server(pool).await;
    server
        .get("/admin/competitions")
        .await
        .assert_status_unauthorized();
}

#[sqlx::test(migrations = "./migrations")]
async fn register_tournament_requires_auth(pool: PgPool) {
    let server = build_test_server(pool).await;
    server
        .post("/admin/tournaments")
        .await
        .assert_status_unauthorized();
}

#[sqlx::test(migrations = "./migrations")]
async fn seed_tournament_requires_auth(pool: PgPool) {
    let server = build_test_server(pool).await;
    server
        .post("/admin/tournaments/1/seed")
        .await
        .assert_status_unauthorized();
}

#[sqlx::test(migrations = "./migrations")]
async fn activate_tournament_requires_auth(pool: PgPool) {
    let server = build_test_server(pool).await;
    server
        .post("/admin/tournaments/1/activate")
        .await
        .assert_status_unauthorized();
}

#[sqlx::test(migrations = "./migrations")]
async fn deactivate_tournament_requires_auth(pool: PgPool) {
    let server = build_test_server(pool).await;
    server
        .post("/admin/tournaments/1/deactivate")
        .await
        .assert_status_unauthorized();
}

#[sqlx::test(migrations = "./migrations")]
async fn lock_tournament_requires_auth(pool: PgPool) {
    let server = build_test_server(pool).await;
    server
        .post("/admin/tournaments/1/lock")
        .await
        .assert_status_unauthorized();
}

#[sqlx::test(migrations = "./migrations")]
async fn unlock_tournament_requires_auth(pool: PgPool) {
    let server = build_test_server(pool).await;
    server
        .post("/admin/tournaments/1/unlock")
        .await
        .assert_status_unauthorized();
}

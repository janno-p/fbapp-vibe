use axum::{extract::Path, http::StatusCode, routing::post};
use axum_login::AuthManagerLayerBuilder;
use axum_test::TestServer;
use fbapp_vibe::{
    config::Config,
    football_api::FootballApiClient,
    modules::auth::{AuthBackend, AuthSession, User},
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
        pool.clone(),
        test_config(),
        test_oauth_client(),
        football_api,
    );

    let login_pool = pool.clone();
    let test_login = move |mut auth_session: AuthSession, Path(user_id): Path<i64>| {
        let p = login_pool.clone();
        async move {
            let user: User = sqlx::query_as(
                "SELECT id, google_id, email, name, avatar_url, is_admin \
                 FROM users WHERE id = $1",
            )
            .bind(user_id)
            .fetch_one(&p)
            .await
            .unwrap();
            auth_session.login(&user).await.unwrap();
            StatusCode::OK
        }
    };

    let app = routes::router(state)
        .route("/test-login/{user_id}", post(test_login))
        .layer(auth_layer);

    TestServer::builder().save_cookies().build(app)
}

async fn create_user(pool: &PgPool, google_id: &str, email: &str) -> User {
    sqlx::query_as(
        "INSERT INTO users (google_id, email, name) \
         VALUES ($1, $2, 'Test User') \
         RETURNING id, google_id, email, name, avatar_url, is_admin",
    )
    .bind(google_id)
    .bind(email)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn make_user_admin(pool: &PgPool, user_id: i64) {
    sqlx::query!("UPDATE users SET is_admin = true WHERE id = $1", user_id)
        .execute(pool)
        .await
        .unwrap();
}

// T-007: dashboard requires auth
#[sqlx::test(migrations = "./migrations")]
async fn dashboard_requires_auth(pool: PgPool) {
    let server = build_test_server(pool).await;
    server.get("/dashboard").await.assert_status_unauthorized();
}

// T-008: logout destroys session
#[sqlx::test(migrations = "./migrations")]
async fn logout_destroys_session(pool: PgPool) {
    let user = create_user(&pool, "g-logout", "logout@example.com").await;
    let server = build_test_server(pool).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();
    server.get("/dashboard").await.assert_status_ok();
    server.post("/auth/logout").await.assert_status_see_other();
    server.get("/dashboard").await.assert_status_unauthorized();
}

// T-009: home redirects authenticated user to /dashboard
#[sqlx::test(migrations = "./migrations")]
async fn home_redirects_authenticated_user_to_dashboard(pool: PgPool) {
    let user = create_user(&pool, "g-home", "home@example.com").await;
    let server = build_test_server(pool).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();
    let response = server.get("/").await;
    response.assert_status_see_other();
    assert_eq!(response.header("location"), "/dashboard");
}

// T-010: non-admin gets 403; admin gets through
#[sqlx::test(migrations = "./migrations")]
async fn non_admin_user_gets_403_on_admin_route(pool: PgPool) {
    let user = create_user(&pool, "g-nonadmin", "regular@example.com").await;
    let server = build_test_server(pool).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();
    server.get("/admin").await.assert_status_forbidden();
}

#[sqlx::test(migrations = "./migrations")]
async fn admin_user_can_access_admin_route(pool: PgPool) {
    let user = create_user(&pool, "g-admin", "admin@example.com").await;
    make_user_admin(&pool, user.id).await;
    let server = build_test_server(pool).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();
    let status = server.get("/admin").await.status_code();
    assert_ne!(status, StatusCode::UNAUTHORIZED);
    assert_ne!(status, StatusCode::FORBIDDEN);
}

// T-011: expired session returns 401
#[sqlx::test(migrations = "./migrations")]
async fn expired_session_returns_401(pool: PgPool) {
    let user = create_user(&pool, "g-expired", "expired@example.com").await;
    let server = build_test_server(pool.clone()).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();
    server.get("/dashboard").await.assert_status_ok();
    sqlx::query("UPDATE tower_sessions.session SET expiry_date = NOW() - INTERVAL '1 hour'")
        .execute(&pool)
        .await
        .unwrap();
    server.get("/dashboard").await.assert_status_unauthorized();
}

#[sqlx::test(migrations = "./migrations")]
async fn email_change_invalidates_session(pool: PgPool) {
    let user = create_user(&pool, "g-email-change", "original@example.com").await;
    let server = build_test_server(pool.clone()).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();
    server.get("/dashboard").await.assert_status_ok();
    sqlx::query!(
        "UPDATE users SET email = 'changed@example.com' WHERE id = $1",
        user.id
    )
    .execute(&pool)
    .await
    .unwrap();
    server.get("/dashboard").await.assert_status_unauthorized();
}

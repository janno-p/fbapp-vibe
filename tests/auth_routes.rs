use std::sync::Arc;

use axum::{
    Form, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use axum_login::AuthManagerLayerBuilder;
use axum_test::TestServer;
use fbapp_vibe::{
    config::{Config, OAuthEndpoints},
    football_api::FootballApiClient,
    modules::auth::{AuthBackend, AuthSession, User},
    routes,
    state::{AppState, OAuthClient},
};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::Mutex, task::JoinHandle};
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

fn test_oauth_client(oauth_endpoints: &OAuthEndpoints) -> OAuthClient {
    BasicClient::new(ClientId::new("test-client-id".to_string()))
        .set_client_secret(ClientSecret::new("test-client-secret".to_string()))
        .set_auth_uri(AuthUrl::new(oauth_endpoints.auth_url.clone()).expect("auth url"))
        .set_token_uri(TokenUrl::new(oauth_endpoints.token_url.clone()).expect("token url"))
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
    build_test_server_with_oauth(pool, test_oauth_endpoints()).await
}

async fn build_test_server_with_oauth(pool: PgPool, oauth_endpoints: OAuthEndpoints) -> TestServer {
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
        test_oauth_client(&oauth_endpoints),
        oauth_endpoints,
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

    let seed_post_login_redirect =
        |session: tower_sessions::Session, Query(params): Query<PostLoginRedirectQuery>| async move {
            session
                .insert("post_login_redirect", params.path)
                .await
                .unwrap();
            StatusCode::NO_CONTENT
        };

    let app = routes::router(state)
        .route("/test-login/{user_id}", post(test_login))
        .route(
            "/test-session/post-login-redirect",
            get(seed_post_login_redirect),
        )
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

#[derive(Debug, Deserialize)]
struct PostLoginRedirectQuery {
    path: String,
}

#[derive(Clone, Debug, Deserialize)]
struct AuthorizeRequest {
    client_id: String,
    redirect_uri: String,
    response_type: String,
    scope: String,
    state: String,
    code_challenge: String,
    code_challenge_method: String,
}

#[derive(Clone, Debug, Deserialize)]
struct TokenRequest {
    code: String,
    code_verifier: String,
    grant_type: String,
    redirect_uri: String,
}

#[derive(Clone, Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Clone, Debug, Serialize)]
struct MockUserInfo {
    id: String,
    email: String,
    name: String,
    picture: Option<String>,
}

#[derive(Default)]
struct MockOAuthState {
    authorize_requests: Vec<AuthorizeRequest>,
    token_requests: Vec<TokenRequest>,
}

struct MockOAuthProvider {
    base_url: String,
    state: Arc<Mutex<MockOAuthState>>,
    task: JoinHandle<()>,
}

impl MockOAuthProvider {
    async fn spawn() -> Self {
        let state = Arc::new(Mutex::new(MockOAuthState::default()));
        let app = axum::Router::new()
            .route("/authorize", get(mock_authorize))
            .route("/token", post(mock_token))
            .route("/userinfo", get(mock_userinfo))
            .with_state(state.clone());

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock oauth");
        let addr = listener.local_addr().expect("mock oauth local addr");
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve mock oauth");
        });

        Self {
            base_url: format!("http://{addr}"),
            state,
            task,
        }
    }

    fn endpoints(&self) -> OAuthEndpoints {
        OAuthEndpoints {
            auth_url: format!("{}/authorize", self.base_url),
            token_url: format!("{}/token", self.base_url),
            userinfo_url: format!("{}/userinfo", self.base_url),
        }
    }

    async fn token_request_count(&self) -> usize {
        self.state.lock().await.token_requests.len()
    }

    async fn token_requests(&self) -> Vec<TokenRequest> {
        self.state.lock().await.token_requests.clone()
    }

    async fn authorize_requests(&self) -> Vec<AuthorizeRequest> {
        self.state.lock().await.authorize_requests.clone()
    }
}

impl Drop for MockOAuthProvider {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn mock_authorize(
    State(state): State<Arc<Mutex<MockOAuthState>>>,
    Query(params): Query<AuthorizeRequest>,
) -> StatusCode {
    state.lock().await.authorize_requests.push(params);
    StatusCode::OK
}

async fn mock_token(
    State(state): State<Arc<Mutex<MockOAuthState>>>,
    Form(form): Form<TokenRequest>,
) -> Json<TokenResponse> {
    state.lock().await.token_requests.push(form);
    Json(TokenResponse {
        access_token: "test-access-token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    })
}

async fn mock_userinfo() -> Json<MockUserInfo> {
    Json(MockUserInfo {
        id: "google-oauth-user".to_string(),
        email: "oauth@example.com".to_string(),
        name: "OAuth Test User".to_string(),
        picture: Some("https://example.com/avatar.png".to_string()),
    })
}

fn query_param(url: &Url, key: &str) -> String {
    url.query_pairs()
        .find_map(|(name, value)| (name == key).then(|| value.into_owned()))
        .unwrap_or_else(|| panic!("missing query param: {key}"))
}

fn parse_location_url(header: axum::http::HeaderValue) -> Url {
    Url::parse(header.to_str().expect("location header must be utf-8"))
        .expect("parse auth redirect")
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

#[sqlx::test(migrations = "./migrations")]
async fn auth_login_redirects_to_configured_oauth_provider(pool: PgPool) {
    let provider = MockOAuthProvider::spawn().await;
    let server = build_test_server_with_oauth(pool, provider.endpoints()).await;

    let response = server.get("/auth/login").await;
    response.assert_status_see_other();

    let redirect_url = parse_location_url(response.header("location"));
    let mut actual_auth_url = redirect_url.clone();
    actual_auth_url.set_query(None);
    assert_eq!(actual_auth_url.as_str(), provider.endpoints().auth_url);
    assert_eq!(query_param(&redirect_url, "client_id"), "test-client-id");
    assert_eq!(
        query_param(&redirect_url, "redirect_uri"),
        "http://localhost:3000/auth/callback"
    );
    assert_eq!(query_param(&redirect_url, "response_type"), "code");
    assert_eq!(query_param(&redirect_url, "code_challenge_method"), "S256");

    let scopes = query_param(&redirect_url, "scope");
    assert_eq!(
        scopes.split(' ').collect::<Vec<_>>(),
        vec!["email", "profile"]
    );
    assert!(!query_param(&redirect_url, "state").is_empty());
    assert!(!query_param(&redirect_url, "code_challenge").is_empty());

    let authorize_response = reqwest::get(redirect_url.clone())
        .await
        .expect("request mock authorize endpoint");
    assert_eq!(authorize_response.status(), reqwest::StatusCode::OK);

    let authorize_request = provider
        .authorize_requests()
        .await
        .into_iter()
        .next()
        .expect("authorize request recorded");
    assert_eq!(authorize_request.client_id, "test-client-id");
    assert_eq!(authorize_request.response_type, "code");
    assert_eq!(authorize_request.code_challenge_method, "S256");
    assert_eq!(
        authorize_request.redirect_uri,
        "http://localhost:3000/auth/callback"
    );
    assert_eq!(authorize_request.scope, "email profile");
    assert!(!authorize_request.state.is_empty());
    assert!(!authorize_request.code_challenge.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn auth_callback_creates_session_from_real_oauth_flow(pool: PgPool) {
    let provider = MockOAuthProvider::spawn().await;
    let server = build_test_server_with_oauth(pool.clone(), provider.endpoints()).await;

    let login = server.get("/auth/login").await;
    login.assert_status_see_other();
    let redirect_url = parse_location_url(login.header("location"));
    let state = query_param(&redirect_url, "state");

    let callback = server
        .get(&format!("/auth/callback?code=test-code&state={state}"))
        .await;
    callback.assert_status_see_other();
    assert_eq!(callback.header("location"), "/dashboard");

    server.get("/dashboard").await.assert_status_ok();
    assert_eq!(provider.token_request_count().await, 1);

    let token_request = provider
        .token_requests()
        .await
        .into_iter()
        .next()
        .expect("token request recorded");
    assert_eq!(token_request.code, "test-code");
    assert_eq!(token_request.grant_type, "authorization_code");
    assert_eq!(
        token_request.redirect_uri,
        "http://localhost:3000/auth/callback"
    );
    assert!(!token_request.code_verifier.is_empty());

    let user = sqlx::query_as::<_, User>(
        "SELECT id, google_id, email, name, avatar_url, is_admin FROM users WHERE google_id = $1",
    )
    .bind("google-oauth-user")
    .fetch_one(&pool)
    .await
    .expect("oauth user created");
    assert_eq!(user.email, "oauth@example.com");
    assert_eq!(user.name, "OAuth Test User");
}

#[sqlx::test(migrations = "./migrations")]
async fn auth_callback_uses_safe_post_login_redirect_when_present(pool: PgPool) {
    let provider = MockOAuthProvider::spawn().await;
    let server = build_test_server_with_oauth(pool, provider.endpoints()).await;
    let next = "/leagues/join/test-token";

    let seed_response = server
        .get("/test-session/post-login-redirect?path=%2Fleagues%2Fjoin%2Ftest-token")
        .await;
    assert_eq!(seed_response.status_code(), StatusCode::NO_CONTENT);

    let login = server.get("/auth/login").await;
    login.assert_status_see_other();
    let redirect_url = parse_location_url(login.header("location"));
    let state = query_param(&redirect_url, "state");

    let callback = server
        .get(&format!("/auth/callback?code=test-code&state={state}"))
        .await;
    callback.assert_status_see_other();
    assert_eq!(callback.header("location"), next);
}

#[sqlx::test(migrations = "./migrations")]
async fn auth_callback_consumes_oauth_session_state_after_success(pool: PgPool) {
    let provider = MockOAuthProvider::spawn().await;
    let server = build_test_server_with_oauth(pool, provider.endpoints()).await;

    let login = server.get("/auth/login").await;
    login.assert_status_see_other();
    let redirect_url = parse_location_url(login.header("location"));
    let state = query_param(&redirect_url, "state");
    let callback_path = format!("/auth/callback?code=test-code&state={state}");

    server.get(&callback_path).await.assert_status_see_other();
    assert_eq!(provider.token_request_count().await, 1);

    let second_callback = server.get(&callback_path).await;
    assert_eq!(second_callback.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(provider.token_request_count().await, 1);
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

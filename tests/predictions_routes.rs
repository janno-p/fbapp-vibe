use std::fmt::Write;

use axum::{extract::Path, http::StatusCode, routing::post};
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
use sqlx::PgPool;
use time::OffsetDateTime;
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
        pool.clone(),
        test_config(),
        test_oauth_client(),
        test_oauth_endpoints(),
        football_api,
    );

    let login_pool = pool.clone();
    let test_login = move |mut auth_session: AuthSession, Path(user_id): Path<i64>| {
        let p = login_pool.clone();
        async move {
            let user: User = sqlx::query_as(
                "SELECT id, google_id, email, name, avatar_url, is_admin FROM users WHERE id = $1",
            )
            .bind(user_id)
            .fetch_one(&p)
            .await
            .expect("load login user");

            auth_session.login(&user).await.expect("set auth session");
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
        "INSERT INTO users (google_id, email, name) VALUES ($1, $2, 'Test User') RETURNING id, google_id, email, name, avatar_url, is_admin",
    )
    .bind(google_id)
    .bind(email)
    .fetch_one(pool)
    .await
    .expect("insert user")
}

async fn create_tournament(pool: &PgPool, locked: bool) -> i64 {
    let locked_at = if locked {
        Some(OffsetDateTime::now_utc() - time::Duration::hours(1))
    } else {
        None
    };

    sqlx::query_scalar!(
        r#"
        INSERT INTO tournaments (external_id, name, season, is_active, predictions_locked_at)
        VALUES ('ROUTE-TEST-2026', 'Route Test Cup', '2026', TRUE, $1)
        RETURNING id
        "#,
        locked_at
    )
    .fetch_one(pool)
    .await
    .expect("insert tournament")
}

async fn create_team(pool: &PgPool, tournament_id: i64, idx: usize) -> i64 {
    let ext = format!("TEAM-{idx}");
    sqlx::query_scalar!(
        r#"
        INSERT INTO teams (tournament_id, external_id, name, short_name)
        VALUES ($1, $2, $2, $2)
        RETURNING id
        "#,
        tournament_id,
        ext,
    )
    .fetch_one(pool)
    .await
    .expect("insert team")
}

async fn create_player(pool: &PgPool, tournament_id: i64, team_id: i64, idx: usize) -> i64 {
    let ext = format!("PLAYER-{idx}");
    sqlx::query_scalar!(
        r#"
        INSERT INTO players (tournament_id, team_id, external_id, name)
        VALUES ($1, $2, $3, $3)
        RETURNING id
        "#,
        tournament_id,
        team_id,
        ext,
    )
    .fetch_one(pool)
    .await
    .expect("insert player")
}

fn form_list_param(key: &str, ids: &[i64]) -> String {
    let mut body = String::new();
    for (idx, id) in ids.iter().enumerate() {
        if idx > 0 {
            body.push('&');
        }
        let _ = write!(&mut body, "{key}={id}");
    }
    body
}

#[sqlx::test(migrations = "./migrations")]
async fn knockout_wrong_count_returns_inline_error(pool: PgPool) {
    let user = create_user(&pool, "g-ko-wrong", "ko-wrong@example.com").await;
    let t_id = create_tournament(&pool, false).await;
    let mut team_ids = Vec::new();
    for idx in 0..7 {
        team_ids.push(create_team(&pool, t_id, idx).await);
    }

    let server = build_test_server(pool).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();

    let body = form_list_param("team_ids", &team_ids);
    let response = server
        .post("/predictions/knockout/qf")
        .add_header("content-type", "application/x-www-form-urlencoded")
        .text(body)
        .await;

    response.assert_status_ok();
    assert!(
        response.text().contains("Select exactly 8 teams."),
        "expected inline count message, got: {}",
        response.text()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn knockout_valid_count_returns_saved(pool: PgPool) {
    let user = create_user(&pool, "g-ko-valid", "ko-valid@example.com").await;
    let t_id = create_tournament(&pool, false).await;
    let mut team_ids = Vec::new();
    for idx in 0..8 {
        team_ids.push(create_team(&pool, t_id, idx).await);
    }

    let server = build_test_server(pool).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();

    let body = form_list_param("team_ids", &team_ids);
    let response = server
        .post("/predictions/knockout/qf")
        .add_header("content-type", "application/x-www-form-urlencoded")
        .text(body)
        .await;

    response.assert_status_ok();
    assert!(response.text().contains("Saved"));
}

#[sqlx::test(migrations = "./migrations")]
async fn knockout_invalid_round_returns_400(pool: PgPool) {
    let user = create_user(&pool, "g-ko-round", "ko-round@example.com").await;
    let _ = create_tournament(&pool, false).await;

    let server = build_test_server(pool).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();

    let response = server
        .post("/predictions/knockout/not-a-round")
        .add_header("content-type", "application/x-www-form-urlencoded")
        .text("team_ids=1")
        .await;

    response.assert_status_bad_request();
    assert!(response.text().contains("invalid knockout round"));
}

#[sqlx::test(migrations = "./migrations")]
async fn top_scorer_wrong_count_returns_inline_error(pool: PgPool) {
    let user = create_user(&pool, "g-ts-wrong", "ts-wrong@example.com").await;
    let t_id = create_tournament(&pool, false).await;

    let team = create_team(&pool, t_id, 1).await;
    let p1 = create_player(&pool, t_id, team, 1).await;
    let p2 = create_player(&pool, t_id, team, 2).await;

    let server = build_test_server(pool).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();

    let body = form_list_param("player_ids", &[p1, p2]);
    let response = server
        .post("/predictions/top-scorer")
        .add_header("content-type", "application/x-www-form-urlencoded")
        .text(body)
        .await;

    response.assert_status_ok();
    assert!(response.text().contains("Select exactly 3 players."));
}

#[sqlx::test(migrations = "./migrations")]
async fn top_scorer_valid_count_returns_saved(pool: PgPool) {
    let user = create_user(&pool, "g-ts-valid", "ts-valid@example.com").await;
    let t_id = create_tournament(&pool, false).await;

    let team = create_team(&pool, t_id, 1).await;
    let p1 = create_player(&pool, t_id, team, 1).await;
    let p2 = create_player(&pool, t_id, team, 2).await;
    let p3 = create_player(&pool, t_id, team, 3).await;

    let server = build_test_server(pool).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();

    let body = form_list_param("player_ids", &[p1, p2, p3]);
    let response = server
        .post("/predictions/top-scorer")
        .add_header("content-type", "application/x-www-form-urlencoded")
        .text(body)
        .await;

    response.assert_status_ok();
    assert!(response.text().contains("Saved"));
}

#[sqlx::test(migrations = "./migrations")]
async fn predictions_post_requires_auth(pool: PgPool) {
    let t_id = create_tournament(&pool, false).await;
    let mut team_ids = Vec::new();
    for idx in 0..8 {
        team_ids.push(create_team(&pool, t_id, idx).await);
    }

    let team = create_team(&pool, t_id, 99).await;
    let p1 = create_player(&pool, t_id, team, 1).await;
    let p2 = create_player(&pool, t_id, team, 2).await;
    let p3 = create_player(&pool, t_id, team, 3).await;

    let server = build_test_server(pool).await;

    let knockout_response = server
        .post("/predictions/knockout/qf")
        .add_header("content-type", "application/x-www-form-urlencoded")
        .text(form_list_param("team_ids", &team_ids))
        .await;
    knockout_response.assert_status_unauthorized();

    let top_scorer_response = server
        .post("/predictions/top-scorer")
        .add_header("content-type", "application/x-www-form-urlencoded")
        .text(form_list_param("player_ids", &[p1, p2, p3]))
        .await;
    top_scorer_response.assert_status_unauthorized();
}

#[sqlx::test(migrations = "./migrations")]
async fn predictions_locked_returns_403_for_valid_count_requests(pool: PgPool) {
    let user = create_user(&pool, "g-locked", "locked@example.com").await;
    let t_id = create_tournament(&pool, true).await;

    let mut team_ids = Vec::new();
    for idx in 0..8 {
        team_ids.push(create_team(&pool, t_id, idx).await);
    }

    let team = create_team(&pool, t_id, 99).await;
    let p1 = create_player(&pool, t_id, team, 1).await;
    let p2 = create_player(&pool, t_id, team, 2).await;
    let p3 = create_player(&pool, t_id, team, 3).await;

    let server = build_test_server(pool).await;
    server
        .post(&format!("/test-login/{}", user.id))
        .await
        .assert_status_ok();

    let knockout_response = server
        .post("/predictions/knockout/qf")
        .add_header("content-type", "application/x-www-form-urlencoded")
        .text(form_list_param("team_ids", &team_ids))
        .await;
    knockout_response.assert_status_forbidden();

    let top_scorer_response = server
        .post("/predictions/top-scorer")
        .add_header("content-type", "application/x-www-form-urlencoded")
        .text(form_list_param("player_ids", &[p1, p2, p3]))
        .await;
    top_scorer_response.assert_status_forbidden();
}

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use reqwest::redirect::Policy as RedirectPolicy;
use serde::Deserialize;
use tower_sessions::Session;

use crate::{error::AppError, state::AppState};

use super::{
    db,
    models::{GoogleUserInfo, User},
    AuthSession,
};

// ── Templates ────────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "home/index.html")]
struct HomeTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "dashboard/index.html")]
struct DashboardTemplate {
    user: User,
}

// ── Query params ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /
pub async fn home(auth_session: AuthSession) -> Response {
    if auth_session.user.is_some() {
        Redirect::to("/dashboard").into_response()
    } else {
        HomeTemplate.into_response()
    }
}

/// GET /dashboard
pub async fn dashboard(auth_session: AuthSession) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    Ok(DashboardTemplate { user })
}

/// GET /auth/login
pub async fn login(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_state) = state
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    session
        .insert("csrf_state", csrf_state.secret().clone())
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    session
        .insert("pkce_verifier", pkce_verifier.secret().clone())
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;

    Ok(Redirect::to(auth_url.as_str()))
}

/// GET /auth/callback
pub async fn callback(
    State(state): State<AppState>,
    session: Session,
    mut auth_session: AuthSession,
    Query(params): Query<CallbackParams>,
) -> Result<impl IntoResponse, AppError> {
    // Verify CSRF state
    let stored_state: String = session
        .get("csrf_state")
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?
        .ok_or_else(|| AppError::BadRequest("missing csrf state".to_string()))?;

    if stored_state != params.state {
        return Err(AppError::BadRequest("csrf state mismatch".to_string()));
    }

    let pkce_verifier: String = session
        .get("pkce_verifier")
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?
        .ok_or_else(|| AppError::BadRequest("missing pkce verifier".to_string()))?;

    // Exchange authorisation code for access token
    let http_client = reqwest::Client::builder()
        .redirect(RedirectPolicy::none())
        .build()
        .map_err(|e| AppError::Unexpected(e.into()))?;

    let token = state
        .oauth_client
        .exchange_code(AuthorizationCode::new(params.code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
        .request_async(&http_client)
        .await
        .map_err(|e| AppError::Unexpected(anyhow::anyhow!("token exchange failed: {e}")))?;

    // Fetch user profile from Google
    let user_info: GoogleUserInfo = reqwest::Client::new()
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?
        .json()
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;

    // Find or create user in database
    let user = db::find_or_create_user(
        &state.pool,
        &user_info.id,
        &user_info.email,
        &user_info.name,
        user_info.picture.as_deref(),
    )
    .await?;

    // Create authenticated session
    auth_session
        .login(&user)
        .await
        .map_err(|e| AppError::Unexpected(anyhow::anyhow!("login failed: {e}")))?;

    Ok(Redirect::to("/dashboard"))
}

/// POST /auth/logout
pub async fn logout(mut auth_session: AuthSession) -> Result<impl IntoResponse, AppError> {
    auth_session
        .logout()
        .await
        .map_err(|e| AppError::Unexpected(anyhow::anyhow!("logout failed: {e}")))?;
    Ok(Redirect::to("/"))
}

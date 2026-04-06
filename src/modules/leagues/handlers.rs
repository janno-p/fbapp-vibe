use askama::Template;
use askama_web::WebTemplate;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    Form,
};
use tower_sessions::Session;

use crate::{error::AppError, modules::auth::AuthSession, state::AppState};

use super::{
    db,
    models::{CreateLeagueForm, LeagueWithCount},
    AdminUser,
};

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "admin/leagues.html")]
struct AdminLeaguesTemplate {
    leagues: Vec<LeagueWithCount>,
}

// ── Admin handlers ─────────────────────────────────────────────────────────────

pub async fn admin_list_leagues(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let leagues = db::list_leagues_with_counts(&state.pool).await?;
    Ok(AdminLeaguesTemplate { leagues })
}

pub async fn admin_create_league(
    admin: AdminUser,
    State(state): State<AppState>,
    Form(form): Form<CreateLeagueForm>,
) -> Result<impl IntoResponse, AppError> {
    let invite_token = uuid::Uuid::new_v4().to_string();
    db::create_league(&state.pool, &form.name, &invite_token, admin.0.id).await?;
    Ok(Redirect::to("/admin/leagues"))
}

// ── User handlers ─────────────────────────────────────────────────────────────

/// GET /leagues/join/{token}
///
/// Authenticated users are added to the league immediately.
/// Unauthenticated users are redirected to login; the invite URL is stored in
/// the session so the flow completes after OAuth callback.
pub async fn join_league(
    auth_session: AuthSession,
    State(state): State<AppState>,
    session: Session,
    Path(token): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let league = db::get_league_by_token_or_404(&state.pool, &token).await?;

    let user = match auth_session.user {
        Some(u) => u,
        None => {
            let invite_path = format!("/leagues/join/{token}");
            session
                .insert("post_login_redirect", invite_path)
                .await
                .map_err(|e| AppError::Unexpected(e.into()))?;
            return Ok(Redirect::to("/auth/login").into_response());
        }
    };

    db::join_league(&state.pool, league.id, user.id).await?;
    Ok(Redirect::to("/dashboard").into_response())
}

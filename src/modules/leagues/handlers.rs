use askama::Template;
use askama_web::WebTemplate;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    Form,
};
use tower_sessions::Session;

use crate::{error::AppError, modules::auth::AuthSession, nav::NavContext, state::AppState};

use super::{
    db,
    models::{CreateLeagueForm, LeagueOverview, LeagueWithCount},
    AdminUser,
};

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "admin/leagues.html")]
struct AdminLeaguesTemplate {
    leagues: Vec<LeagueWithCount>,
    nav: NavContext,
}

#[derive(Template, WebTemplate)]
#[template(path = "leagues/overview.html")]
struct LeagueOverviewTemplate {
    overview: LeagueOverview,
    nav: NavContext,
}

// ── Admin handlers ─────────────────────────────────────────────────────────────

pub async fn admin_list_leagues(
    admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let (leagues, nav) = tokio::try_join!(
        db::list_leagues_with_counts(&state.pool),
        crate::nav::load(&state.pool, &admin.0, "admin"),
    )?;
    Ok(AdminLeaguesTemplate { leagues, nav })
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

/// GET /leagues/{id}
pub async fn league_overview(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path(league_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;

    if !db::is_member(&state.pool, league_id, user.id).await? {
        return Err(AppError::Forbidden);
    }

    let (overview_opt, nav) = tokio::try_join!(
        db::get_league_overview(&state.pool, league_id),
        crate::nav::load(&state.pool, &user, "leagues"),
    )?;
    let mut overview = overview_opt.ok_or(AppError::NotFound)?;

    // Only expose the invite token to the league creator or admins.
    if user.id != overview.created_by && !user.is_admin {
        overview.invite_token = None;
    }

    Ok(LeagueOverviewTemplate { overview, nav })
}



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
            if is_safe_redirect(&invite_path) {
                session
                    .insert("post_login_redirect", invite_path)
                    .await
                    .map_err(|e| AppError::Unexpected(e.into()))?;
            }
            return Ok(Redirect::to("/auth/login").into_response());
        }
    };

    db::join_league(&state.pool, league.id, user.id).await?;
    Ok(Redirect::to("/dashboard").into_response())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns `true` only for relative paths that are safe to use as post-login
/// redirects. Rejects absolute URLs, protocol-relative URLs, and values
/// containing newlines (which could enable header injection).
fn is_safe_redirect(url: &str) -> bool {
    url.starts_with('/') && !url.starts_with("//") && !url.contains("://") && !url.contains('\n')
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_relative_paths() {
        assert!(is_safe_redirect("/predictions#knockout"));
        assert!(is_safe_redirect("/leagues/join/abc-123"));
        assert!(is_safe_redirect("/dashboard"));
    }

    #[test]
    fn rejects_absolute_urls() {
        assert!(!is_safe_redirect("https://evil.com"));
        assert!(!is_safe_redirect("http://evil.com"));
    }

    #[test]
    fn rejects_protocol_relative_urls() {
        assert!(!is_safe_redirect("//evil.com"));
    }

    #[test]
    fn rejects_newlines() {
        assert!(!is_safe_redirect("/foo\nbar"));
    }
}

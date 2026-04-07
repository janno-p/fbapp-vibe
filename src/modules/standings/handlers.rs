use askama::Template;
use askama_web::WebTemplate;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{error::AppError, modules::auth::AuthSession, nav::NavContext, state::AppState};

use super::{
    db,
    models::{
        build_leaderboard, group_fixtures, CompareGroupRow, FixtureGroup, LeaderboardEntry,
        LeagueMember, MatchBreakdownRow, MatchInfo, NearestMatch,
    },
};

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "standings/index.html")]
struct StandingsTemplate {
    league_id: i64,
    league_name: String,
    entries: Vec<LeaderboardEntry>,
    nearest: Option<NearestMatch>,
    has_live: bool,
    no_tournament: bool,
    nav: NavContext,
}

#[derive(Template, WebTemplate)]
#[template(path = "standings/leaderboard.html")]
struct LeaderboardFragment {
    league_id: i64,
    entries: Vec<LeaderboardEntry>,
    has_live: bool,
    no_tournament: bool,
}

#[derive(Template, WebTemplate)]
#[template(path = "standings/match.html")]
struct MatchBreakdownTemplate {
    league_name: String,
    match_info: MatchInfo,
    rows: Vec<MatchBreakdownRow>,
    nav: NavContext,
}

#[derive(Template, WebTemplate)]
#[template(path = "standings/compare.html")]
struct CompareTemplate {
    league_id: i64,
    league_name: String,
    members: Vec<LeagueMember>,
    user_a: Option<LeagueMember>,
    user_b: Option<LeagueMember>,
    group_rows: Vec<CompareGroupRow>,
    nav: NavContext,
}

#[derive(Template, WebTemplate)]
#[template(path = "standings/fixtures.html")]
struct FixturesTemplate {
    league_id: i64,
    league_name: String,
    groups: Vec<FixtureGroup>,
    no_tournament: bool,
    nav: NavContext,
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CompareParams {
    pub a: Option<i64>,
    pub b: Option<i64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /leagues/{id}/standings
pub async fn standings_page(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path(league_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    require_member(&state, league_id, user.id).await?;

    let (league_name_opt, nav) = tokio::try_join!(
        db::get_league_name(&state.pool, league_id),
        crate::nav::load(&state.pool, &user, "standings"),
    )?;
    let league_name = league_name_opt.ok_or(AppError::NotFound)?;

    let (entries, nearest, has_live, no_tournament) =
        match db::get_active_tournament_id(&state.pool).await? {
            None => (vec![], None, false, true),
            Some(t_id) => {
                let (raw, nearest, has_live) = tokio::try_join!(
                    db::get_leaderboard(&state.pool, t_id, league_id),
                    db::get_nearest_match(&state.pool, t_id),
                    db::has_live_matches(&state.pool, t_id),
                )?;
                (build_leaderboard(raw), nearest, has_live, false)
            }
        };

    Ok(StandingsTemplate {
        league_id,
        league_name,
        entries,
        nearest,
        has_live,
        no_tournament,
        nav,
    })
}

/// GET /leagues/{id}/standings/leaderboard  — HTMX fragment for auto-refresh
pub async fn leaderboard_fragment(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path(league_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    require_member(&state, league_id, user.id).await?;

    let (entries, has_live, no_tournament) = match db::get_active_tournament_id(&state.pool).await?
    {
        None => (vec![], false, true),
        Some(t_id) => {
            let (raw, has_live) = tokio::try_join!(
                db::get_leaderboard(&state.pool, t_id, league_id),
                db::has_live_matches(&state.pool, t_id),
            )?;
            (build_leaderboard(raw), has_live, false)
        }
    };

    Ok(LeaderboardFragment {
        league_id,
        entries,
        has_live,
        no_tournament,
    })
}

/// GET /leagues/{id}/standings/match/{match_id}
pub async fn match_breakdown(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path((league_id, match_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    require_member(&state, league_id, user.id).await?;

    let (league_name_opt, nav) = tokio::try_join!(
        db::get_league_name(&state.pool, league_id),
        crate::nav::load(&state.pool, &user, "standings"),
    )?;
    let league_name = league_name_opt.ok_or(AppError::NotFound)?;

    let t_id = db::get_active_tournament_id(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let match_info = db::get_match_info(&state.pool, t_id, match_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let rows = db::get_group_match_breakdown(&state.pool, league_id, match_id).await?;

    Ok(MatchBreakdownTemplate {
        league_name,
        match_info,
        rows,
        nav,
    })
}

/// GET /leagues/{id}/standings/compare
pub async fn compare_page(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path(league_id): Path<i64>,
    Query(params): Query<CompareParams>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    require_member(&state, league_id, user.id).await?;

    let (league_name_opt, nav) = tokio::try_join!(
        db::get_league_name(&state.pool, league_id),
        crate::nav::load(&state.pool, &user, "standings"),
    )?;
    let league_name = league_name_opt.ok_or(AppError::NotFound)?;

    let all_members = db::get_league_members(&state.pool, league_id).await?;

    let find_member = |id: i64| -> Option<LeagueMember> {
        all_members
            .iter()
            .find(|m| m.id == id)
            .map(|m| LeagueMember {
                id: m.id,
                name: m.name.clone(),
            })
    };

    let user_a = params.a.and_then(find_member);
    let user_b = params.b.and_then(find_member);

    let group_rows = match (&user_a, &user_b) {
        (Some(a), Some(b)) => match db::get_active_tournament_id(&state.pool).await? {
            Some(t_id) => db::get_compare_group_rows(&state.pool, t_id, a.id, b.id).await?,
            None => vec![],
        },
        _ => vec![],
    };

    Ok(CompareTemplate {
        league_id,
        league_name,
        members: all_members,
        user_a,
        user_b,
        group_rows,
        nav,
    })
}

/// GET /leagues/{id}/fixtures
pub async fn fixture_list(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path(league_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    require_member(&state, league_id, user.id).await?;

    let (league_name_opt, nav) = tokio::try_join!(
        db::get_league_name(&state.pool, league_id),
        crate::nav::load(&state.pool, &user, "standings"),
    )?;
    let league_name = league_name_opt.ok_or(AppError::NotFound)?;

    let (groups, no_tournament) = match db::get_active_tournament_id(&state.pool).await? {
        None => (vec![], true),
        Some(t_id) => {
            let rows = db::get_all_fixtures(&state.pool, t_id).await?;
            (group_fixtures(rows), false)
        }
    };

    Ok(FixturesTemplate {
        league_id,
        league_name,
        groups,
        no_tournament,
        nav,
    })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn require_member(state: &AppState, league_id: i64, user_id: i64) -> Result<(), AppError> {
    if !db::is_member(&state.pool, league_id, user_id).await? {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

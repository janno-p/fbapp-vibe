use askama::Template;
use askama_web::WebTemplate;
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    achievements,
    error::AppError,
    modules::auth::AuthSession,
    nav::NavContext,
    state::AppState,
};

use super::{
    db,
    models::{
        CompareGroupRow, FixtureGroup, GroupStandingsView, LeaderboardEntry, LeagueMember,
        MatchBreakdownRow, MatchConsensus, MatchInfo, MemberStats, NearestMatch, build_leaderboard,
        compute_streaks, group_fixtures,
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
    is_locked: bool,
    nav: NavContext,
}

#[derive(Template, WebTemplate)]
#[template(path = "standings/leaderboard.html")]
struct LeaderboardFragment {
    league_id: i64,
    entries: Vec<LeaderboardEntry>,
    has_live: bool,
    no_tournament: bool,
    is_locked: bool,
}

#[derive(Template, WebTemplate)]
#[template(path = "standings/not_locked.html")]
struct NotLockedTemplate {
    league_name: String,
    nav: NavContext,
}

#[derive(Template, WebTemplate)]
#[template(path = "standings/match.html")]
struct MatchBreakdownTemplate {
    league_name: String,
    match_info: MatchInfo,
    rows: Vec<MatchBreakdownRow>,
    is_locked: bool,
    consensus: Option<MatchConsensus>,
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
#[template(path = "standings/member_stats.html")]
struct MemberStatsTemplate {
    league_id: i64,
    league_name: String,
    stats: MemberStats,
    badges: Vec<achievements::BadgeDisplay>,
    nav: NavContext,
}

#[derive(Template, WebTemplate)]
#[template(path = "standings/groups.html")]
struct GroupStandingsTemplate {
    league_id: i64,
    league_name: String,
    groups: Vec<GroupStandingsView>,
    no_tournament: bool,
    nav: NavContext,
}

#[derive(Template, WebTemplate)]
#[template(path = "standings/rounds.html")]
struct RoundBreakdownTemplate {
    league_id: i64,
    league_name: String,
    rows: Vec<db::RoundPoints>,
    no_tournament: bool,
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

    let (entries, nearest, has_live, no_tournament, is_locked) =
        match db::get_active_tournament(&state.pool).await? {
            None => (vec![], None, false, true, false),
            Some(tournament) => {
                let locked = tournament.is_predictions_locked();
                let (raw, nearest, has_live, badges) = tokio::try_join!(
                    db::get_leaderboard(&state.pool, tournament.id, league_id),
                    db::get_nearest_match(&state.pool, tournament.id),
                    db::has_live_matches(&state.pool, tournament.id),
                    crate::achievements::get_top_badge_per_user(&state.pool, tournament.id),
                )?;
                (build_leaderboard(raw, badges), nearest, has_live, false, locked)
            }
        };

    Ok(StandingsTemplate {
        league_id,
        league_name,
        entries,
        nearest,
        has_live,
        no_tournament,
        is_locked,
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

    let (entries, has_live, no_tournament, is_locked) =
        match db::get_active_tournament(&state.pool).await? {
            None => (vec![], false, true, false),
            Some(tournament) => {
                let locked = tournament.is_predictions_locked();
                let (raw, has_live, badges) = tokio::try_join!(
                    db::get_leaderboard(&state.pool, tournament.id, league_id),
                    db::has_live_matches(&state.pool, tournament.id),
                    crate::achievements::get_top_badge_per_user(&state.pool, tournament.id),
                )?;
                (build_leaderboard(raw, badges), has_live, false, locked)
            }
        };

    Ok(LeaderboardFragment {
        league_id,
        entries,
        has_live,
        no_tournament,
        is_locked,
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

    let tournament = db::get_active_tournament(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let match_info = db::get_match_info(&state.pool, tournament.id, match_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let is_locked = tournament.is_predictions_locked();

    let (rows, consensus) = tokio::try_join!(
        db::get_group_match_breakdown(&state.pool, league_id, match_id),
        async {
            if is_locked {
                db::get_match_consensus(&state.pool, league_id, match_id)
                    .await
                    .map(Some)
            } else {
                Ok(None)
            }
        },
    )?;

    Ok(MatchBreakdownTemplate {
        league_name,
        match_info,
        rows,
        is_locked,
        consensus,
        nav,
    })
}

/// GET /leagues/{id}/standings/compare
pub async fn compare_page(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path(league_id): Path<i64>,
    Query(params): Query<CompareParams>,
) -> Result<Response, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    require_member(&state, league_id, user.id).await?;

    let (league_name_opt, nav) = tokio::try_join!(
        db::get_league_name(&state.pool, league_id),
        crate::nav::load(&state.pool, &user, "standings"),
    )?;
    let league_name = league_name_opt.ok_or(AppError::NotFound)?;

    if !db::get_active_tournament(&state.pool)
        .await?
        .as_ref()
        .map(|t| t.is_predictions_locked())
        .unwrap_or(false)
    {
        return Ok(NotLockedTemplate { league_name, nav }.into_response());
    }

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
    }
    .into_response())
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

/// GET /leagues/{id}/members/{user_id}
pub async fn member_stats(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path((league_id, target_user_id)): Path<(i64, i64)>,
) -> Result<Response, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    require_member(&state, league_id, user.id).await?;

    let (league_name_opt, member_info, nav) = tokio::try_join!(
        db::get_league_name(&state.pool, league_id),
        db::get_member_info(&state.pool, league_id, target_user_id),
        crate::nav::load(&state.pool, &user, "standings"),
    )?;
    let league_name = league_name_opt.ok_or(AppError::NotFound)?;
    let (user_name, league_joined_at) = member_info.ok_or(AppError::Forbidden)?;

    let tournament = db::get_active_tournament(&state.pool).await?;

    if !tournament
        .as_ref()
        .map(|t| t.is_predictions_locked())
        .unwrap_or(false)
    {
        return Ok(NotLockedTemplate { league_name, nav }.into_response());
    }

    let tournament_id = tournament.as_ref().map(|t| t.id);

    let stats = match tournament_id {
        None => MemberStats {
            user_id: target_user_id,
            user_name,
            league_joined_at,
            total_points: 0,
            rank: 0,
            group_correct: 0,
            group_total: 0,
            knockout_correct: 0,
            knockout_total: 0,
            top_scorer_points: 0,
            current_streak: 0,
            best_streak: 0,
        },
        Some(t_id) => {
            let (raw_leaderboard, group_preds, (knockout_correct, knockout_total), top_scorer_pts) =
                tokio::try_join!(
                    db::get_leaderboard(&state.pool, t_id, league_id),
                    db::get_member_group_preds(&state.pool, t_id, target_user_id),
                    db::get_member_knockout_stats(&state.pool, t_id, target_user_id),
                    db::get_member_top_scorer_points(&state.pool, t_id, target_user_id),
                )?;

            let leaderboard = build_leaderboard(raw_leaderboard, std::collections::HashMap::new());
            let lb_entry = leaderboard.iter().find(|e| e.user_id == target_user_id);
            let total_points = lb_entry.map(|e| e.total_points).unwrap_or(0);
            let rank = lb_entry.map(|e| e.rank).unwrap_or(leaderboard.len() + 1);

            let group_total = group_preds.len() as i64;
            let group_correct = group_preds.iter().filter(|r| r.is_correct()).count() as i64;
            let streak_bools: Vec<bool> = group_preds.iter().map(|r| r.is_correct()).collect();
            let (current_streak, best_streak) = compute_streaks(&streak_bools);

            MemberStats {
                user_id: target_user_id,
                user_name,
                league_joined_at,
                total_points,
                rank,
                group_correct,
                group_total,
                knockout_correct,
                knockout_total,
                top_scorer_points: top_scorer_pts,
                current_streak,
                best_streak,
            }
        }
    };

    let badges = match tournament_id {
        Some(t_id) => {
            achievements::get_user_badges(&state.pool, target_user_id, t_id)
                .await
                .unwrap_or_default()
        }
        None => vec![],
    };

    Ok(MemberStatsTemplate {
        league_id,
        league_name,
        stats,
        badges,
        nav,
    }
    .into_response())
}

/// GET /leagues/{id}/groups
pub async fn groups_page(
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
            let (match_results, team_names, group_names) =
                db::get_group_standings_data(&state.pool, t_id).await?;
            let computed = crate::group_standings::compute_standings(&match_results, &team_names);
            let mut views: Vec<GroupStandingsView> = computed
                .into_iter()
                .map(|gs| {
                    let group_name = group_names
                        .get(&gs.group_id)
                        .cloned()
                        .unwrap_or_else(|| gs.group_id.to_string());
                    GroupStandingsView {
                        group_name,
                        standings: gs.standings,
                    }
                })
                .collect();
            views.sort_by(|a, b| a.group_name.cmp(&b.group_name));
            (views, false)
        }
    };

    Ok(GroupStandingsTemplate {
        league_id,
        league_name,
        groups,
        no_tournament,
        nav,
    })
}

/// GET /leagues/{id}/standings/rounds
pub async fn round_breakdown(
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

    let (rows, no_tournament) = match db::get_active_tournament_id(&state.pool).await? {
        None => (vec![], true),
        Some(t_id) => {
            let rows = db::get_round_points(&state.pool, t_id, league_id).await?;
            (rows, false)
        }
    };

    Ok(RoundBreakdownTemplate {
        league_id,
        league_name,
        rows,
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

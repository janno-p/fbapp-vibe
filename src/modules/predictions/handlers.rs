use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Form,
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
};

use crate::{
    db_types::{KnockoutRound, MatchOutcome},
    error::AppError,
    extractors::QsForm,
    modules::auth::AuthSession,
    nav::NavContext,
    state::AppState,
};

const TOP_SCORER_PICKS: usize = 3;
const MAX_CONFIDENT_PICKS: usize = 3;

use super::{
    db,
    models::{
        GroupReviewRow, GroupStageForm, GroupWithMatches, KnockoutForm, KnockoutReviewRow,
        KnockoutRoundState, PlayerInfo, TeamInfo, TopScorerForm, TopScorerReviewRow,
    },
};

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "predictions/no_tournament.html")]
struct NoTournamentTemplate {
    nav: NavContext,
}

#[derive(Template, WebTemplate)]
#[template(path = "predictions/index.html")]
struct PredictionsTemplate {
    tournament_name: String,
    predictions_locked: bool,
    groups: Vec<GroupWithMatches>,
    teams: Vec<TeamInfo>,
    knockout_rounds: Vec<KnockoutRoundState>,
    players: Vec<PlayerInfo>,
    top_scorer_ids: Vec<i64>,
    nav: NavContext,
}

impl PredictionsTemplate {
    fn is_top_scorer(&self, player_id: &i64) -> bool {
        self.top_scorer_ids.contains(player_id)
    }

    /// Returns (predicted_count, total_count) for group stage matches.
    fn group_prediction_counts(&self) -> (usize, usize) {
        let total: usize = self.groups.iter().map(|g| g.matches.len()).sum();
        let predicted: usize = self
            .groups
            .iter()
            .flat_map(|g| &g.matches)
            .filter(|m| m.predicted_outcome.is_some())
            .count();
        (predicted, total)
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "predictions/review.html")]
struct ReviewTemplate {
    league_id: i64,
    group_rows: Vec<GroupReviewRow>,
    knockout_rows: Vec<KnockoutReviewRow>,
    top_scorer_rows: Vec<TopScorerReviewRow>,
    no_tournament: bool,
    nav: NavContext,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn predictions_page(
    auth_session: AuthSession,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    let nav = crate::nav::load(&state.pool, &user, "predictions").await?;

    let Some(tournament) = db::get_active_tournament(&state.pool).await? else {
        return Ok(NoTournamentTemplate { nav }.into_response());
    };

    let (groups, teams, knockout_rounds, players, top_scorer_ids) = tokio::try_join!(
        db::get_group_matches_with_predictions(&state.pool, tournament.id, user.id),
        db::get_teams(&state.pool, tournament.id),
        db::get_knockout_predictions(&state.pool, tournament.id, user.id),
        db::get_players_with_team(&state.pool, tournament.id),
        db::get_top_scorer_prediction_ids(&state.pool, tournament.id, user.id),
    )?;

    Ok(PredictionsTemplate {
        predictions_locked: tournament.is_predictions_locked(),
        tournament_name: tournament.name,
        groups,
        teams,
        knockout_rounds,
        players,
        top_scorer_ids,
        nav,
    }
    .into_response())
}

pub async fn save_group(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Form(form): Form<GroupStageForm>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;

    let tournament = db::get_active_tournament(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    if tournament.is_predictions_locked() {
        return Err(AppError::Forbidden);
    }

    // Parse match_{id} → MatchOutcome entries and confident_{id} → bool
    let mut predictions: Vec<(i64, MatchOutcome)> = Vec::new();
    let mut confident_match_ids: Vec<i64> = Vec::new();

    for (key, value) in form.iter() {
        if let Some(rest) = key.strip_prefix("match_") {
            if let (Ok(id), Some(outcome)) = (rest.parse::<i64>(), MatchOutcome::from_slug(value)) {
                predictions.push((id, outcome));
            }
        } else if let Some(rest) = key.strip_prefix("confident_")
            && value == "on"
            && let Ok(id) = rest.parse::<i64>()
        {
            confident_match_ids.push(id);
        }
    }

    if confident_match_ids.len() > MAX_CONFIDENT_PICKS {
        return Ok(Html(format!(
            r#"<span class="text-signal-red">Maximum {} confident picks allowed.</span>"#,
            MAX_CONFIDENT_PICKS
        ))
        .into_response());
    }

    db::save_group_stage_predictions(
        &state.pool,
        tournament.id,
        user.id,
        &predictions,
        &confident_match_ids,
    )
    .await?;

    Ok(Html("Saved").into_response())
}

pub async fn save_knockout(
    auth_session: AuthSession,
    State(state): State<AppState>,
    Path(round_slug): Path<String>,
    QsForm(form): QsForm<KnockoutForm>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;

    let round = KnockoutRound::from_slug(&round_slug)
        .ok_or(AppError::BadRequest("invalid knockout round".to_string()))?;

    if form.team_ids.len() != round.expected_team_count() {
        return Ok(Html(format!(
            r#"<span class="text-signal-red">Select exactly {} teams.</span>"#,
            round.expected_team_count()
        ))
        .into_response());
    }

    let tournament = db::get_active_tournament(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    if tournament.is_predictions_locked() {
        return Err(AppError::Forbidden);
    }

    db::save_knockout_round_predictions(
        &state.pool,
        tournament.id,
        user.id,
        &round,
        &form.team_ids,
    )
    .await?;

    Ok(Html("Saved").into_response())
}

pub async fn save_top_scorer(
    auth_session: AuthSession,
    State(state): State<AppState>,
    QsForm(form): QsForm<TopScorerForm>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;

    if form.player_ids.len() != TOP_SCORER_PICKS {
        return Ok(Html(format!(
            r#"<span class="text-signal-red">Select exactly {TOP_SCORER_PICKS} players.</span>"#
        ))
        .into_response());
    }

    let tournament = db::get_active_tournament(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    if tournament.is_predictions_locked() {
        return Err(AppError::Forbidden);
    }

    db::save_top_scorer_predictions(&state.pool, tournament.id, user.id, &form.player_ids).await?;

    Ok(Html("Saved").into_response())
}

/// GET /leagues/{id}/predictions/review
pub async fn predictions_review(
    auth_session: AuthSession,
    State(state): State<AppState>,
    axum::extract::Path(league_id): axum::extract::Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;

    if !crate::modules::leagues::is_member(&state.pool, league_id, user.id).await? {
        return Err(AppError::Forbidden);
    }

    let nav = crate::nav::load(&state.pool, &user, "standings").await?;

    let (group_rows, knockout_rows, top_scorer_rows, no_tournament) =
        match db::get_active_tournament(&state.pool).await? {
            None => (vec![], vec![], vec![], true),
            Some(t) => {
                let (group_rows, knockout_rows, top_scorer_rows) = tokio::try_join!(
                    db::get_group_predictions_review(&state.pool, t.id, user.id),
                    db::get_knockout_predictions_review(&state.pool, t.id, user.id),
                    db::get_top_scorer_predictions_review(&state.pool, t.id, user.id),
                )?;
                (group_rows, knockout_rows, top_scorer_rows, false)
            }
        };

    Ok(ReviewTemplate {
        league_id,
        group_rows,
        knockout_rows,
        top_scorer_rows,
        no_tournament,
        nav,
    })
}

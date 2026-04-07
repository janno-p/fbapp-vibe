use askama::Template;
use askama_web::WebTemplate;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Form,
};

use crate::{
    db_types::{KnockoutRound, MatchOutcome},
    error::AppError,
    extractors::QsForm,
    modules::auth::AuthSession,
    state::AppState,
};

const TOP_SCORER_PICKS: usize = 3;

use super::{
    db,
    models::{
        GroupStageForm, GroupWithMatches, KnockoutForm, KnockoutRoundState, PlayerInfo, TeamInfo,
        TopScorerForm,
    },
};

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "predictions/no_tournament.html")]
struct NoTournamentTemplate;

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
}

impl PredictionsTemplate {
    fn is_top_scorer(&self, player_id: &i64) -> bool {
        self.top_scorer_ids.contains(player_id)
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn predictions_page(
    auth_session: AuthSession,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;

    let Some(tournament) = db::get_active_tournament(&state.pool).await? else {
        return Ok(NoTournamentTemplate.into_response());
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

    // Parse match_{id} → MatchOutcome entries
    let predictions: Vec<(i64, MatchOutcome)> = form
        .iter()
        .filter_map(|(key, value)| {
            let id: i64 = key.strip_prefix("match_")?.parse().ok()?;
            let outcome = MatchOutcome::from_slug(value)?;
            Some((id, outcome))
        })
        .collect();

    db::save_group_stage_predictions(&state.pool, tournament.id, user.id, &predictions).await?;

    Ok(htmx_redirect("/predictions#group"))
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
        return Err(AppError::BadRequest(format!(
            "{} requires exactly {} teams, got {}",
            round.label(),
            round.expected_team_count(),
            form.team_ids.len()
        )));
    }

    let tournament = db::get_active_tournament(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    db::save_knockout_round_predictions(
        &state.pool,
        tournament.id,
        user.id,
        &round,
        &form.team_ids,
    )
    .await?;

    Ok(htmx_redirect("/predictions#knockout"))
}

pub async fn save_top_scorer(
    auth_session: AuthSession,
    State(state): State<AppState>,
    QsForm(form): QsForm<TopScorerForm>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;

    if form.player_ids.len() != TOP_SCORER_PICKS {
        return Err(AppError::BadRequest(format!(
            "top scorer requires exactly {TOP_SCORER_PICKS} players, got {}",
            form.player_ids.len()
        )));
    }

    let tournament = db::get_active_tournament(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    db::save_top_scorer_predictions(&state.pool, tournament.id, user.id, &form.player_ids).await?;

    Ok(htmx_redirect("/predictions#top-scorer"))
}

// ── Helper ────────────────────────────────────────────────────────────────────

/// Returns an HX-Redirect response so HTMX navigates to `url` after a save.
fn htmx_redirect(url: &str) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(url).unwrap_or_else(|_| HeaderValue::from_static("/predictions")),
    );
    (StatusCode::OK, headers, "")
}

use askama::Template;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    Form,
};

use crate::{error::AppError, football_api::Competition, state::AppState};

use super::{
    db,
    models::{RegisterTournamentForm, Tournament},
    AdminUser,
};

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct DashboardTemplate {
    tournaments: Vec<Tournament>,
}

#[derive(Template)]
#[template(path = "admin/competitions.html")]
struct CompetitionsTemplate {
    competitions: Vec<Competition>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn dashboard(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let tournaments = db::list_tournaments(&state.pool).await?;
    Ok(DashboardTemplate { tournaments })
}

pub async fn list_competitions(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let competitions = state
        .football_api
        .list_competitions()
        .await
        .map_err(|e| AppError::Unexpected(e))?;
    Ok(CompetitionsTemplate { competitions })
}

pub async fn register_tournament(
    _admin: AdminUser,
    State(state): State<AppState>,
    Form(form): Form<RegisterTournamentForm>,
) -> Result<impl IntoResponse, AppError> {
    let tournament_id = db::create_tournament(
        &state.pool,
        &form.external_id,
        &form.name,
        &form.season,
    )
    .await?;

    seed(&state, tournament_id, &form.code).await?;

    Ok(Redirect::to("/admin"))
}

pub async fn seed_tournament(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let tournaments = db::list_tournaments(&state.pool).await?;
    let tournament = tournaments
        .into_iter()
        .find(|t| t.id == id)
        .ok_or(AppError::NotFound)?;

    seed(&state, id, &tournament.external_id).await?;

    Ok(Redirect::to("/admin"))
}

pub async fn activate_tournament(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    db::activate_tournament(&state.pool, id).await?;
    Ok(Redirect::to("/admin"))
}

pub async fn deactivate_tournament(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    db::deactivate_tournament(&state.pool, id).await?;
    Ok(Redirect::to("/admin"))
}

pub async fn lock_tournament(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    db::lock_tournament(&state.pool, id).await?;
    Ok(Redirect::to("/admin"))
}

pub async fn unlock_tournament(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    db::unlock_tournament(&state.pool, id).await?;
    Ok(Redirect::to("/admin"))
}

// ── Shared helpers ────────────────────────────────────────────────────────────

async fn seed(state: &AppState, tournament_id: i64, code: &str) -> Result<(), AppError> {
    tracing::info!(tournament_id, code, "seeding tournament data");

    let teams = state
        .football_api
        .get_teams(code)
        .await
        .map_err(|e| AppError::Unexpected(e))?;

    let matches = state
        .football_api
        .get_matches(code)
        .await
        .map_err(|e| AppError::Unexpected(e))?;

    db::seed_tournament_data(&state.pool, tournament_id, &teams, &matches)
        .await
        .map_err(|e| AppError::Unexpected(e))?;

    tracing::info!(
        tournament_id,
        teams = teams.len(),
        matches = matches.len(),
        "seeding complete"
    );

    Ok(())
}

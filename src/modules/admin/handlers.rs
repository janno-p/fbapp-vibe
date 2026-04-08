use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Form,
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

use crate::{error::AppError, football_api::Competition, nav::NavContext, state::AppState};

use super::{
    AdminUser, db,
    models::{RegisterTournamentForm, Tournament},
};

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(Template, WebTemplate)]
#[template(path = "admin/dashboard.html")]
struct DashboardTemplate {
    tournaments: Vec<Tournament>,
    nav: NavContext,
}

#[derive(Template, WebTemplate)]
#[template(path = "admin/competitions.html")]
struct CompetitionsTemplate {
    competitions: Vec<Competition>,
    nav: NavContext,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn dashboard(
    admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let (tournaments, nav) = tokio::try_join!(
        db::list_tournaments(&state.pool),
        crate::nav::load(&state.pool, &admin.0, "admin"),
    )?;
    Ok(DashboardTemplate { tournaments, nav })
}

pub async fn list_competitions(
    admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let nav = crate::nav::load(&state.pool, &admin.0, "admin").await?;
    let competitions = state
        .football_api
        .list_competitions()
        .await
        .map_err(AppError::Unexpected)?;
    Ok(CompetitionsTemplate { competitions, nav })
}

pub async fn register_tournament(
    _admin: AdminUser,
    State(state): State<AppState>,
    Form(form): Form<RegisterTournamentForm>,
) -> Result<impl IntoResponse, AppError> {
    let tournament_id =
        db::create_tournament(&state.pool, &form.external_id, &form.name, &form.season).await?;

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
    admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    db::activate_tournament(&state.pool, id).await?;
    tracing::info!(
        tournament_id = id,
        user_id = admin.0.id,
        "tournament activated"
    );
    Ok(Redirect::to("/admin"))
}

pub async fn deactivate_tournament(
    admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    db::deactivate_tournament(&state.pool, id).await?;
    tracing::info!(
        tournament_id = id,
        user_id = admin.0.id,
        "tournament deactivated"
    );
    Ok(Redirect::to("/admin"))
}

pub async fn lock_tournament(
    admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    db::lock_tournament(&state.pool, id).await?;
    tracing::info!(
        tournament_id = id,
        user_id = admin.0.id,
        "tournament locked"
    );
    Ok(Redirect::to("/admin"))
}

pub async fn unlock_tournament(
    admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    db::unlock_tournament(&state.pool, id).await?;
    tracing::info!(
        tournament_id = id,
        user_id = admin.0.id,
        "tournament unlocked"
    );
    Ok(Redirect::to("/admin"))
}

// ── Shared helpers ────────────────────────────────────────────────────────────

async fn seed(state: &AppState, tournament_id: i64, code: &str) -> Result<(), AppError> {
    tracing::info!(tournament_id, code, "seeding tournament data");

    let teams = state
        .football_api
        .get_teams(code)
        .await
        .map_err(AppError::Unexpected)?;

    let matches = state
        .football_api
        .get_matches(code)
        .await
        .map_err(AppError::Unexpected)?;

    db::seed_tournament_data(&state.pool, tournament_id, &teams, &matches)
        .await
        .map_err(AppError::Unexpected)?;

    tracing::info!(
        tournament_id,
        teams = teams.len(),
        matches = matches.len(),
        "seeding complete"
    );

    Ok(())
}

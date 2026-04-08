pub mod db;
pub mod scorer;

use std::time::Duration;

use tracing::{info, warn};

use crate::{db_types::KnockoutRound, football_api::MatchStatus, state::AppState};

/// Background task entry point. Loops forever, polling the football API and
/// scoring predictions. Errors are logged and retried on the next cycle.
pub async fn run(state: AppState) {
    info!("polling task started");
    loop {
        let interval_secs = match poll(&state).await {
            Ok(is_live) => {
                if is_live {
                    state.config.poll_interval_live_secs
                } else {
                    state.config.poll_interval_secs
                }
            }
            Err(e) => {
                warn!("poll cycle failed: {e:#}");
                state.config.poll_interval_secs
            }
        };
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}

/// Runs a single poll cycle. Returns `true` if there are live matches (which
/// triggers a shorter sleep before the next cycle).
async fn poll(state: &AppState) -> anyhow::Result<bool> {
    let Some(tournament) = db::get_active_tournament(&state.pool).await? else {
        return Ok(false);
    };

    // ── Process finished matches ──────────────────────────────────────────────

    let matches = state
        .football_api
        .get_matches(&tournament.external_id)
        .await?;

    for m in &matches {
        if m.status != MatchStatus::Finished {
            continue;
        }
        let Some(winner) = m.score.winner.as_ref() else {
            continue;
        };
        let outcome = winner.to_outcome();
        if let Err(e) = db::process_finished_match(
            &state.pool,
            tournament.id,
            m.id,
            outcome,
            m.score.full_time.home,
            m.score.full_time.away,
        )
        .await
        {
            warn!("failed to process match {}: {e:#}", m.id);
        }
    }

    // ── Score knockout rounds that are now complete ────────────────────────────

    for round in &[
        KnockoutRound::R32,
        KnockoutRound::R16,
        KnockoutRound::Qf,
        KnockoutRound::Sf,
        KnockoutRound::Final,
    ] {
        if !db::is_knockout_round_complete(&state.pool, tournament.id, round).await? {
            continue;
        }

        let team_ids = db::get_teams_in_knockout_round(&state.pool, tournament.id, round).await?;

        if !team_ids.is_empty() {
            let points = scorer::knockout_points_per_team(round);
            db::score_knockout_predictions(&state.pool, tournament.id, round, &team_ids, points)
                .await?;
        }

        if *round == KnockoutRound::Final
            && let Some(winner_id) = db::get_final_winner(&state.pool, tournament.id).await?
        {
            db::score_winner_predictions(&state.pool, tournament.id, winner_id).await?;
        }
    }

    // ── Update player goal tallies ────────────────────────────────────────────

    match state
        .football_api
        .get_scorers(&tournament.external_id)
        .await
    {
        Ok(scorers) => {
            for entry in &scorers {
                let goals = entry.goals.unwrap_or(0);
                if let Err(e) = db::update_player_goals(
                    &state.pool,
                    tournament.id,
                    &entry.player.id.to_string(),
                    goals,
                )
                .await
                {
                    warn!(
                        "failed to update goals for player {}: {e:#}",
                        entry.player.id
                    );
                }
            }
        }
        Err(e) => warn!("failed to fetch scorers: {e:#}"),
    }

    // ── Score top scorers once the tournament is finished ─────────────────────

    if db::all_matches_complete(&state.pool, tournament.id).await? {
        db::score_top_scorer_predictions(&state.pool, tournament.id).await?;
    }

    let is_live = db::has_live_matches(&state.pool, tournament.id).await?;
    Ok(is_live)
}

use sqlx::PgPool;

use crate::db_types::{KnockoutRound, MatchOutcome};

// ── Read queries ──────────────────────────────────────────────────────────────

pub struct ActiveTournament {
    pub id: i64,
    pub external_id: String,
}

pub async fn get_active_tournament(pool: &PgPool) -> anyhow::Result<Option<ActiveTournament>> {
    let row =
        sqlx::query!("SELECT id, external_id FROM tournaments WHERE is_active = TRUE LIMIT 1")
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| ActiveTournament {
        id: r.id,
        external_id: r.external_id,
    }))
}

/// Returns true if there is at least one match that started within the last
/// 2 hours and does not yet have a result (i.e. likely in progress).
pub async fn has_live_matches(pool: &PgPool, tournament_id: i64) -> anyhow::Result<bool> {
    let live = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM matches
            WHERE tournament_id = $1
              AND outcome IS NULL
              AND scheduled_at >= NOW() - INTERVAL '2 hours'
        )
        "#,
        tournament_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);
    Ok(live)
}

/// Returns true when every match in the tournament has an outcome recorded.
pub async fn all_matches_complete(pool: &PgPool, tournament_id: i64) -> anyhow::Result<bool> {
    let done = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) > 0 AND COUNT(*) FILTER (WHERE outcome IS NULL) = 0
        FROM matches
        WHERE tournament_id = $1
        "#,
        tournament_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);
    Ok(done)
}

/// Returns true when all matches for `round` exist and all have an outcome.
pub async fn is_knockout_round_complete(
    pool: &PgPool,
    tournament_id: i64,
    round: &KnockoutRound,
) -> anyhow::Result<bool> {
    let done = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) > 0 AND COUNT(*) FILTER (WHERE outcome IS NULL) = 0
        FROM matches
        WHERE tournament_id = $1 AND round = $2
        "#,
        tournament_id,
        round as &KnockoutRound,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);
    Ok(done)
}

/// Returns IDs of all teams that participated in `round` (as home or away).
pub async fn get_teams_in_knockout_round(
    pool: &PgPool,
    tournament_id: i64,
    round: &KnockoutRound,
) -> anyhow::Result<Vec<i64>> {
    let home_ids = sqlx::query_scalar!(
        "SELECT home_team_id FROM matches WHERE tournament_id = $1 AND round = $2 AND home_team_id IS NOT NULL",
        tournament_id,
        round as &KnockoutRound,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let away_ids = sqlx::query_scalar!(
        "SELECT away_team_id FROM matches WHERE tournament_id = $1 AND round = $2 AND away_team_id IS NOT NULL",
        tournament_id,
        round as &KnockoutRound,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let mut team_ids: Vec<i64> = home_ids.into_iter().chain(away_ids).collect();
    team_ids.sort_unstable();
    team_ids.dedup();
    Ok(team_ids)
}

/// Returns the winning team ID from the Final match, or None if the Final is
/// not yet complete or ended in a draw (which shouldn't happen in a real final).
pub async fn get_final_winner(pool: &PgPool, tournament_id: i64) -> anyhow::Result<Option<i64>> {
    let row = sqlx::query!(
        r#"
        SELECT home_team_id, away_team_id,
               outcome AS "outcome: MatchOutcome"
        FROM matches
        WHERE tournament_id = $1
          AND round = 'final'::knockout_round
          AND outcome IS NOT NULL
        LIMIT 1
        "#,
        tournament_id,
    )
    .fetch_optional(pool)
    .await?;

    let winner = row.and_then(|r| match r.outcome {
        Some(MatchOutcome::Home) => r.home_team_id,
        Some(MatchOutcome::Away) => r.away_team_id,
        _ => None,
    });
    Ok(winner)
}

// ── Write queries ─────────────────────────────────────────────────────────────

/// If the tournament's first match has started and `predictions_locked_at` is NULL,
/// sets `predictions_locked_at` to the first match's `scheduled_at`.
/// Returns `true` if the lock was applied, `false` if already locked or no match
/// has started yet. Safe under concurrent polling runs: the `WHERE
/// predictions_locked_at IS NULL` guard means only one cycle can win the update.
pub async fn auto_lock_if_started(pool: &PgPool, tournament_id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE tournaments
        SET predictions_locked_at = (
            SELECT MIN(scheduled_at)
            FROM matches
            WHERE tournament_id = $1
        )
        WHERE id = $1
          AND predictions_locked_at IS NULL
          AND EXISTS (
              SELECT 1 FROM matches
              WHERE tournament_id = $1
                AND scheduled_at <= NOW()
          )
        RETURNING id
        "#,
        tournament_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

/// Tries to record a finished match result and score group stage predictions.
///
/// Uses `pg_try_advisory_xact_lock` (keyed on the API match ID) so that
/// concurrent instances skip rather than double-process the same match.
/// The `outcome IS NULL` guard makes the update idempotent.
///
/// Returns `true` if the match was newly scored, `false` if it was already
/// processed or the advisory lock was held by another session.
pub async fn process_finished_match(
    pool: &PgPool,
    tournament_id: i64,
    api_match_id: i64,
    outcome: MatchOutcome,
    home_score: Option<i32>,
    away_score: Option<i32>,
) -> anyhow::Result<bool> {
    let mut tx = pool.begin().await?;

    let got_lock: bool = sqlx::query_scalar!("SELECT pg_try_advisory_xact_lock($1)", api_match_id,)
        .fetch_one(&mut *tx)
        .await?
        .unwrap_or(false);

    if !got_lock {
        tx.commit().await?;
        return Ok(false);
    }

    let ext_id = api_match_id.to_string();

    let rows_affected = sqlx::query!(
        r#"
        UPDATE matches
        SET outcome = $1, home_score = $2, away_score = $3
        WHERE tournament_id = $4 AND external_id = $5 AND outcome IS NULL
        "#,
        outcome.clone() as MatchOutcome,
        home_score,
        away_score,
        tournament_id,
        ext_id,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        tx.commit().await?;
        return Ok(false);
    }

    // Score group stage predictions for this match (no-op for knockout matches).
    // Confident + correct = 2pts; non-confident correct = 1pt; wrong = 0pts.
    sqlx::query!(
        r#"
        UPDATE group_stage_predictions
        SET points_awarded = CASE
            WHEN predicted_outcome = $1 AND is_confident THEN 2
            WHEN predicted_outcome = $1 THEN 1
            ELSE 0
        END
        WHERE match_id = (
            SELECT id FROM matches WHERE tournament_id = $2 AND external_id = $3
        )
        AND points_awarded IS NULL
        "#,
        outcome as MatchOutcome,
        tournament_id,
        ext_id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(true)
}

/// Sets `points_awarded` on all unscored knockout predictions for `round`.
/// Teams in `team_ids_in_round` receive `points_per_team`; others receive 0.
pub async fn score_knockout_predictions(
    pool: &PgPool,
    tournament_id: i64,
    round: &KnockoutRound,
    team_ids_in_round: &[i64],
    points_per_team: i32,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE knockout_predictions
        SET points_awarded = CASE WHEN team_id = ANY($1::bigint[]) THEN $2 ELSE 0 END
        WHERE tournament_id = $3 AND round = $4 AND points_awarded IS NULL
        "#,
        team_ids_in_round as &[i64],
        points_per_team,
        tournament_id,
        round as &KnockoutRound,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Sets `points_awarded` on all unscored winner predictions.
/// The team that won the Final scores 10; all other picks score 0.
pub async fn score_winner_predictions(
    pool: &PgPool,
    tournament_id: i64,
    winner_team_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE knockout_predictions
        SET points_awarded = CASE WHEN team_id = $1 THEN 10 ELSE 0 END
        WHERE tournament_id = $2
          AND round = 'winner'::knockout_round
          AND points_awarded IS NULL
        "#,
        winner_team_id,
        tournament_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Updates a player's `goals_scored` from the scorers API response.
pub async fn update_player_goals(
    pool: &PgPool,
    tournament_id: i64,
    player_external_id: &str,
    goals: i32,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE players SET goals_scored = $1 WHERE tournament_id = $2 AND external_id = $3",
        goals,
        tournament_id,
        player_external_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Scores top scorer predictions once the tournament is over.
///
/// Finds the player(s) with the highest `goals_scored` (ties all count).
/// Matching predictions receive 5 + goals; non-matching receive 0.
/// No-op if no players have scored or all predictions are already scored.
pub async fn score_top_scorer_predictions(pool: &PgPool, tournament_id: i64) -> anyhow::Result<()> {
    let max_goals: i32 = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(goals_scored), 0) FROM players WHERE tournament_id = $1",
        tournament_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    if max_goals == 0 {
        return Ok(());
    }

    let top_scorer_ids: Vec<i64> = sqlx::query_scalar!(
        "SELECT id FROM players WHERE tournament_id = $1 AND goals_scored = $2",
        tournament_id,
        max_goals,
    )
    .fetch_all(pool)
    .await?;

    // Award points for matching predictions.
    sqlx::query!(
        r#"
        UPDATE top_scorer_predictions
        SET points_awarded = 5 + (SELECT goals_scored FROM players WHERE id = player_id)
        WHERE tournament_id = $1
          AND player_id = ANY($2::bigint[])
          AND points_awarded IS NULL
        "#,
        tournament_id,
        &top_scorer_ids as &[i64],
    )
    .execute(pool)
    .await?;

    // Award 0 for non-matching predictions.
    sqlx::query!(
        r#"
        UPDATE top_scorer_predictions
        SET points_awarded = 0
        WHERE tournament_id = $1
          AND player_id != ALL($2::bigint[])
          AND points_awarded IS NULL
        "#,
        tournament_id,
        &top_scorer_ids as &[i64],
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use time::OffsetDateTime;

    async fn make_tournament(pool: &PgPool) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO tournaments (external_id, name, season, is_active) VALUES ('WC', 'Test Cup', '2026', TRUE) RETURNING id"
        )
        .fetch_one(pool)
        .await
        .expect("insert tournament")
    }

    async fn make_user(pool: &PgPool) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO users (google_id, email, name) VALUES ('gid', 'u@test.com', 'U') RETURNING id"
        )
        .fetch_one(pool)
        .await
        .expect("insert user")
    }

    async fn make_team(pool: &PgPool, tournament_id: i64, ext_id: &str) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO teams (tournament_id, external_id, name, short_name) VALUES ($1, $2, $2, $2) RETURNING id",
            tournament_id, ext_id,
        )
        .fetch_one(pool)
        .await
        .expect("insert team")
    }

    async fn make_player(pool: &PgPool, tournament_id: i64, team_id: i64, ext_id: &str) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO players (tournament_id, team_id, external_id, name) VALUES ($1, $2, $3, $3) RETURNING id",
            tournament_id, team_id, ext_id,
        )
        .fetch_one(pool)
        .await
        .expect("insert player")
    }

    async fn make_group_match(
        pool: &PgPool,
        tournament_id: i64,
        home_id: i64,
        away_id: i64,
        api_id: i64,
    ) -> i64 {
        let group_id: i64 = sqlx::query_scalar!(
            "INSERT INTO groups (tournament_id, name) VALUES ($1, 'A') ON CONFLICT (tournament_id, name) DO UPDATE SET name = 'A' RETURNING id",
            tournament_id
        )
        .fetch_one(pool)
        .await
        .expect("upsert group");

        sqlx::query_scalar!(
            r#"
            INSERT INTO matches (tournament_id, external_id, group_id, home_team_id, away_team_id, scheduled_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            tournament_id, api_id.to_string(), group_id, home_id, away_id,
            OffsetDateTime::now_utc(),
        )
        .fetch_one(pool)
        .await
        .expect("insert match")
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn process_finished_match_scores_group_predictions(pool: PgPool) {
        let t_id = make_tournament(&pool).await;
        let u_id = make_user(&pool).await;
        let home = make_team(&pool, t_id, "Home").await;
        let away = make_team(&pool, t_id, "Away").await;
        let _m_id = make_group_match(&pool, t_id, home, away, 99001).await;

        // User predicts home win
        sqlx::query!(
            "INSERT INTO group_stage_predictions (user_id, match_id, predicted_outcome)
             SELECT $1, id, 'home'::match_outcome FROM matches WHERE external_id = '99001'",
            u_id,
        )
        .execute(&pool)
        .await
        .expect("insert prediction");

        // Actual result: home wins
        let updated =
            process_finished_match(&pool, t_id, 99001, MatchOutcome::Home, Some(2), Some(1))
                .await
                .expect("process match");

        assert!(updated, "match should be newly scored");

        let points: Option<i32> = sqlx::query_scalar!(
            "SELECT points_awarded FROM group_stage_predictions WHERE user_id = $1",
            u_id,
        )
        .fetch_one(&pool)
        .await
        .expect("fetch points");

        assert_eq!(points, Some(1), "correct prediction should score 1 point");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn process_finished_match_is_idempotent(pool: PgPool) {
        let t_id = make_tournament(&pool).await;
        let u_id = make_user(&pool).await;
        let home = make_team(&pool, t_id, "Home").await;
        let away = make_team(&pool, t_id, "Away").await;
        let _m_id = make_group_match(&pool, t_id, home, away, 99002).await;

        sqlx::query!(
            "INSERT INTO group_stage_predictions (user_id, match_id, predicted_outcome)
             SELECT $1, id, 'home'::match_outcome FROM matches WHERE external_id = '99002'",
            u_id,
        )
        .execute(&pool)
        .await
        .expect("insert prediction");

        process_finished_match(&pool, t_id, 99002, MatchOutcome::Home, Some(1), Some(0))
            .await
            .expect("first process");
        let second =
            process_finished_match(&pool, t_id, 99002, MatchOutcome::Home, Some(1), Some(0))
                .await
                .expect("second process");

        assert!(!second, "second call should be a no-op");

        let points: Option<i32> = sqlx::query_scalar!(
            "SELECT points_awarded FROM group_stage_predictions WHERE user_id = $1",
            u_id,
        )
        .fetch_one(&pool)
        .await
        .expect("fetch points");

        assert_eq!(points, Some(1), "points should not be double-counted");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn top_scorer_tied_players_both_award_points(pool: PgPool) {
        let t_id = make_tournament(&pool).await;
        let u1 = make_user(&pool).await;
        let team = make_team(&pool, t_id, "T1").await;
        let p1 = make_player(&pool, t_id, team, "P1").await;
        let p2 = make_player(&pool, t_id, team, "P2").await;
        let p3 = make_player(&pool, t_id, team, "P3").await;

        // Both p1 and p2 have 5 goals (tied top scorers)
        sqlx::query!("UPDATE players SET goals_scored = 5 WHERE id = $1", p1)
            .execute(&pool)
            .await
            .expect("set goals p1");
        sqlx::query!("UPDATE players SET goals_scored = 5 WHERE id = $1", p2)
            .execute(&pool)
            .await
            .expect("set goals p2");

        // u1 predicts p1, p2, p3
        for player_id in [p1, p2, p3] {
            sqlx::query!(
                "INSERT INTO top_scorer_predictions (user_id, tournament_id, player_id) VALUES ($1, $2, $3)",
                u1, t_id, player_id,
            )
            .execute(&pool)
            .await
            .expect("insert top scorer prediction");
        }

        score_top_scorer_predictions(&pool, t_id)
            .await
            .expect("score");

        // p1 and p2 should each award 5 + 5 = 10 points
        let p1_pts: Option<i32> = sqlx::query_scalar!(
            "SELECT points_awarded FROM top_scorer_predictions WHERE user_id = $1 AND player_id = $2",
            u1, p1,
        ).fetch_one(&pool).await.expect("fetch p1 points");

        let p2_pts: Option<i32> = sqlx::query_scalar!(
            "SELECT points_awarded FROM top_scorer_predictions WHERE user_id = $1 AND player_id = $2",
            u1, p2,
        ).fetch_one(&pool).await.expect("fetch p2 points");

        let p3_pts: Option<i32> = sqlx::query_scalar!(
            "SELECT points_awarded FROM top_scorer_predictions WHERE user_id = $1 AND player_id = $2",
            u1, p3,
        ).fetch_one(&pool).await.expect("fetch p3 points");

        assert_eq!(p1_pts, Some(10), "p1 tied top scorer: 5 + 5 goals");
        assert_eq!(p2_pts, Some(10), "p2 tied top scorer: 5 + 5 goals");
        assert_eq!(p3_pts, Some(0), "p3 not a top scorer");
    }

    async fn make_match_at(pool: &PgPool, tournament_id: i64, scheduled_at: OffsetDateTime) {
        let group_id: i64 = sqlx::query_scalar!(
            "INSERT INTO groups (tournament_id, name) VALUES ($1, 'X') ON CONFLICT (tournament_id, name) DO UPDATE SET name = 'X' RETURNING id",
            tournament_id
        )
        .fetch_one(pool)
        .await
        .expect("upsert group");

        sqlx::query!(
            "INSERT INTO matches (tournament_id, external_id, group_id, scheduled_at) VALUES ($1, 'ext-autolock', $2, $3)",
            tournament_id, group_id, scheduled_at,
        )
        .execute(pool)
        .await
        .expect("insert match");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn auto_lock_sets_lock_when_first_match_started(pool: PgPool) {
        let t_id = make_tournament(&pool).await;
        let past = OffsetDateTime::now_utc() - time::Duration::hours(1);
        make_match_at(&pool, t_id, past).await;

        let locked = auto_lock_if_started(&pool, t_id).await.expect("auto_lock");
        assert!(locked, "should lock when a match has started");

        let locked_at: Option<OffsetDateTime> = sqlx::query_scalar!(
            "SELECT predictions_locked_at FROM tournaments WHERE id = $1",
            t_id,
        )
        .fetch_one(&pool)
        .await
        .expect("fetch tournament");

        assert!(locked_at.is_some(), "predictions_locked_at should be set");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn auto_lock_does_not_lock_when_match_is_in_future(pool: PgPool) {
        let t_id = make_tournament(&pool).await;
        let future = OffsetDateTime::now_utc() + time::Duration::hours(1);
        make_match_at(&pool, t_id, future).await;

        let locked = auto_lock_if_started(&pool, t_id).await.expect("auto_lock");
        assert!(!locked, "should not lock when all matches are in the future");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn auto_lock_does_not_overwrite_existing_lock(pool: PgPool) {
        let t_id = make_tournament(&pool).await;
        let past = OffsetDateTime::now_utc() - time::Duration::hours(1);
        make_match_at(&pool, t_id, past).await;

        // Manually lock the tournament first
        let manual_lock = OffsetDateTime::now_utc() - time::Duration::hours(2);
        sqlx::query!(
            "UPDATE tournaments SET predictions_locked_at = $1 WHERE id = $2",
            manual_lock,
            t_id,
        )
        .execute(&pool)
        .await
        .expect("manual lock");

        let locked = auto_lock_if_started(&pool, t_id).await.expect("auto_lock");
        assert!(!locked, "should not overwrite existing lock");

        let locked_at: Option<OffsetDateTime> = sqlx::query_scalar!(
            "SELECT predictions_locked_at FROM tournaments WHERE id = $1",
            t_id,
        )
        .fetch_one(&pool)
        .await
        .expect("fetch tournament");

        assert_eq!(
            locked_at.map(|t| t.unix_timestamp()),
            Some(manual_lock.unix_timestamp()),
            "lock timestamp should remain unchanged"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn auto_lock_is_idempotent_after_first_lock(pool: PgPool) {
        let t_id = make_tournament(&pool).await;
        let past = OffsetDateTime::now_utc() - time::Duration::hours(1);
        make_match_at(&pool, t_id, past).await;

        let first = auto_lock_if_started(&pool, t_id).await.expect("first call");
        assert!(first, "first call should lock");

        let second = auto_lock_if_started(&pool, t_id).await.expect("second call");
        assert!(!second, "second call should be a no-op");
    }
}

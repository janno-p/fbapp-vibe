use std::collections::HashMap;

use sqlx::PgPool;

use crate::{
    crests::find_crest_url,
    db_types::{KnockoutRound, MatchOutcome},
    error::AppError,
    modules::admin::models::Tournament,
};

use super::models::{
    GroupReviewRow, GroupWithMatches, KnockoutReviewRow, KnockoutRoundState, MatchRow, PlayerInfo,
    TeamInfo, TopScorerReviewRow,
};

// ── Read queries ──────────────────────────────────────────────────────────────

pub async fn get_active_tournament(pool: &PgPool) -> anyhow::Result<Option<Tournament>> {
    let t = sqlx::query_as!(
        Tournament,
        r#"
        SELECT id, external_id, name, season, is_active, predictions_locked_at
        FROM tournaments
        WHERE is_active = TRUE
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?;
    Ok(t)
}

pub async fn get_group_matches_with_predictions(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
) -> anyhow::Result<Vec<GroupWithMatches>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            m.id,
            g.name  AS group_name,
            ht.name AS "home_team_name?: String",
            at.name AS "away_team_name?: String",
            ht.crest_url  AS "home_crest_url?: String",
            at.crest_url  AS "away_crest_url?: String",
            m.scheduled_at,
            gsp.predicted_outcome AS "predicted_outcome?: MatchOutcome"
        FROM matches m
        JOIN groups g ON m.group_id = g.id
        LEFT JOIN teams ht ON m.home_team_id = ht.id
        LEFT JOIN teams at ON m.away_team_id = at.id
        LEFT JOIN group_stage_predictions gsp
               ON gsp.match_id = m.id AND gsp.user_id = $2
        WHERE m.tournament_id = $1
        ORDER BY g.name, m.scheduled_at
        "#,
        tournament_id,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    // Group by group_name preserving sorted order
    let mut map: HashMap<String, Vec<MatchRow>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for r in rows {
        let entry = map.entry(r.group_name.clone()).or_insert_with(|| {
            order.push(r.group_name.clone());
            Vec::new()
        });
        entry.push(MatchRow {
            id: r.id,
            group_name: r.group_name,
            home_team_name: r.home_team_name,
            away_team_name: r.away_team_name,
            home_crest_url: find_crest_url(r.home_crest_url.as_deref()),
            away_crest_url: find_crest_url(r.away_crest_url.as_deref()),
            scheduled_at: r.scheduled_at,
            predicted_outcome: r.predicted_outcome,
        });
    }

    Ok(order
        .into_iter()
        .filter_map(|name| {
            map.remove(&name)
                .map(|matches| GroupWithMatches { name, matches })
        })
        .collect())
}

pub async fn get_teams(pool: &PgPool, tournament_id: i64) -> anyhow::Result<Vec<TeamInfo>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, name, short_name, crest_url
        FROM teams
        WHERE tournament_id = $1
        ORDER BY name
        "#,
        tournament_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| TeamInfo {
            id: r.id,
            name: r.name,
            short_name: r.short_name,
            crest_url: find_crest_url(r.crest_url.as_deref()),
        })
        .collect())
}

pub async fn get_players_with_team(
    pool: &PgPool,
    tournament_id: i64,
) -> anyhow::Result<Vec<PlayerInfo>> {
    let players = sqlx::query_as!(
        PlayerInfo,
        r#"
        SELECT p.id, p.name, t.name AS team_name, p.goals_scored
        FROM players p
        JOIN teams t ON p.team_id = t.id
        WHERE p.tournament_id = $1
        ORDER BY p.goals_scored DESC, p.name ASC
        "#,
        tournament_id
    )
    .fetch_all(pool)
    .await?;
    Ok(players)
}

pub async fn get_knockout_predictions(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
) -> anyhow::Result<Vec<KnockoutRoundState>> {
    let rows = sqlx::query!(
        r#"
        SELECT round AS "round: KnockoutRound", team_id
        FROM knockout_predictions
        WHERE tournament_id = $1 AND user_id = $2
        ORDER BY round, team_id
        "#,
        tournament_id,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    // Build a state entry for every round, populating team_ids from the DB rows
    let mut round_map: HashMap<String, Vec<i64>> = HashMap::new();
    for r in rows {
        round_map
            .entry(r.round.slug().to_string())
            .or_default()
            .push(r.team_id);
    }

    Ok(KnockoutRound::all()
        .iter()
        .map(|round| KnockoutRoundState {
            predicted_team_ids: round_map.remove(round.slug()).unwrap_or_default(),
            round: round.clone(),
        })
        .collect())
}

pub async fn get_top_scorer_prediction_ids(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
) -> anyhow::Result<Vec<i64>> {
    let ids = sqlx::query_scalar!(
        "SELECT player_id FROM top_scorer_predictions WHERE tournament_id = $1 AND user_id = $2",
        tournament_id,
        user_id
    )
    .fetch_all(pool)
    .await?;
    Ok(ids)
}

// ── Review queries ────────────────────────────────────────────────────────────

pub async fn get_group_predictions_review(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
) -> anyhow::Result<Vec<GroupReviewRow>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            g.name                          AS group_name,
            COALESCE(ht.name, 'TBD')        AS "home_name!: String",
            COALESCE(at.name, 'TBD')        AS "away_name!: String",
            m.scheduled_at,
            gsp.predicted_outcome           AS "predicted_outcome: MatchOutcome",
            m.outcome                       AS "actual_outcome?: MatchOutcome",
            gsp.points_awarded
        FROM group_stage_predictions gsp
        JOIN matches m  ON m.id  = gsp.match_id
        JOIN groups  g  ON g.id  = m.group_id
        LEFT JOIN teams ht ON ht.id = m.home_team_id
        LEFT JOIN teams at ON at.id = m.away_team_id
        WHERE m.tournament_id = $1 AND gsp.user_id = $2
        ORDER BY g.name, m.scheduled_at
        "#,
        tournament_id,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| GroupReviewRow {
            group_name: r.group_name,
            home_name: r.home_name,
            away_name: r.away_name,
            scheduled_at: r.scheduled_at,
            predicted_outcome: r.predicted_outcome,
            actual_outcome: r.actual_outcome,
            points_awarded: r.points_awarded,
        })
        .collect())
}

pub async fn get_knockout_predictions_review(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
) -> anyhow::Result<Vec<KnockoutReviewRow>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            kp.round            AS "round: KnockoutRound",
            t.name              AS team_name,
            kp.points_awarded
        FROM knockout_predictions kp
        JOIN teams t ON t.id = kp.team_id
        WHERE kp.tournament_id = $1 AND kp.user_id = $2
        ORDER BY kp.round, t.name
        "#,
        tournament_id,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| KnockoutReviewRow {
            round: r.round,
            team_name: r.team_name,
            points_awarded: r.points_awarded,
        })
        .collect())
}

pub async fn get_top_scorer_predictions_review(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
) -> anyhow::Result<Vec<TopScorerReviewRow>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            p.name      AS player_name,
            t.name      AS team_name,
            p.goals_scored,
            tsp.points_awarded
        FROM top_scorer_predictions tsp
        JOIN players p ON p.id = tsp.player_id
        JOIN teams   t ON t.id = p.team_id
        WHERE tsp.tournament_id = $1 AND tsp.user_id = $2
        ORDER BY p.goals_scored DESC, p.name
        "#,
        tournament_id,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| TopScorerReviewRow {
            player_name: r.player_name,
            team_name: r.team_name,
            goals_scored: r.goals_scored,
            points_awarded: r.points_awarded,
        })
        .collect())
}

// ── Write queries (all require lock check) ────────────────────────────────────

/// Acquires a `FOR UPDATE` lock on the tournament row and returns an error if
/// predictions are locked.  Must be called inside an open transaction.
async fn assert_predictions_open(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tournament_id: i64,
) -> Result<(), AppError> {
    let row = sqlx::query!(
        "SELECT predictions_locked_at FROM tournaments WHERE id = $1 FOR UPDATE",
        tournament_id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| AppError::Unexpected(e.into()))?;

    if row
        .predictions_locked_at
        .is_some_and(|t| t <= time::OffsetDateTime::now_utc())
    {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

pub async fn save_group_stage_predictions(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
    predictions: &[(i64, MatchOutcome)],
) -> Result<(), AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    assert_predictions_open(&mut tx, tournament_id).await?;

    for (match_id, outcome) in predictions {
        sqlx::query!(
            r#"
            INSERT INTO group_stage_predictions (user_id, match_id, predicted_outcome)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, match_id) DO UPDATE
                SET predicted_outcome = EXCLUDED.predicted_outcome
            "#,
            user_id,
            match_id,
            outcome as &MatchOutcome,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    Ok(())
}

pub async fn save_knockout_round_predictions(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
    round: &KnockoutRound,
    team_ids: &[i64],
) -> Result<(), AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    assert_predictions_open(&mut tx, tournament_id).await?;

    let valid_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM teams WHERE tournament_id = $1 AND id = ANY($2::bigint[])",
        tournament_id,
        team_ids as &[i64],
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Unexpected(e.into()))?
    .unwrap_or(0);

    if valid_count as usize != team_ids.len() {
        return Err(AppError::BadRequest(
            "one or more team IDs are not valid for this tournament".to_string(),
        ));
    }

    sqlx::query!(
        r#"
        DELETE FROM knockout_predictions
        WHERE user_id = $1 AND tournament_id = $2 AND round = $3
        "#,
        user_id,
        tournament_id,
        round as &KnockoutRound,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Unexpected(e.into()))?;

    for team_id in team_ids {
        sqlx::query!(
            r#"
            INSERT INTO knockout_predictions (user_id, tournament_id, round, team_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT DO NOTHING
            "#,
            user_id,
            tournament_id,
            round as &KnockoutRound,
            team_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    Ok(())
}

pub async fn save_top_scorer_predictions(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
    player_ids: &[i64],
) -> Result<(), AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    assert_predictions_open(&mut tx, tournament_id).await?;

    let valid_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM players WHERE tournament_id = $1 AND id = ANY($2::bigint[])",
        tournament_id,
        player_ids as &[i64],
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Unexpected(e.into()))?
    .unwrap_or(0);

    if valid_count as usize != player_ids.len() {
        return Err(AppError::BadRequest(
            "one or more player IDs are not valid for this tournament".to_string(),
        ));
    }

    sqlx::query!(
        "DELETE FROM top_scorer_predictions WHERE user_id = $1 AND tournament_id = $2",
        user_id,
        tournament_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Unexpected(e.into()))?;

    for player_id in player_ids {
        sqlx::query!(
            r#"
            INSERT INTO top_scorer_predictions (user_id, tournament_id, player_id)
            VALUES ($1, $2, $3)
            "#,
            user_id,
            tournament_id,
            player_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Unexpected(e.into()))?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use time::OffsetDateTime;

    async fn make_tournament(pool: &PgPool, locked: bool) -> i64 {
        let locked_at: Option<OffsetDateTime> = if locked {
            Some(OffsetDateTime::now_utc() - time::Duration::hours(1))
        } else {
            None
        };
        sqlx::query_scalar!(
            r#"
            INSERT INTO tournaments (external_id, name, season, is_active, predictions_locked_at)
            VALUES ('TEST-2024', 'Test Cup', '2024', TRUE, $1)
            RETURNING id
            "#,
            locked_at
        )
        .fetch_one(pool)
        .await
        .expect("insert tournament")
    }

    async fn make_user(pool: &PgPool) -> i64 {
        sqlx::query_scalar!(
            r#"INSERT INTO users (google_id, email, name) VALUES ('gid', 'u@test.com', 'U') RETURNING id"#
        )
        .fetch_one(pool)
        .await
        .expect("insert user")
    }

    /// Creates a second, inactive tournament (different external_id so it doesn't
    /// conflict with the active one created by `make_tournament`).
    async fn make_other_tournament(pool: &PgPool) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO tournaments (external_id, name, season, is_active)
            VALUES ('TEST-OTHER', 'Other Cup', '2025', FALSE)
            RETURNING id
            "#,
        )
        .fetch_one(pool)
        .await
        .expect("insert other tournament")
    }

    async fn make_team(pool: &PgPool, tournament_id: i64, ext_id: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO teams (tournament_id, external_id, name, short_name)
            VALUES ($1, $2, $2, $2)
            RETURNING id
            "#,
            tournament_id,
            ext_id,
        )
        .fetch_one(pool)
        .await
        .expect("insert team")
    }

    async fn make_player(pool: &PgPool, tournament_id: i64, team_id: i64, ext_id: &str) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO players (tournament_id, team_id, external_id, name)
            VALUES ($1, $2, $3, $3)
            RETURNING id
            "#,
            tournament_id,
            team_id,
            ext_id,
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
    ) -> i64 {
        let group_id: i64 = sqlx::query_scalar!(
            r#"
            INSERT INTO groups (tournament_id, name) VALUES ($1, 'A')
            ON CONFLICT (tournament_id, name) DO UPDATE SET name = 'A'
            RETURNING id
            "#,
            tournament_id
        )
        .fetch_one(pool)
        .await
        .expect("upsert group");

        sqlx::query_scalar!(
            r#"
            INSERT INTO matches (tournament_id, external_id, group_id, home_team_id, away_team_id, scheduled_at)
            VALUES ($1, 'M1', $2, $3, $4, NOW())
            ON CONFLICT (tournament_id, external_id) DO UPDATE SET group_id = $2
            RETURNING id
            "#,
            tournament_id, group_id, home_id, away_id,
        )
        .fetch_one(pool)
        .await
        .expect("insert match")
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn locked_tournament_rejects_group_prediction(pool: PgPool) {
        let t_id = make_tournament(&pool, true).await;
        let u_id = make_user(&pool).await;
        let home = make_team(&pool, t_id, "Home").await;
        let away = make_team(&pool, t_id, "Away").await;
        let match_id = make_group_match(&pool, t_id, home, away).await;

        let result =
            save_group_stage_predictions(&pool, t_id, u_id, &[(match_id, MatchOutcome::Home)])
                .await;

        assert!(
            matches!(result, Err(AppError::Forbidden)),
            "expected Forbidden for locked tournament, got {result:?}"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn group_stage_upsert_is_idempotent(pool: PgPool) {
        let t_id = make_tournament(&pool, false).await;
        let u_id = make_user(&pool).await;
        let home = make_team(&pool, t_id, "Home").await;
        let away = make_team(&pool, t_id, "Away").await;
        let match_id = make_group_match(&pool, t_id, home, away).await;

        // Submit home, then change to away
        save_group_stage_predictions(&pool, t_id, u_id, &[(match_id, MatchOutcome::Home)])
            .await
            .expect("first save");
        save_group_stage_predictions(&pool, t_id, u_id, &[(match_id, MatchOutcome::Away)])
            .await
            .expect("second save");

        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM group_stage_predictions WHERE match_id = $1 AND user_id = $2",
            match_id,
            u_id
        )
        .fetch_one(&pool)
        .await
        .expect("count")
        .unwrap_or(0);

        assert_eq!(count, 1, "second save must update, not duplicate");

        let outcome: MatchOutcome = sqlx::query_scalar!(
            r#"SELECT predicted_outcome AS "predicted_outcome: MatchOutcome"
               FROM group_stage_predictions WHERE match_id = $1 AND user_id = $2"#,
            match_id,
            u_id
        )
        .fetch_one(&pool)
        .await
        .expect("fetch outcome");

        assert_eq!(
            outcome,
            MatchOutcome::Away,
            "second save must update the value"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn knockout_rejects_team_from_wrong_tournament(pool: PgPool) {
        let t_a = make_tournament(&pool, false).await;
        let t_b = make_other_tournament(&pool).await;
        let u_id = make_user(&pool).await;
        let team_b = make_team(&pool, t_b, "TeamB").await;

        let result =
            save_knockout_round_predictions(&pool, t_a, u_id, &KnockoutRound::Qf, &[team_b]).await;

        assert!(
            matches!(result, Err(AppError::BadRequest(_))),
            "expected BadRequest for team from wrong tournament, got {result:?}"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn top_scorer_rejects_player_from_wrong_tournament(pool: PgPool) {
        let t_a = make_tournament(&pool, false).await;
        let t_b = make_other_tournament(&pool).await;
        let u_id = make_user(&pool).await;
        let team_b = make_team(&pool, t_b, "TeamB").await;
        let player_b = make_player(&pool, t_b, team_b, "P1").await;
        let player_b2 = make_player(&pool, t_b, team_b, "P2").await;
        let player_b3 = make_player(&pool, t_b, team_b, "P3").await;

        let result =
            save_top_scorer_predictions(&pool, t_a, u_id, &[player_b, player_b2, player_b3]).await;

        assert!(
            matches!(result, Err(AppError::BadRequest(_))),
            "expected BadRequest for players from wrong tournament, got {result:?}"
        );
    }
}

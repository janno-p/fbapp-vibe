use sqlx::PgPool;

use crate::{
    db_types::{KnockoutRound, MatchOutcome},
    flags::flag_emoji,
    modules::admin::models::Tournament,
};

use super::models::{
    CompareGroupRow, FixtureRow, LeaderboardRawRow, LeagueMember, MatchBreakdownRow,
    MatchConsensus, MatchInfo, MemberGroupPredRow, NearestMatch,
};

// ── Access guard ──────────────────────────────────────────────────────────────

pub async fn is_member(pool: &PgPool, league_id: i64, user_id: i64) -> anyhow::Result<bool> {
    let member = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM league_members WHERE league_id = $1 AND user_id = $2)",
        league_id,
        user_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);
    Ok(member)
}

// ── League meta ───────────────────────────────────────────────────────────────

pub async fn get_active_tournament_id(pool: &PgPool) -> anyhow::Result<Option<i64>> {
    let id = sqlx::query_scalar!("SELECT id FROM tournaments WHERE is_active = TRUE LIMIT 1")
        .fetch_optional(pool)
        .await?;
    Ok(id)
}

pub async fn get_active_tournament(pool: &PgPool) -> anyhow::Result<Option<Tournament>> {
    let t = sqlx::query_as!(
        Tournament,
        "SELECT id, external_id, name, season, is_active, predictions_locked_at \
         FROM tournaments WHERE is_active = TRUE LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;
    Ok(t)
}

pub async fn get_league_name(pool: &PgPool, league_id: i64) -> anyhow::Result<Option<String>> {
    let name = sqlx::query_scalar!("SELECT name FROM leagues WHERE id = $1", league_id)
        .fetch_optional(pool)
        .await?;
    Ok(name)
}

pub async fn get_league_members(
    pool: &PgPool,
    league_id: i64,
) -> anyhow::Result<Vec<LeagueMember>> {
    let rows = sqlx::query!(
        r#"
        SELECT u.id, u.name
        FROM league_members lm
        JOIN users u ON u.id = lm.user_id
        WHERE lm.league_id = $1
        ORDER BY u.name ASC
        "#,
        league_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| LeagueMember {
            id: r.id,
            name: r.name,
        })
        .collect())
}

// ── Leaderboard ───────────────────────────────────────────────────────────────

pub async fn get_leaderboard(
    pool: &PgPool,
    tournament_id: i64,
    league_id: i64,
) -> anyhow::Result<Vec<LeaderboardRawRow>> {
    // Use a `combined` CTE so the outer SELECT can reference alias names rather
    // than repeating the sum expressions (SQLx compile-time check requires column
    // names to exist before they can be used in expressions).
    let rows = sqlx::query!(
        r#"
        WITH league_users AS (
            SELECT u.id, u.name
            FROM league_members lm
            JOIN users u ON u.id = lm.user_id
            WHERE lm.league_id = $2
        ),
        gsp_earned AS (
            SELECT gsp.user_id,
                   COALESCE(SUM(gsp.points_awarded), 0)::bigint AS points
            FROM group_stage_predictions gsp
            JOIN matches m ON m.id = gsp.match_id AND m.tournament_id = $1
            WHERE gsp.user_id IN (SELECT id FROM league_users)
            GROUP BY gsp.user_id
        ),
        kp_earned AS (
            SELECT user_id,
                   COALESCE(SUM(points_awarded), 0)::bigint AS points
            FROM knockout_predictions
            WHERE tournament_id = $1
              AND user_id IN (SELECT id FROM league_users)
            GROUP BY user_id
        ),
        tsp_earned AS (
            SELECT user_id,
                   COALESCE(SUM(points_awarded), 0)::bigint AS points
            FROM top_scorer_predictions
            WHERE tournament_id = $1
              AND user_id IN (SELECT id FROM league_users)
            GROUP BY user_id
        ),
        group_possible AS (
            SELECT gsp.user_id, COUNT(*)::bigint AS possible
            FROM group_stage_predictions gsp
            JOIN matches m ON m.id = gsp.match_id AND m.tournament_id = $1
            WHERE gsp.points_awarded IS NULL
              AND m.outcome IS NULL
              AND gsp.user_id IN (SELECT id FROM league_users)
            GROUP BY gsp.user_id
        ),
        knockout_possible AS (
            SELECT kp.user_id,
                   COALESCE(SUM(
                       CASE kp.round::text
                       WHEN 'r32'    THEN 2
                       WHEN 'r16'    THEN 3
                       WHEN 'qf'     THEN 4
                       WHEN 'sf'     THEN 6
                       WHEN 'final'  THEN 8
                       WHEN 'winner' THEN 10
                       ELSE 0 END
                   ), 0)::bigint AS possible
            FROM knockout_predictions kp
            WHERE kp.tournament_id = $1
              AND kp.points_awarded IS NULL
              AND kp.user_id IN (SELECT id FROM league_users)
            GROUP BY kp.user_id
        ),
        top_scorer_possible AS (
            SELECT tsp.user_id,
                   (5 + COALESCE(MAX(p.goals_scored), 0))::bigint AS possible
            FROM top_scorer_predictions tsp
            JOIN players p ON p.id = tsp.player_id
            WHERE tsp.tournament_id = $1
              AND tsp.points_awarded IS NULL
              AND tsp.user_id IN (SELECT id FROM league_users)
            GROUP BY tsp.user_id
        ),
        combined AS (
            SELECT
                lu.id   AS user_id,
                lu.name AS user_name,
                COALESCE(gsp.points, 0) + COALESCE(kp.points, 0) + COALESCE(tsp.points, 0)  AS earned,
                COALESCE(gp.possible, 0) + COALESCE(kpp.possible, 0) + COALESCE(tspp.possible, 0) AS possible
            FROM league_users lu
            LEFT JOIN gsp_earned          gsp  ON gsp.user_id  = lu.id
            LEFT JOIN kp_earned           kp   ON kp.user_id   = lu.id
            LEFT JOIN tsp_earned          tsp  ON tsp.user_id  = lu.id
            LEFT JOIN group_possible      gp   ON gp.user_id   = lu.id
            LEFT JOIN knockout_possible   kpp  ON kpp.user_id  = lu.id
            LEFT JOIN top_scorer_possible tspp ON tspp.user_id = lu.id
        )
        SELECT
            user_id   AS "user_id!: i64",
            user_name AS "user_name!: String",
            earned    AS "total_points!: i64",
            earned + possible AS "max_achievable!: i64"
        FROM combined
        ORDER BY earned DESC, (earned + possible) DESC, user_name ASC
        "#,
        tournament_id,
        league_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| LeaderboardRawRow {
            user_id: r.user_id,
            user_name: r.user_name,
            total_points: r.total_points,
            max_achievable: r.max_achievable,
        })
        .collect())
}

// ── Match utilities ───────────────────────────────────────────────────────────

pub async fn get_nearest_match(
    pool: &PgPool,
    tournament_id: i64,
) -> anyhow::Result<Option<NearestMatch>> {
    let row = sqlx::query!(
        r#"
        SELECT m.id,
               COALESCE(ht.name, 'TBD') AS "home_name!: String",
               COALESCE(at.name, 'TBD') AS "away_name!: String",
               ht.tla AS "home_tla?: String",
               at.tla AS "away_tla?: String",
               m.scheduled_at,
               m.outcome    AS "outcome?: MatchOutcome",
               m.home_score,
               m.away_score
        FROM matches m
        LEFT JOIN teams ht ON ht.id = m.home_team_id
        LEFT JOIN teams at ON at.id = m.away_team_id
        WHERE m.tournament_id = $1
          AND m.group_id IS NOT NULL
        ORDER BY ABS(EXTRACT(EPOCH FROM (m.scheduled_at - NOW())))
        LIMIT 1
        "#,
        tournament_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| NearestMatch {
        id: r.id,
        home_emoji: flag_emoji(r.home_tla.as_deref()),
        away_emoji: flag_emoji(r.away_tla.as_deref()),
        home_name: r.home_name,
        away_name: r.away_name,
        scheduled_at: r.scheduled_at,
        outcome: r.outcome,
        home_score: r.home_score,
        away_score: r.away_score,
    }))
}

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

// ── Match breakdown ───────────────────────────────────────────────────────────

pub async fn get_match_info(
    pool: &PgPool,
    tournament_id: i64,
    match_id: i64,
) -> anyhow::Result<Option<MatchInfo>> {
    let row = sqlx::query!(
        r#"
        SELECT m.id,
               COALESCE(ht.name, 'TBD') AS "home_name!: String",
               COALESCE(at.name, 'TBD') AS "away_name!: String",
               ht.tla AS "home_tla?: String",
               at.tla AS "away_tla?: String",
               m.scheduled_at,
               m.home_score,
               m.away_score,
               m.outcome AS "outcome?: MatchOutcome"
        FROM matches m
        LEFT JOIN teams ht ON ht.id = m.home_team_id
        LEFT JOIN teams at ON at.id = m.away_team_id
        WHERE m.id = $1 AND m.tournament_id = $2
        "#,
        match_id,
        tournament_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| MatchInfo {
        id: r.id,
        home_emoji: flag_emoji(r.home_tla.as_deref()),
        away_emoji: flag_emoji(r.away_tla.as_deref()),
        home_name: r.home_name,
        away_name: r.away_name,
        scheduled_at: r.scheduled_at,
        home_score: r.home_score,
        away_score: r.away_score,
        outcome: r.outcome,
    }))
}

pub async fn get_group_match_breakdown(
    pool: &PgPool,
    league_id: i64,
    match_id: i64,
) -> anyhow::Result<Vec<MatchBreakdownRow>> {
    let rows = sqlx::query!(
        r#"
        SELECT u.id AS user_id,
               u.name AS user_name,
               gsp.predicted_outcome AS "predicted_outcome?: MatchOutcome",
               gsp.points_awarded
        FROM league_members lm
        JOIN users u ON u.id = lm.user_id
        LEFT JOIN group_stage_predictions gsp
               ON gsp.user_id = u.id AND gsp.match_id = $2
        WHERE lm.league_id = $1
        ORDER BY u.name ASC
        "#,
        league_id,
        match_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| MatchBreakdownRow {
            user_id: r.user_id,
            user_name: r.user_name,
            predicted_outcome: r.predicted_outcome,
            points_awarded: r.points_awarded,
        })
        .collect())
}

pub async fn get_match_consensus(
    pool: &PgPool,
    league_id: i64,
    match_id: i64,
) -> anyhow::Result<MatchConsensus> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE gsp.predicted_outcome = 'home') AS "home_count!: i64",
            COUNT(*) FILTER (WHERE gsp.predicted_outcome = 'draw') AS "draw_count!: i64",
            COUNT(*) FILTER (WHERE gsp.predicted_outcome = 'away') AS "away_count!: i64",
            COUNT(*) FILTER (WHERE gsp.predicted_outcome IS NULL)  AS "no_prediction_count!: i64"
        FROM league_members lm
        LEFT JOIN group_stage_predictions gsp
               ON gsp.user_id = lm.user_id AND gsp.match_id = $2
        WHERE lm.league_id = $1
        "#,
        league_id,
        match_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(MatchConsensus {
        home_count: row.home_count,
        draw_count: row.draw_count,
        away_count: row.away_count,
        no_prediction_count: row.no_prediction_count,
    })
}

// ── Comparison ────────────────────────────────────────────────────────────────

pub async fn get_compare_group_rows(
    pool: &PgPool,
    tournament_id: i64,
    user_a_id: i64,
    user_b_id: i64,
) -> anyhow::Result<Vec<CompareGroupRow>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(ht.name, 'TBD') AS "home_name!: String",
            COALESCE(at.name, 'TBD') AS "away_name!: String",
            m.scheduled_at,
            m.outcome      AS "actual_outcome?: MatchOutcome",
            m.home_score,
            m.away_score,
            a.predicted_outcome AS "a_prediction?: MatchOutcome",
            a.points_awarded    AS a_points,
            b.predicted_outcome AS "b_prediction?: MatchOutcome",
            b.points_awarded    AS b_points
        FROM matches m
        LEFT JOIN teams ht ON ht.id = m.home_team_id
        LEFT JOIN teams at ON at.id = m.away_team_id
        LEFT JOIN group_stage_predictions a ON a.match_id = m.id AND a.user_id = $2
        LEFT JOIN group_stage_predictions b ON b.match_id = m.id AND b.user_id = $3
        WHERE m.tournament_id = $1
          AND m.group_id IS NOT NULL
        ORDER BY m.scheduled_at ASC
        "#,
        tournament_id,
        user_a_id,
        user_b_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CompareGroupRow {
            home_name: r.home_name,
            away_name: r.away_name,
            scheduled_at: r.scheduled_at,
            actual_outcome: r.actual_outcome,
            home_score: r.home_score,
            away_score: r.away_score,
            a_prediction: r.a_prediction,
            a_points: r.a_points,
            b_prediction: r.b_prediction,
            b_points: r.b_points,
        })
        .collect())
}

// ── Fixtures ──────────────────────────────────────────────────────────────────

pub async fn get_all_fixtures(
    pool: &PgPool,
    tournament_id: i64,
) -> anyhow::Result<Vec<FixtureRow>> {
    let rows = sqlx::query!(
        r#"
        SELECT m.id,
               COALESCE(ht.name, 'TBD') AS "home_name!: String",
               COALESCE(at.name, 'TBD') AS "away_name!: String",
               ht.tla AS "home_tla?: String",
               at.tla AS "away_tla?: String",
               m.scheduled_at,
               m.home_score,
               m.away_score,
               m.outcome    AS "outcome?: MatchOutcome",
               g.name       AS "group_name?: String",
               m.round      AS "round?: KnockoutRound"
        FROM matches m
        LEFT JOIN teams ht ON ht.id = m.home_team_id
        LEFT JOIN teams at ON at.id = m.away_team_id
        LEFT JOIN groups g  ON g.id  = m.group_id
        WHERE m.tournament_id = $1
        ORDER BY
            CASE WHEN m.group_id IS NOT NULL THEN 0 ELSE 1 END,
            g.name ASC NULLS LAST,
            CASE m.round
                WHEN 'r32'   THEN 1
                WHEN 'r16'   THEN 2
                WHEN 'qf'    THEN 3
                WHEN 'sf'    THEN 4
                WHEN 'final' THEN 5
                ELSE 6
            END,
            m.scheduled_at ASC
        "#,
        tournament_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| FixtureRow {
            id: r.id,
            home_emoji: flag_emoji(r.home_tla.as_deref()),
            away_emoji: flag_emoji(r.away_tla.as_deref()),
            home_name: r.home_name,
            away_name: r.away_name,
            scheduled_at: r.scheduled_at,
            home_score: r.home_score,
            away_score: r.away_score,
            outcome: r.outcome,
            group_name: r.group_name,
            round: r.round,
        })
        .collect())
}

// ── Member stats ─────────────────────────────────────────────────────────────

/// Returns the target user's display name and league join date.
/// Returns `None` if the user is not a member of this league.
pub async fn get_member_info(
    pool: &PgPool,
    league_id: i64,
    user_id: i64,
) -> anyhow::Result<Option<(String, time::OffsetDateTime)>> {
    let row = sqlx::query!(
        r#"
        SELECT u.name, lm.joined_at
        FROM league_members lm
        JOIN users u ON u.id = lm.user_id
        WHERE lm.league_id = $1 AND lm.user_id = $2
        "#,
        league_id,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| (r.name, r.joined_at)))
}

/// Returns group stage prediction rows for played matches, ordered chronologically.
///
/// Only includes matches that have been decided (`outcome IS NOT NULL`), so the
/// caller can directly derive accuracy and streak from the returned slice.
pub async fn get_member_group_preds(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
) -> anyhow::Result<Vec<MemberGroupPredRow>> {
    let rows = sqlx::query!(
        r#"
        SELECT gsp.predicted_outcome AS "predicted_outcome?: MatchOutcome",
               m.outcome             AS "actual_outcome!: MatchOutcome"
        FROM group_stage_predictions gsp
        JOIN matches m ON m.id = gsp.match_id
        WHERE m.tournament_id = $1
          AND gsp.user_id = $2
          AND m.outcome IS NOT NULL
        ORDER BY m.scheduled_at ASC
        "#,
        tournament_id,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| MemberGroupPredRow {
            predicted_outcome: r.predicted_outcome,
            actual_outcome: r.actual_outcome,
        })
        .collect())
}

/// Returns `(correct, total)` for knockout predictions in the given tournament.
///
/// `total` counts predictions where `points_awarded IS NOT NULL` (match decided).
/// `correct` counts predictions where `points_awarded > 0`.
pub async fn get_member_knockout_stats(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
) -> anyhow::Result<(i64, i64)> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE points_awarded IS NOT NULL)  AS "total!: i64",
            COUNT(*) FILTER (WHERE points_awarded > 0)          AS "correct!: i64"
        FROM knockout_predictions
        WHERE tournament_id = $1
          AND user_id = $2
        "#,
        tournament_id,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok((row.correct, row.total))
}

/// Returns the sum of top scorer points awarded to the user in this tournament.
pub async fn get_member_top_scorer_points(
    pool: &PgPool,
    tournament_id: i64,
    user_id: i64,
) -> anyhow::Result<i64> {
    let pts = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(SUM(points_awarded), 0)::bigint AS "pts!: i64"
        FROM top_scorer_predictions
        WHERE tournament_id = $1 AND user_id = $2
        "#,
        tournament_id,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(pts)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use time::OffsetDateTime;

    async fn make_user(pool: &PgPool, google_id: &str, email: &str, name: &str) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO users (google_id, email, name) VALUES ($1, $2, $3) RETURNING id",
            google_id,
            email,
            name,
        )
        .fetch_one(pool)
        .await
        .expect("insert user")
    }

    async fn make_tournament(pool: &PgPool) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO tournaments (external_id, name, season, is_active) VALUES ('WC', 'Test Cup', '2026', TRUE) RETURNING id"
        )
        .fetch_one(pool)
        .await
        .expect("insert tournament")
    }

    async fn make_league(pool: &PgPool, creator_id: i64) -> i64 {
        let token = format!("tok-{creator_id}");
        sqlx::query_scalar!(
            "INSERT INTO leagues (name, invite_token, created_by) VALUES ('Test', $1, $2) RETURNING id",
            token,
            creator_id,
        )
        .fetch_one(pool)
        .await
        .expect("insert league")
    }

    async fn add_member(pool: &PgPool, league_id: i64, user_id: i64) {
        sqlx::query!(
            "INSERT INTO league_members (league_id, user_id) VALUES ($1, $2)",
            league_id,
            user_id,
        )
        .execute(pool)
        .await
        .expect("add member");
    }

    async fn make_team(pool: &PgPool, tournament_id: i64, ext: &str) -> i64 {
        sqlx::query_scalar!(
            "INSERT INTO teams (tournament_id, external_id, name, short_name) VALUES ($1, $2, $2, $2) RETURNING id",
            tournament_id, ext,
        )
        .fetch_one(pool)
        .await
        .expect("insert team")
    }

    async fn make_match(pool: &PgPool, tournament_id: i64, home: i64, away: i64) -> i64 {
        let group_id: i64 = sqlx::query_scalar!(
            "INSERT INTO groups (tournament_id, name) VALUES ($1, 'A') ON CONFLICT (tournament_id, name) DO UPDATE SET name = 'A' RETURNING id",
            tournament_id,
        )
        .fetch_one(pool)
        .await
        .expect("upsert group");

        sqlx::query_scalar!(
            r#"
            INSERT INTO matches (tournament_id, external_id, group_id, home_team_id, away_team_id, scheduled_at)
            VALUES ($1, 'M1', $2, $3, $4, $5) RETURNING id
            "#,
            tournament_id, group_id, home, away, OffsetDateTime::now_utc(),
        )
        .fetch_one(pool)
        .await
        .expect("insert match")
    }

    async fn predict(pool: &PgPool, user_id: i64, match_id: i64, outcome: Option<MatchOutcome>) {
        if let Some(o) = outcome {
            sqlx::query!(
                r#"INSERT INTO group_stage_predictions (user_id, match_id, predicted_outcome)
                   VALUES ($1, $2, $3)"#,
                user_id,
                match_id,
                o as MatchOutcome,
            )
            .execute(pool)
            .await
            .expect("insert prediction");
        }
        // None = no row inserted (member without prediction)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn consensus_all_predict_same_outcome(pool: PgPool) {
        let u1 = make_user(&pool, "g1", "a@t.com", "Alice").await;
        let u2 = make_user(&pool, "g2", "b@t.com", "Bob").await;
        let t_id = make_tournament(&pool).await;
        let league = make_league(&pool, u1).await;
        add_member(&pool, league, u1).await;
        add_member(&pool, league, u2).await;
        let home = make_team(&pool, t_id, "H").await;
        let away = make_team(&pool, t_id, "A").await;
        let m_id = make_match(&pool, t_id, home, away).await;

        predict(&pool, u1, m_id, Some(MatchOutcome::Home)).await;
        predict(&pool, u2, m_id, Some(MatchOutcome::Home)).await;

        let c = get_match_consensus(&pool, league, m_id)
            .await
            .expect("consensus");
        assert_eq!(c.home_count, 2);
        assert_eq!(c.draw_count, 0);
        assert_eq!(c.away_count, 0);
        assert_eq!(c.no_prediction_count, 0);
        assert_eq!(c.home_percentage(), 100);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn consensus_mixed_predictions(pool: PgPool) {
        let u1 = make_user(&pool, "g1", "a@t.com", "Alice").await;
        let u2 = make_user(&pool, "g2", "b@t.com", "Bob").await;
        let u3 = make_user(&pool, "g3", "c@t.com", "Carol").await;
        let t_id = make_tournament(&pool).await;
        let league = make_league(&pool, u1).await;
        add_member(&pool, league, u1).await;
        add_member(&pool, league, u2).await;
        add_member(&pool, league, u3).await;
        let home = make_team(&pool, t_id, "H").await;
        let away = make_team(&pool, t_id, "A").await;
        let m_id = make_match(&pool, t_id, home, away).await;

        predict(&pool, u1, m_id, Some(MatchOutcome::Home)).await;
        predict(&pool, u2, m_id, Some(MatchOutcome::Draw)).await;
        predict(&pool, u3, m_id, Some(MatchOutcome::Away)).await;

        let c = get_match_consensus(&pool, league, m_id)
            .await
            .expect("consensus");
        assert_eq!(c.home_count, 1);
        assert_eq!(c.draw_count, 1);
        assert_eq!(c.away_count, 1);
        assert_eq!(c.no_prediction_count, 0);
        assert_eq!(c.total_predictors(), 3);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn consensus_some_members_did_not_predict(pool: PgPool) {
        let u1 = make_user(&pool, "g1", "a@t.com", "Alice").await;
        let u2 = make_user(&pool, "g2", "b@t.com", "Bob").await;
        let u3 = make_user(&pool, "g3", "c@t.com", "Carol").await;
        let t_id = make_tournament(&pool).await;
        let league = make_league(&pool, u1).await;
        add_member(&pool, league, u1).await;
        add_member(&pool, league, u2).await;
        add_member(&pool, league, u3).await;
        let home = make_team(&pool, t_id, "H").await;
        let away = make_team(&pool, t_id, "A").await;
        let m_id = make_match(&pool, t_id, home, away).await;

        // Only u1 predicted; u2 and u3 did not
        predict(&pool, u1, m_id, Some(MatchOutcome::Home)).await;

        let c = get_match_consensus(&pool, league, m_id)
            .await
            .expect("consensus");
        assert_eq!(c.home_count, 1);
        assert_eq!(c.no_prediction_count, 2);
        assert_eq!(c.total_predictors(), 1);
        assert_eq!(c.home_percentage(), 100);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn consensus_cross_league_isolation(pool: PgPool) {
        let u1 = make_user(&pool, "g1", "a@t.com", "Alice").await;
        let u2 = make_user(&pool, "g2", "b@t.com", "Bob").await;
        let t_id = make_tournament(&pool).await;
        let league_a = make_league(&pool, u1).await;
        let league_b = make_league(&pool, u2).await;
        add_member(&pool, league_a, u1).await;
        add_member(&pool, league_b, u2).await;
        let home = make_team(&pool, t_id, "H").await;
        let away = make_team(&pool, t_id, "A").await;
        let m_id = make_match(&pool, t_id, home, away).await;

        // u1 (league_a) predicts home; u2 (league_b) predicts away
        predict(&pool, u1, m_id, Some(MatchOutcome::Home)).await;
        predict(&pool, u2, m_id, Some(MatchOutcome::Away)).await;

        // league_a consensus should only see u1's prediction
        let c = get_match_consensus(&pool, league_a, m_id)
            .await
            .expect("consensus");
        assert_eq!(c.home_count, 1);
        assert_eq!(c.away_count, 0);
        assert_eq!(c.total_predictors(), 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn is_member_returns_correct_membership(pool: PgPool) {
        let u1 = make_user(&pool, "g1", "a@t.com", "Alice").await;
        let u2 = make_user(&pool, "g2", "b@t.com", "Bob").await;
        let league = make_league(&pool, u1).await;
        add_member(&pool, league, u1).await;

        assert!(is_member(&pool, league, u1).await.expect("check u1"));
        assert!(!is_member(&pool, league, u2).await.expect("check u2"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn leaderboard_ranks_by_points(pool: PgPool) {
        let u1 = make_user(&pool, "g1", "a@t.com", "Alice").await;
        let u2 = make_user(&pool, "g2", "b@t.com", "Bob").await;
        let t_id = make_tournament(&pool).await;
        let league = make_league(&pool, u1).await;
        add_member(&pool, league, u1).await;
        add_member(&pool, league, u2).await;

        let home = make_team(&pool, t_id, "H").await;
        let away = make_team(&pool, t_id, "A").await;
        let m_id = make_match(&pool, t_id, home, away).await;

        sqlx::query!(
            "INSERT INTO group_stage_predictions (user_id, match_id, predicted_outcome, points_awarded) VALUES ($1, $2, 'home', 1)",
            u1, m_id,
        ).execute(&pool).await.expect("alice pred");
        sqlx::query!(
            "INSERT INTO group_stage_predictions (user_id, match_id, predicted_outcome, points_awarded) VALUES ($1, $2, 'away', 0)",
            u2, m_id,
        ).execute(&pool).await.expect("bob pred");

        let rows = get_leaderboard(&pool, t_id, league)
            .await
            .expect("leaderboard");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].user_id, u1, "Alice should be ranked first");
        assert_eq!(rows[0].total_points, 1);
        assert_eq!(rows[1].user_id, u2, "Bob should be ranked second");
        assert_eq!(rows[1].total_points, 0);
    }
}

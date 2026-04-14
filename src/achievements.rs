/// Achievement badge system — badge definitions, metadata, and award criteria.
/// Implements cavekit-badges.md R1, R3, R6.
use sqlx::PgPool;
use tracing::{error, info};

// ── Badge definitions ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BadgeSlug {
    PerfectGroupRound,
    UnderdogCaller,
    TopScorer,
    ConsistentPredictor,
    Oracle,
}

impl BadgeSlug {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PerfectGroupRound => "perfect_group_round",
            Self::UnderdogCaller => "underdog_caller",
            Self::TopScorer => "top_scorer",
            Self::ConsistentPredictor => "consistent_predictor",
            Self::Oracle => "oracle",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::PerfectGroupRound => "Perfect Round",
            Self::UnderdogCaller => "Underdog Caller",
            Self::TopScorer => "Top Scorer",
            Self::ConsistentPredictor => "Consistent Predictor",
            Self::Oracle => "Oracle",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::PerfectGroupRound => {
                "Predicted all matches in a group stage day correctly"
            }
            Self::UnderdogCaller => {
                "Correctly predicted 3 or more away wins"
            }
            Self::TopScorer => "Finished #1 on the leaderboard at tournament end",
            Self::ConsistentPredictor => {
                "Achieved over 70% accuracy in group stage predictions"
            }
            Self::Oracle => "Correctly predicted the tournament winner",
        }
    }

    pub fn emoji(&self) -> char {
        match self {
            Self::PerfectGroupRound => '🎯',
            Self::UnderdogCaller => '🐉',
            Self::TopScorer => '🏆',
            Self::ConsistentPredictor => '📊',
            Self::Oracle => '🔮',
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::PerfectGroupRound,
            Self::UnderdogCaller,
            Self::TopScorer,
            Self::ConsistentPredictor,
            Self::Oracle,
        ]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "perfect_group_round" => Some(Self::PerfectGroupRound),
            "underdog_caller" => Some(Self::UnderdogCaller),
            "top_scorer" => Some(Self::TopScorer),
            "consistent_predictor" => Some(Self::ConsistentPredictor),
            "oracle" => Some(Self::Oracle),
            _ => None,
        }
    }
}

// ── Badge data for display ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BadgeDisplay {
    pub slug: String,
    pub name: &'static str,
    pub description: &'static str,
    pub emoji: char,
    pub awarded_at: time::OffsetDateTime,
}

// ── Award job ─────────────────────────────────────────────────────────────

/// Data needed to evaluate badge criteria — loaded once per award job run.
struct AwardContext {
    tournament_id: i64,
    all_user_ids: Vec<i64>,
}

/// Runs badge award evaluation for all users in the active tournament.
/// Idempotent — the unique constraint on user_achievements prevents duplicates.
/// R3: job runs after scoring, logs awards, continues on per-user errors.
pub async fn run_badge_award_job(pool: &PgPool, tournament_id: i64) -> anyhow::Result<()> {
    let ctx = load_award_context(pool, tournament_id).await?;
    if ctx.all_user_ids.is_empty() {
        return Ok(());
    }

    for badge in BadgeSlug::all() {
        if let Err(e) = evaluate_and_award(pool, &ctx, badge).await {
            error!(
                badge = badge.as_str(),
                tournament_id,
                "Badge evaluation failed: {e:#}"
            );
        }
    }

    Ok(())
}

async fn load_award_context(pool: &PgPool, tournament_id: i64) -> anyhow::Result<AwardContext> {
    // Get all users who have made predictions in this tournament
    // (leagues are not directly linked to tournaments; all league members share the active tournament)
    let rows = sqlx::query_scalar!(
        r#"SELECT DISTINCT gsp.user_id
           FROM group_stage_predictions gsp
           JOIN matches m ON m.id = gsp.match_id
           WHERE m.tournament_id = $1"#,
        tournament_id
    )
    .fetch_all(pool)
    .await?;

    Ok(AwardContext {
        tournament_id,
        all_user_ids: rows,
    })
}

async fn evaluate_and_award(
    pool: &PgPool,
    ctx: &AwardContext,
    badge: BadgeSlug,
) -> anyhow::Result<()> {
    let eligible = match badge {
        BadgeSlug::PerfectGroupRound => {
            evaluate_perfect_group_round(pool, ctx).await?
        }
        BadgeSlug::UnderdogCaller => evaluate_underdog_caller(pool, ctx).await?,
        BadgeSlug::TopScorer => evaluate_top_scorer(pool, ctx).await?,
        BadgeSlug::ConsistentPredictor => {
            evaluate_consistent_predictor(pool, ctx).await?
        }
        BadgeSlug::Oracle => evaluate_oracle(pool, ctx).await?,
    };

    for user_id in eligible {
        let inserted = sqlx::query!(
            "INSERT INTO user_achievements (user_id, tournament_id, badge_slug)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, tournament_id, badge_slug) DO NOTHING",
            user_id,
            ctx.tournament_id,
            badge.as_str()
        )
        .execute(pool)
        .await?
        .rows_affected();

        if inserted > 0 {
            info!(
                user_id,
                badge = badge.as_str(),
                tournament_id = ctx.tournament_id,
                "Badge awarded: user_id={}, badge={}, tournament_id={}",
                user_id,
                badge.as_str(),
                ctx.tournament_id
            );
        }
    }

    Ok(())
}

// ── Per-badge criteria ─────────────────────────────────────────────────────

/// All group stage match days where user predicted every match correctly.
async fn evaluate_perfect_group_round(
    pool: &PgPool,
    ctx: &AwardContext,
) -> anyhow::Result<Vec<i64>> {
    let rows = sqlx::query_scalar!(
        r#"
        SELECT DISTINCT gsp.user_id
        FROM group_stage_predictions gsp
        JOIN matches m ON m.id = gsp.match_id
        WHERE m.tournament_id = $1
          AND m.group_id IS NOT NULL
          AND m.outcome IS NOT NULL
          AND gsp.user_id = ANY($2)
        GROUP BY gsp.user_id, DATE(m.scheduled_at)
        HAVING COUNT(*) FILTER (WHERE gsp.predicted_outcome = m.outcome) = COUNT(*)
           AND COUNT(*) > 0
        "#,
        ctx.tournament_id,
        &ctx.all_user_ids
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Users with >= 3 correct away-win predictions in group stage.
async fn evaluate_underdog_caller(
    pool: &PgPool,
    ctx: &AwardContext,
) -> anyhow::Result<Vec<i64>> {
    let rows = sqlx::query_scalar!(
        r#"
        SELECT gsp.user_id
        FROM group_stage_predictions gsp
        JOIN matches m ON m.id = gsp.match_id
        WHERE m.tournament_id = $1
          AND m.group_id IS NOT NULL
          AND m.outcome = 'away'
          AND gsp.predicted_outcome = 'away'
          AND gsp.user_id = ANY($2)
        GROUP BY gsp.user_id
        HAVING COUNT(*) >= 3
        "#,
        ctx.tournament_id,
        &ctx.all_user_ids
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// User ranked #1 on the final leaderboard (all matches finished).
async fn evaluate_top_scorer(
    pool: &PgPool,
    ctx: &AwardContext,
) -> anyhow::Result<Vec<i64>> {
    // Only award after all matches in the tournament are finished (outcome set)
    let unfinished: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM matches WHERE tournament_id = $1 AND outcome IS NULL",
        ctx.tournament_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    if unfinished > 0 {
        return Ok(vec![]);
    }

    let top_user: Option<i64> = sqlx::query_scalar!(
        r#"
        SELECT user_id
        FROM (
            SELECT gsp.user_id,
                   SUM(CASE WHEN gsp.predicted_outcome = m.outcome THEN 1 ELSE 0 END) AS pts
            FROM group_stage_predictions gsp
            JOIN matches m ON m.id = gsp.match_id
            WHERE m.tournament_id = $1 AND m.group_id IS NOT NULL AND m.outcome IS NOT NULL
              AND gsp.user_id = ANY($2)
            GROUP BY gsp.user_id
        ) sub
        ORDER BY pts DESC
        LIMIT 1
        "#,
        ctx.tournament_id,
        &ctx.all_user_ids
    )
    .fetch_optional(pool)
    .await?;

    Ok(top_user.into_iter().collect())
}

/// Users with group stage accuracy > 70%.
async fn evaluate_consistent_predictor(
    pool: &PgPool,
    ctx: &AwardContext,
) -> anyhow::Result<Vec<i64>> {
    let rows = sqlx::query_scalar!(
        r#"
        SELECT gsp.user_id
        FROM group_stage_predictions gsp
        JOIN matches m ON m.id = gsp.match_id
        WHERE m.tournament_id = $1
          AND m.group_id IS NOT NULL
          AND m.outcome IS NOT NULL
          AND gsp.user_id = ANY($2)
        GROUP BY gsp.user_id
        HAVING COUNT(*) > 0
           AND (COUNT(*) FILTER (WHERE gsp.predicted_outcome = m.outcome))::float /
               COUNT(*)::float > 0.70
        "#,
        ctx.tournament_id,
        &ctx.all_user_ids
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Users who predicted the tournament winner in the knockout round.
async fn evaluate_oracle(
    pool: &PgPool,
    ctx: &AwardContext,
) -> anyhow::Result<Vec<i64>> {
    // Find the winning team (team that won the FINAL match as home or away winner)
    let winner_team_id: Option<i64> = sqlx::query_scalar!(
        r#"
        SELECT
            CASE WHEN m.outcome = 'home' THEN m.home_team_id
                 WHEN m.outcome = 'away' THEN m.away_team_id
                 ELSE NULL
            END AS "winner_team_id: i64"
        FROM matches m
        WHERE m.tournament_id = $1
          AND m.round = 'final'
          AND m.outcome IS NOT NULL
        LIMIT 1
        "#,
        ctx.tournament_id
    )
    .fetch_optional(pool)
    .await?
    .flatten();

    let Some(winner_id) = winner_team_id else {
        return Ok(vec![]);
    };

    // Users who predicted that team as the winner
    let rows = sqlx::query_scalar!(
        r#"
        SELECT kp.user_id
        FROM knockout_predictions kp
        WHERE kp.tournament_id = $1
          AND kp.round = 'final'
          AND kp.team_id = $2
          AND kp.user_id = ANY($3)
        "#,
        ctx.tournament_id,
        winner_id,
        &ctx.all_user_ids
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

// ── Query helpers for display ──────────────────────────────────────────────

/// Fetch all badges earned by a user in a tournament, ordered chronologically.
pub async fn get_user_badges(
    pool: &PgPool,
    user_id: i64,
    tournament_id: i64,
) -> anyhow::Result<Vec<BadgeDisplay>> {
    let rows = sqlx::query!(
        r#"SELECT badge_slug, awarded_at FROM user_achievements
           WHERE user_id = $1 AND tournament_id = $2
           ORDER BY awarded_at ASC"#,
        user_id,
        tournament_id
    )
    .fetch_all(pool)
    .await?;

    let badges: Vec<BadgeDisplay> = rows
        .into_iter()
        .filter_map(|r| {
            BadgeSlug::from_str(&r.badge_slug).map(|slug| BadgeDisplay {
                slug: slug.as_str().to_string(),
                name: slug.name(),
                description: slug.description(),
                emoji: slug.emoji(),
                awarded_at: r.awarded_at,
            })
        })
        .collect();

    Ok(badges)
}

/// Returns the most recently awarded badge for each user, keyed by user_id.
/// Used to show a "top badge" column on the leaderboard.
pub async fn get_top_badge_per_user(
    pool: &PgPool,
    tournament_id: i64,
) -> anyhow::Result<std::collections::HashMap<i64, BadgeDisplay>> {
    // DISTINCT ON returns the latest badge per user (ordered by awarded_at DESC)
    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT ON (user_id)
               user_id, badge_slug, awarded_at
        FROM user_achievements
        WHERE tournament_id = $1
        ORDER BY user_id, awarded_at DESC
        "#,
        tournament_id
    )
    .fetch_all(pool)
    .await?;

    let map = rows
        .into_iter()
        .filter_map(|r| {
            BadgeSlug::from_str(&r.badge_slug).map(|slug| {
                (
                    r.user_id,
                    BadgeDisplay {
                        slug: slug.as_str().to_string(),
                        name: slug.name(),
                        description: slug.description(),
                        emoji: slug.emoji(),
                        awarded_at: r.awarded_at,
                    },
                )
            })
        })
        .collect();

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_badges_have_five_entries() {
        assert_eq!(BadgeSlug::all().len(), 5);
    }

    #[test]
    fn badge_roundtrip_from_str() {
        for badge in BadgeSlug::all() {
            let slug = badge.as_str();
            assert_eq!(BadgeSlug::from_str(slug), Some(badge), "roundtrip failed for {slug}");
        }
    }

    #[test]
    fn unknown_slug_returns_none() {
        assert_eq!(BadgeSlug::from_str("not_a_badge"), None);
    }

    #[test]
    fn all_badges_have_non_empty_metadata() {
        for badge in BadgeSlug::all() {
            assert!(!badge.name().is_empty(), "{:?} name empty", badge);
            assert!(!badge.description().is_empty(), "{:?} description empty", badge);
            assert!(badge.emoji() != '\0', "{:?} emoji empty", badge);
            assert!(!badge.as_str().is_empty(), "{:?} slug empty", badge);
        }
    }

    // ── Integration tests (require TEST_DATABASE_URL) ─────────────────────────

    async fn insert_tournament(pool: &PgPool, external_id: &str) -> i64 {
        sqlx::query!(
            "INSERT INTO tournaments (name, external_id, season, is_active)
             VALUES ('Test Cup', $1, '2024', TRUE)
             RETURNING id",
            external_id
        )
        .fetch_one(pool)
        .await
        .expect("insert tournament")
        .id
    }

    async fn insert_group(pool: &PgPool, tournament_id: i64, name: &str) -> i64 {
        sqlx::query!(
            "INSERT INTO groups (tournament_id, name) VALUES ($1, $2) RETURNING id",
            tournament_id,
            name
        )
        .fetch_one(pool)
        .await
        .expect("insert group")
        .id
    }

    async fn insert_team(pool: &PgPool, tournament_id: i64, external_id: &str, name: &str) -> i64 {
        sqlx::query!(
            "INSERT INTO teams (tournament_id, external_id, name, short_name) VALUES ($1, $2, $3, $4) RETURNING id",
            tournament_id,
            external_id,
            name,
            name
        )
        .fetch_one(pool)
        .await
        .expect("insert team")
        .id
    }

    async fn insert_user(pool: &PgPool, google_id: &str, email: &str) -> i64 {
        sqlx::query!(
            "INSERT INTO users (google_id, email, name) VALUES ($1, $2, 'Tester') RETURNING id",
            google_id,
            email
        )
        .fetch_one(pool)
        .await
        .expect("insert user")
        .id
    }

    async fn insert_match(
        pool: &PgPool,
        tournament_id: i64,
        external_id: &str,
        home: i64,
        away: i64,
        group_id: i64,
        outcome: crate::db_types::MatchOutcome,
    ) -> i64 {
        sqlx::query!(
            r#"INSERT INTO matches
               (tournament_id, external_id, home_team_id, away_team_id,
                group_id, scheduled_at, outcome)
               VALUES ($1, $2, $3, $4, $5, NOW() - interval '1 hour', $6)
               RETURNING id"#,
            tournament_id,
            external_id,
            home,
            away,
            group_id,
            outcome as crate::db_types::MatchOutcome,
        )
        .fetch_one(pool)
        .await
        .expect("insert match")
        .id
    }

    async fn insert_prediction(
        pool: &PgPool,
        user_id: i64,
        match_id: i64,
        outcome: crate::db_types::MatchOutcome,
    ) {
        sqlx::query!(
            "INSERT INTO group_stage_predictions (user_id, match_id, predicted_outcome)
             VALUES ($1, $2, $3)",
            user_id,
            match_id,
            outcome as crate::db_types::MatchOutcome,
        )
        .execute(pool)
        .await
        .expect("insert prediction");
    }

    async fn badge_count(pool: &PgPool, user_id: i64, tournament_id: i64, slug: &str) -> i64 {
        sqlx::query!(
            "SELECT COUNT(*) AS cnt FROM user_achievements WHERE user_id = $1 AND tournament_id = $2 AND badge_slug = $3",
            user_id,
            tournament_id,
            slug
        )
        .fetch_one(pool)
        .await
        .expect("count badges")
        .cnt
        .unwrap_or(0)
    }

    /// Sets up a tournament, group, teams, and matches with known outcomes, then seeds
    /// a user's predictions and runs the badge award job. Verifies the expected badge
    /// is written to `user_achievements` and is idempotent on a second run.
    #[sqlx::test(migrations = "./migrations")]
    async fn consistent_predictor_badge_awarded_and_idempotent(pool: PgPool) {
        use crate::db_types::MatchOutcome;

        let t_id = insert_tournament(&pool, "T-BADGE-1").await;
        let g_id = insert_group(&pool, t_id, "Group A").await;
        let home = insert_team(&pool, t_id, "TA", "Team A").await;
        let away = insert_team(&pool, t_id, "TB", "Team B").await;
        let user_id = insert_user(&pool, "G-BADGE", "badge@test.com").await;

        // 4 matches: user predicts Home for all; 3 are correct (75% > 70%)
        let outcomes = [MatchOutcome::Home, MatchOutcome::Home, MatchOutcome::Home, MatchOutcome::Away];
        for (i, outcome) in outcomes.iter().enumerate() {
            let mid = insert_match(&pool, t_id, &format!("M-B1-{i}"), home, away, g_id, outcome.clone()).await;
            insert_prediction(&pool, user_id, mid, MatchOutcome::Home).await;
        }

        run_badge_award_job(&pool, t_id).await.expect("badge job");

        assert_eq!(
            badge_count(&pool, user_id, t_id, "consistent_predictor").await,
            1,
            "consistent_predictor badge not awarded"
        );

        // Idempotency: second run must not insert a duplicate
        run_badge_award_job(&pool, t_id).await.expect("badge job second run");
        assert_eq!(
            badge_count(&pool, user_id, t_id, "consistent_predictor").await,
            1,
            "badge must not be awarded twice"
        );
    }

    /// Users below the 70% threshold do not get consistent_predictor badge.
    #[sqlx::test(migrations = "./migrations")]
    async fn consistent_predictor_not_awarded_below_threshold(pool: PgPool) {
        use crate::db_types::MatchOutcome;

        let t_id = insert_tournament(&pool, "T-BADGE-2").await;
        let g_id = insert_group(&pool, t_id, "Group B").await;
        let home = insert_team(&pool, t_id, "TC", "Team C").await;
        let away = insert_team(&pool, t_id, "TD", "Team D").await;
        let user_id = insert_user(&pool, "G-BADGE-2", "badge2@test.com").await;

        // 2 matches: both outcome=Away, user predicts Home → 0/2 = 0% < 70%
        for i in 0..2i64 {
            let mid = insert_match(&pool, t_id, &format!("M-B2-{i}"), home, away, g_id, MatchOutcome::Away).await;
            insert_prediction(&pool, user_id, mid, MatchOutcome::Home).await;
        }

        run_badge_award_job(&pool, t_id).await.expect("badge job");

        assert_eq!(
            badge_count(&pool, user_id, t_id, "consistent_predictor").await,
            0,
            "badge must not be awarded below threshold"
        );
    }
}

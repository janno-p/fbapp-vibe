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
}

use sqlx::PgPool;

use crate::error::AppError;

use super::models::{League, LeagueMember, LeagueOverview, LeagueWithCount};

pub async fn create_league(
    pool: &PgPool,
    name: &str,
    invite_token: &str,
    created_by: i64,
) -> anyhow::Result<League> {
    let league = sqlx::query_as!(
        League,
        r#"
        INSERT INTO leagues (name, invite_token, created_by)
        VALUES ($1, $2, $3)
        RETURNING id, name, invite_token, created_by, created_at
        "#,
        name,
        invite_token,
        created_by,
    )
    .fetch_one(pool)
    .await?;
    Ok(league)
}

pub async fn list_leagues_with_counts(pool: &PgPool) -> anyhow::Result<Vec<LeagueWithCount>> {
    let rows = sqlx::query!(
        r#"
        SELECT l.id, l.name, l.invite_token,
               COUNT(lm.user_id)::bigint as "member_count!"
        FROM leagues l
        LEFT JOIN league_members lm ON l.id = lm.league_id
        GROUP BY l.id
        ORDER BY l.created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| LeagueWithCount {
            id: r.id,
            name: r.name,
            invite_token: r.invite_token,
            member_count: r.member_count,
        })
        .collect())
}

pub async fn find_league_by_token(pool: &PgPool, token: &str) -> anyhow::Result<Option<League>> {
    let league = sqlx::query_as!(
        League,
        r#"
        SELECT id, name, invite_token, created_by, created_at
        FROM leagues
        WHERE invite_token = $1
        "#,
        token
    )
    .fetch_optional(pool)
    .await?;
    Ok(league)
}

pub async fn join_league(pool: &PgPool, league_id: i64, user_id: i64) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO league_members (league_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
        league_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_user_leagues(pool: &PgPool, user_id: i64) -> anyhow::Result<Vec<League>> {
    let leagues = sqlx::query_as!(
        League,
        r#"
        SELECT l.id, l.name, l.invite_token, l.created_by, l.created_at
        FROM leagues l
        JOIN league_members lm ON l.id = lm.league_id
        WHERE lm.user_id = $1
        ORDER BY lm.joined_at ASC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;
    Ok(leagues)
}

pub async fn is_member(pool: &PgPool, league_id: i64, user_id: i64) -> anyhow::Result<bool> {
    let exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM league_members
            WHERE league_id = $1 AND user_id = $2
        ) as "exists!"
        "#,
        league_id,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

pub async fn get_league_overview(
    pool: &PgPool,
    league_id: i64,
) -> anyhow::Result<Option<LeagueOverview>> {
    let league = sqlx::query!(
        r#"
        SELECT id, name, invite_token, created_by, created_at
        FROM leagues
        WHERE id = $1
        "#,
        league_id
    )
    .fetch_optional(pool)
    .await?;

    let Some(league) = league else {
        return Ok(None);
    };

    let members = sqlx::query!(
        r#"
        SELECT u.name, lm.joined_at
        FROM league_members lm
        JOIN users u ON u.id = lm.user_id
        WHERE lm.league_id = $1
        ORDER BY lm.joined_at ASC
        "#,
        league_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| LeagueMember {
        name: r.name,
        joined_at: r.joined_at,
    })
    .collect();

    Ok(Some(LeagueOverview {
        id: league.id,
        name: league.name,
        created_by: league.created_by,
        created_at: league.created_at,
        invite_token: Some(league.invite_token),
        members,
    }))
}

pub async fn get_league_by_token_or_404(pool: &PgPool, token: &str) -> Result<League, AppError> {
    find_league_by_token(pool, token)
        .await?
        .ok_or(AppError::NotFound)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn make_user(pool: &PgPool) -> i64 {
        sqlx::query_scalar!(
            r#"
            INSERT INTO users (google_id, email, name)
            VALUES ('test-google-id', 'test@example.com', 'Test User')
            RETURNING id
            "#
        )
        .fetch_one(pool)
        .await
        .expect("insert user")
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn join_valid_token(pool: PgPool) {
        let user_id = make_user(&pool).await;
        let league = create_league(&pool, "Test League", "tok-abc", user_id)
            .await
            .expect("create league");

        join_league(&pool, league.id, user_id).await.expect("join");

        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM league_members WHERE league_id = $1 AND user_id = $2",
            league.id,
            user_id
        )
        .fetch_one(&pool)
        .await
        .expect("count")
        .unwrap_or(0);

        assert_eq!(count, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn join_is_idempotent(pool: PgPool) {
        let user_id = make_user(&pool).await;
        let league = create_league(&pool, "Test League", "tok-idem", user_id)
            .await
            .expect("create league");

        join_league(&pool, league.id, user_id)
            .await
            .expect("first join");
        join_league(&pool, league.id, user_id)
            .await
            .expect("second join");

        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM league_members WHERE league_id = $1",
            league.id
        )
        .fetch_one(&pool)
        .await
        .expect("count")
        .unwrap_or(0);

        assert_eq!(count, 1, "duplicate join must not insert a second row");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn invalid_token_returns_not_found(pool: PgPool) {
        let result = get_league_by_token_or_404(&pool, "nonexistent-token").await;
        assert!(
            matches!(result, Err(AppError::NotFound)),
            "expected NotFound, got {result:?}"
        );
    }
}

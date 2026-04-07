use sqlx::PgPool;

use crate::modules::auth::User;

/// Navigation context passed to every authenticated page template.
#[derive(Debug, Clone)]
pub struct NavContext {
    pub user_name: String,
    pub is_admin: bool,
    pub current_route: &'static str,
    /// League ID for the Standings nav link — `Some` only when an active
    /// tournament exists and the user belongs to at least one league.
    pub standings_league_id: Option<i64>,
}

/// Build navigation context for the given authenticated user.
///
/// The standings link is only included when there is an active tournament and
/// the user is a member of at least one league.
pub async fn load(
    pool: &PgPool,
    user: &User,
    current_route: &'static str,
) -> anyhow::Result<NavContext> {
    let standings_league_id = sqlx::query_scalar!(
        r#"
        SELECT lm.league_id
        FROM   league_members lm
        WHERE  lm.user_id = $1
          AND  EXISTS (SELECT 1 FROM tournaments WHERE is_active = TRUE)
        ORDER BY lm.joined_at ASC
        LIMIT 1
        "#,
        user.id
    )
    .fetch_optional(pool)
    .await?;

    Ok(NavContext {
        user_name: user.name.clone(),
        is_admin: user.is_admin,
        current_route,
        standings_league_id,
    })
}

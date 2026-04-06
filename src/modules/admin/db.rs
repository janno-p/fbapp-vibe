use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use time::format_description::well_known::Rfc3339;

use crate::db_types::KnockoutRound;
use crate::football_api::{Match as ApiMatch, Player as ApiPlayer, Team as ApiTeam};

use super::models::Tournament;

// ── Tournament queries ────────────────────────────────────────────────────────

pub async fn list_tournaments(pool: &PgPool) -> anyhow::Result<Vec<Tournament>> {
    let rows = sqlx::query_as!(
        Tournament,
        r#"
        SELECT id, external_id, name, season, is_active, predictions_locked_at
        FROM tournaments
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create_tournament(
    pool: &PgPool,
    external_id: &str,
    name: &str,
    season: &str,
) -> anyhow::Result<i64> {
    let row = sqlx::query!(
        r#"
        INSERT INTO tournaments (external_id, name, season)
        VALUES ($1, $2, $3)
        ON CONFLICT (external_id) DO UPDATE
            SET name = EXCLUDED.name,
                season = EXCLUDED.season
        RETURNING id
        "#,
        external_id,
        name,
        season
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

pub async fn activate_tournament(pool: &PgPool, id: i64) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE tournaments SET is_active = TRUE WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn deactivate_tournament(pool: &PgPool, id: i64) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE tournaments SET is_active = FALSE WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn lock_tournament(pool: &PgPool, id: i64) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE tournaments SET predictions_locked_at = NOW() WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unlock_tournament(pool: &PgPool, id: i64) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE tournaments SET predictions_locked_at = NULL WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ── Seeding ───────────────────────────────────────────────────────────────────

/// Seeds all tournament data from API responses. Idempotent — safe to run multiple times.
pub async fn seed_tournament_data(
    pool: &PgPool,
    tournament_id: i64,
    api_teams: &[ApiTeam],
    api_matches: &[ApiMatch],
) -> anyhow::Result<()> {
    // 1. Upsert teams; build external_id → db_id map
    let mut team_id_map: HashMap<i64, i64> = HashMap::new();
    for team in api_teams {
        let db_id = upsert_team(pool, tournament_id, team).await?;
        team_id_map.insert(team.id, db_id);
    }

    // 2. Derive group names from group-stage matches; upsert groups
    let group_names: HashSet<String> = api_matches
        .iter()
        .filter(|m| m.stage == "GROUP_STAGE")
        .filter_map(|m| m.group.clone())
        .collect();

    let mut group_id_map: HashMap<String, i64> = HashMap::new();
    for name in &group_names {
        let db_id = upsert_group(pool, tournament_id, name).await?;
        group_id_map.insert(name.clone(), db_id);
    }

    // 3. Upsert players from team squads
    for team in api_teams {
        if let Some(&db_team_id) = team_id_map.get(&team.id) {
            for player in &team.squad {
                upsert_player(pool, tournament_id, db_team_id, player).await?;
            }
        }
    }

    // 4. Upsert group memberships and matches
    for m in api_matches {
        let (group_id, round) = match m.stage.as_str() {
            "GROUP_STAGE" => {
                let gid = m.group.as_ref().and_then(|g| group_id_map.get(g)).copied();
                (gid, None)
            }
            other => (None, api_stage_to_round(other)),
        };

        let home_team_id = m.home_team.id.and_then(|id| team_id_map.get(&id)).copied();
        let away_team_id = m.away_team.id.and_then(|id| team_id_map.get(&id)).copied();

        // Insert group memberships for group-stage matches
        if let Some(gid) = group_id {
            if let Some(htid) = home_team_id {
                upsert_group_membership(pool, gid, htid).await?;
            }
            if let Some(atid) = away_team_id {
                upsert_group_membership(pool, gid, atid).await?;
            }
        }

        upsert_match(pool, tournament_id, m, group_id, round, home_team_id, away_team_id).await?;
    }

    Ok(())
}

fn api_stage_to_round(stage: &str) -> Option<KnockoutRound> {
    match stage {
        "LAST_16" | "ROUND_OF_16" => Some(KnockoutRound::R16),
        "QUARTER_FINALS" => Some(KnockoutRound::Qf),
        "SEMI_FINALS" => Some(KnockoutRound::Sf),
        "FINAL" => Some(KnockoutRound::Final),
        _ => None,
    }
}

async fn upsert_team(pool: &PgPool, tournament_id: i64, team: &ApiTeam) -> anyhow::Result<i64> {
    let row = sqlx::query!(
        r#"
        INSERT INTO teams (tournament_id, external_id, name, short_name, crest_url)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (tournament_id, external_id) DO UPDATE
            SET name      = EXCLUDED.name,
                short_name = EXCLUDED.short_name,
                crest_url  = EXCLUDED.crest_url
        RETURNING id
        "#,
        tournament_id,
        team.id.to_string(),
        team.name,
        team.short_name.as_deref().unwrap_or(&team.name),
        team.crest.as_deref()
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

async fn upsert_group(pool: &PgPool, tournament_id: i64, name: &str) -> anyhow::Result<i64> {
    let row = sqlx::query!(
        r#"
        INSERT INTO groups (tournament_id, name)
        VALUES ($1, $2)
        ON CONFLICT (tournament_id, name) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
        tournament_id,
        name
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

async fn upsert_player(
    pool: &PgPool,
    tournament_id: i64,
    team_id: i64,
    player: &ApiPlayer,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO players (tournament_id, external_id, name, team_id)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (tournament_id, external_id) DO UPDATE
            SET name    = EXCLUDED.name,
                team_id = EXCLUDED.team_id
        "#,
        tournament_id,
        player.id.to_string(),
        player.name,
        team_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn upsert_group_membership(
    pool: &PgPool,
    group_id: i64,
    team_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO group_memberships (group_id, team_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        group_id,
        team_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn upsert_match(
    pool: &PgPool,
    tournament_id: i64,
    m: &ApiMatch,
    group_id: Option<i64>,
    round: Option<KnockoutRound>,
    home_team_id: Option<i64>,
    away_team_id: Option<i64>,
) -> anyhow::Result<()> {
    let scheduled_at = time::OffsetDateTime::parse(&m.utc_date, &Rfc3339)
        .map_err(|e| anyhow::anyhow!("failed to parse match date '{}': {e}", m.utc_date))?;

    sqlx::query!(
        r#"
        INSERT INTO matches
            (tournament_id, external_id, group_id, round, home_team_id, away_team_id, scheduled_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (tournament_id, external_id) DO UPDATE
            SET group_id     = EXCLUDED.group_id,
                round        = EXCLUDED.round,
                home_team_id = EXCLUDED.home_team_id,
                away_team_id = EXCLUDED.away_team_id,
                scheduled_at = EXCLUDED.scheduled_at
        "#,
        tournament_id,
        m.id.to_string(),
        group_id,
        round as Option<KnockoutRound>,
        home_team_id,
        away_team_id,
        scheduled_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::football_api::{Match as ApiMatch, MatchScore, MatchStatus, MatchTeam, ScoreDetail, Team as ApiTeam};

    fn make_team(id: i64, name: &str) -> ApiTeam {
        ApiTeam {
            id,
            name: name.to_string(),
            short_name: Some(name[..3.min(name.len())].to_string()),
            tla: Some(name[..3.min(name.len())].to_uppercase()),
            crest: None,
            squad: vec![],
        }
    }

    fn make_match(id: i64, stage: &str, group: Option<&str>, home_id: i64, away_id: i64) -> ApiMatch {
        ApiMatch {
            id,
            utc_date: "2024-06-14T21:00:00Z".to_string(),
            status: MatchStatus::Scheduled,
            stage: stage.to_string(),
            group: group.map(str::to_string),
            home_team: MatchTeam { id: Some(home_id), name: None },
            away_team: MatchTeam { id: Some(away_id), name: None },
            score: MatchScore {
                winner: None,
                full_time: ScoreDetail { home: None, away: None },
            },
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn seed_is_idempotent(pool: PgPool) {
        // Create a tournament to seed into
        let tournament_id = create_tournament(&pool, "TEST-2024", "Test Cup", "2024")
            .await
            .expect("create tournament");

        let teams = vec![make_team(1, "Alpha"), make_team(2, "Beta")];
        let matches = vec![make_match(101, "GROUP_STAGE", Some("GROUP_A"), 1, 2)];

        // First seed
        seed_tournament_data(&pool, tournament_id, &teams, &matches)
            .await
            .expect("first seed");

        let team_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM teams")
            .fetch_one(&pool)
            .await
            .expect("count")
            .unwrap_or(0);
        assert_eq!(team_count, 2);

        // Second seed with updated name — should update, not duplicate
        let updated_teams = vec![make_team(1, "Alpha FC"), make_team(2, "Beta")];
        seed_tournament_data(&pool, tournament_id, &updated_teams, &matches)
            .await
            .expect("second seed");

        let team_count_after: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM teams")
            .fetch_one(&pool)
            .await
            .expect("count")
            .unwrap_or(0);
        assert_eq!(team_count_after, 2, "re-seeding must not duplicate rows");

        let updated_name: String = sqlx::query_scalar!(
            "SELECT name FROM teams WHERE external_id = '1'"
        )
        .fetch_one(&pool)
        .await
        .expect("fetch name");
        assert_eq!(updated_name, "Alpha FC", "re-seeding must update existing rows");
    }
}

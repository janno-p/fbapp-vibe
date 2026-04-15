use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use time::format_description::well_known::Rfc3339;

use crate::db_types::KnockoutRound;
use crate::football_api::{Match as ApiMatch, Team as ApiTeam};
use crate::national_flags::tla_to_flag;

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
    sqlx::query!("UPDATE tournaments SET is_active = TRUE WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn deactivate_tournament(pool: &PgPool, id: i64) -> anyhow::Result<()> {
    sqlx::query!("UPDATE tournaments SET is_active = FALSE WHERE id = $1", id)
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

    // 3. Upsert players from team squads — single bulk query via UNNEST
    let mut p_team_ids: Vec<i64> = vec![];
    let mut p_ext_ids: Vec<String> = vec![];
    let mut p_names: Vec<String> = vec![];

    for team in api_teams {
        if let Some(&db_team_id) = team_id_map.get(&team.id) {
            for player in &team.squad {
                p_team_ids.push(db_team_id);
                p_ext_ids.push(player.id.to_string());
                p_names.push(player.name.clone());
            }
        }
    }

    if !p_team_ids.is_empty() {
        sqlx::query!(
            r#"
            INSERT INTO players (tournament_id, external_id, name, team_id)
            SELECT $1, unnest($2::text[]), unnest($3::text[]), unnest($4::bigint[])
            ON CONFLICT (tournament_id, external_id) DO UPDATE
                SET name = EXCLUDED.name, team_id = EXCLUDED.team_id
            "#,
            tournament_id,
            &p_ext_ids as &[String],
            &p_names as &[String],
            &p_team_ids as &[i64],
        )
        .execute(pool)
        .await?;
    }

    // 4. Upsert matches; collect group memberships for bulk insert
    let mut memberships: HashSet<(i64, i64)> = HashSet::new();

    for m in api_matches {
        let (group_id, round) = match m.stage.as_str() {
            "GROUP_STAGE" => {
                let gid = m.group.as_ref().and_then(|g| group_id_map.get(g)).copied();
                (gid, None)
            }
            other => (None, api_stage_to_round(other)),
        };

        // Skip matches with stages we don't model (e.g. 3RD_PLACE play-off).
        if group_id.is_none() && round.is_none() {
            tracing::warn!(stage = %m.stage, match_id = m.id, "skipping match with unrecognised stage");
            continue;
        }

        let home_team_id = m.home_team.id.and_then(|id| team_id_map.get(&id)).copied();
        let away_team_id = m.away_team.id.and_then(|id| team_id_map.get(&id)).copied();

        if let Some(gid) = group_id {
            if let Some(htid) = home_team_id {
                memberships.insert((gid, htid));
            }
            if let Some(atid) = away_team_id {
                memberships.insert((gid, atid));
            }
        }

        upsert_match(
            pool,
            tournament_id,
            m,
            group_id,
            round,
            home_team_id,
            away_team_id,
        )
        .await?;
    }

    // Bulk upsert all collected group memberships — single query via UNNEST
    if !memberships.is_empty() {
        let (gids, tids): (Vec<i64>, Vec<i64>) = memberships.into_iter().unzip();
        sqlx::query!(
            r#"
            INSERT INTO group_memberships (group_id, team_id)
            SELECT unnest($1::bigint[]), unnest($2::bigint[])
            ON CONFLICT DO NOTHING
            "#,
            &gids as &[i64],
            &tids as &[i64],
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn api_stage_to_round(stage: &str) -> Option<KnockoutRound> {
    match stage {
        "ROUND_OF_32" | "LAST_32" => Some(KnockoutRound::R32),
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
        INSERT INTO teams (tournament_id, external_id, name, short_name, tla, flag)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (tournament_id, external_id) DO UPDATE
            SET name       = EXCLUDED.name,
                short_name = EXCLUDED.short_name,
                tla        = EXCLUDED.tla,
                flag       = EXCLUDED.flag
        RETURNING id
        "#,
        tournament_id,
        team.id.to_string(),
        team.name,
        team.short_name.as_deref().unwrap_or(&team.name),
        team.tla.as_deref(),
        tla_to_flag(team.tla.as_deref()),
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
    use crate::football_api::{
        Match as ApiMatch, MatchScore, MatchStatus, MatchTeam, ScoreDetail, Team as ApiTeam,
    };

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

    fn make_match(
        id: i64,
        stage: &str,
        group: Option<&str>,
        home_id: i64,
        away_id: i64,
    ) -> ApiMatch {
        ApiMatch {
            id,
            utc_date: "2024-06-14T21:00:00Z".to_string(),
            status: MatchStatus::Scheduled,
            stage: stage.to_string(),
            group: group.map(str::to_string),
            home_team: MatchTeam {
                id: Some(home_id),
                name: None,
            },
            away_team: MatchTeam {
                id: Some(away_id),
                name: None,
            },
            score: MatchScore {
                winner: None,
                full_time: ScoreDetail {
                    home: None,
                    away: None,
                },
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

        let updated_name: String =
            sqlx::query_scalar!("SELECT name FROM teams WHERE external_id = '1'")
                .fetch_one(&pool)
                .await
                .expect("fetch name");
        assert_eq!(
            updated_name, "Alpha FC",
            "re-seeding must update existing rows"
        );
    }
}

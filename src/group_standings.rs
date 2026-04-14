/// Pure-function group standings computation.
/// Implements cavekit-standings.md R8.
///
/// No DB calls, no async. Input is pre-fetched match results; output is ranked
/// standings per group. Tiebreaker order: Pts DESC → GD DESC → GF DESC →
/// H2H Pts DESC → H2H GD DESC → H2H GF DESC → team name alphabetical.

// ── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Home,
    Draw,
    Away,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupMatchResult {
    pub group_id: i64,
    pub home_team_id: i64,
    pub away_team_id: i64,
    /// None when match is not yet played (pending).
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
    /// None when match is not yet played.
    pub outcome: Option<Outcome>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TeamStanding {
    pub team_id: i64,
    pub team_name: String,
    pub mp: u32,
    pub w: u32,
    pub d: u32,
    pub l: u32,
    pub gf: i32,
    pub ga: i32,
    pub gd: i32,
    pub pts: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupStandings {
    pub group_id: i64,
    pub standings: Vec<TeamStanding>,
}

// ── Computation ────────────────────────────────────────────────────────────

/// Compute group standings from all finished and pending group matches.
/// Pending matches (outcome = None) are excluded from stats.
/// Returns one GroupStandings per group, sorted internally by rank.
pub fn compute_standings(matches: &[GroupMatchResult], team_names: &std::collections::HashMap<i64, String>) -> Vec<GroupStandings> {
    use std::collections::{BTreeMap, HashMap};

    // group_id → team_id → accumulator
    let mut groups: BTreeMap<i64, HashMap<i64, TeamStanding>> = BTreeMap::new();

    for m in matches {
        let group = groups.entry(m.group_id).or_default();

        // Ensure all participating teams are in the map even if pending
        for &team_id in &[m.home_team_id, m.away_team_id] {
            group.entry(team_id).or_insert_with(|| TeamStanding {
                team_id,
                team_name: team_names.get(&team_id).cloned().unwrap_or_default(),
                mp: 0,
                w: 0,
                d: 0,
                l: 0,
                gf: 0,
                ga: 0,
                gd: 0,
                pts: 0,
            });
        }

        // Only count finished matches
        let Some(outcome) = m.outcome else { continue };
        let (Some(hs), Some(as_)) = (m.home_score, m.away_score) else { continue };

        let (hw, hd, hl, hp, aw, ad, al, ap) = match outcome {
            Outcome::Home => (1, 0, 0, 3, 0, 0, 1, 0),
            Outcome::Draw => (0, 1, 0, 1, 0, 1, 0, 1),
            Outcome::Away => (0, 0, 1, 0, 1, 0, 0, 3),
        };

        let home = group.get_mut(&m.home_team_id).expect("just inserted");
        home.mp += 1;
        home.w += hw;
        home.d += hd;
        home.l += hl;
        home.gf += hs;
        home.ga += as_;
        home.gd = home.gf - home.ga;
        home.pts += hp;

        let away = group.get_mut(&m.away_team_id).expect("just inserted");
        away.mp += 1;
        away.w += aw;
        away.d += ad;
        away.l += al;
        away.gf += as_;
        away.ga += hs;
        away.gd = away.gf - away.ga;
        away.pts += ap;
    }

    groups
        .into_iter()
        .map(|(group_id, team_map)| {
            let mut rows: Vec<TeamStanding> = team_map.into_values().collect();
            sort_standings(&mut rows, matches, group_id);
            GroupStandings { group_id, standings: rows }
        })
        .collect()
}

/// Sort standings rows by: Pts DESC → GD DESC → GF DESC → H2H → alphabetical.
fn sort_standings(rows: &mut Vec<TeamStanding>, matches: &[GroupMatchResult], group_id: i64) {
    // Initial sort: Pts DESC, GD DESC, GF DESC, alphabetical
    rows.sort_by(|a, b| {
        b.pts
            .cmp(&a.pts)
            .then(b.gd.cmp(&a.gd))
            .then(b.gf.cmp(&a.gf))
            .then(a.team_name.cmp(&b.team_name))
    });

    // Apply H2H tiebreaker within tied sub-groups
    apply_h2h_tiebreaker(rows, matches, group_id);
}

/// Apply head-to-head tiebreaker within contiguous runs of teams with equal
/// Pts, GD, and GF. Only reorders teams within each tied sub-group.
fn apply_h2h_tiebreaker(
    rows: &mut Vec<TeamStanding>,
    matches: &[GroupMatchResult],
    group_id: i64,
) {
    let n = rows.len();
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n
            && rows[j].pts == rows[i].pts
            && rows[j].gd == rows[i].gd
            && rows[j].gf == rows[i].gf
        {
            j += 1;
        }
        if j - i > 1 {
            let tied_ids: Vec<i64> = rows[i..j].iter().map(|r| r.team_id).collect();
            sort_tied_by_h2h(&mut rows[i..j], matches, group_id, &tied_ids);
        }
        i = j;
    }
}

fn sort_tied_by_h2h(
    slice: &mut [TeamStanding],
    matches: &[GroupMatchResult],
    group_id: i64,
    tied_ids: &[i64],
) {
    use std::collections::HashMap;

    let mut h2h_pts: HashMap<i64, i32> = tied_ids.iter().map(|&id| (id, 0)).collect();
    let mut h2h_gf: HashMap<i64, i32> = tied_ids.iter().map(|&id| (id, 0)).collect();
    let mut h2h_ga: HashMap<i64, i32> = tied_ids.iter().map(|&id| (id, 0)).collect();

    for m in matches {
        if m.group_id != group_id { continue; }
        let Some(outcome) = m.outcome else { continue };
        let (Some(hs), Some(as_)) = (m.home_score, m.away_score) else { continue };

        let both = tied_ids.contains(&m.home_team_id) && tied_ids.contains(&m.away_team_id);
        if !both { continue; }

        match outcome {
            Outcome::Home => *h2h_pts.entry(m.home_team_id).or_default() += 3,
            Outcome::Draw => {
                *h2h_pts.entry(m.home_team_id).or_default() += 1;
                *h2h_pts.entry(m.away_team_id).or_default() += 1;
            }
            Outcome::Away => *h2h_pts.entry(m.away_team_id).or_default() += 3,
        }
        *h2h_gf.entry(m.home_team_id).or_default() += hs;
        *h2h_ga.entry(m.home_team_id).or_default() += as_;
        *h2h_gf.entry(m.away_team_id).or_default() += as_;
        *h2h_ga.entry(m.away_team_id).or_default() += hs;
    }

    slice.sort_by(|a, b| {
        let ap = h2h_pts.get(&a.team_id).copied().unwrap_or(0);
        let bp = h2h_pts.get(&b.team_id).copied().unwrap_or(0);
        let agf = h2h_gf.get(&a.team_id).copied().unwrap_or(0);
        let bgf = h2h_gf.get(&b.team_id).copied().unwrap_or(0);
        let aga = h2h_ga.get(&a.team_id).copied().unwrap_or(0);
        let bga = h2h_ga.get(&b.team_id).copied().unwrap_or(0);

        bp.cmp(&ap)                         // H2H pts DESC
            .then((bgf - bga).cmp(&(agf - aga))) // H2H GD DESC
            .then(bgf.cmp(&agf))            // H2H GF DESC
            .then(a.team_name.cmp(&b.team_name)) // alphabetical
    });
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn names(pairs: &[(i64, &str)]) -> HashMap<i64, String> {
        pairs.iter().map(|&(id, n)| (id, n.to_string())).collect()
    }

    fn m(group_id: i64, home: i64, away: i64, hs: i32, as_: i32) -> GroupMatchResult {
        let outcome = match hs.cmp(&as_) {
            std::cmp::Ordering::Greater => Some(Outcome::Home),
            std::cmp::Ordering::Equal   => Some(Outcome::Draw),
            std::cmp::Ordering::Less    => Some(Outcome::Away),
        };
        GroupMatchResult {
            group_id,
            home_team_id: home,
            away_team_id: away,
            home_score: Some(hs),
            away_score: Some(as_),
            outcome,
        }
    }

    fn pending(group_id: i64, home: i64, away: i64) -> GroupMatchResult {
        GroupMatchResult {
            group_id,
            home_team_id: home,
            away_team_id: away,
            home_score: None,
            away_score: None,
            outcome: None,
        }
    }

    fn standing<'a>(groups: &'a [GroupStandings], group_id: i64, team_id: i64) -> &'a TeamStanding {
        groups
            .iter()
            .find(|g| g.group_id == group_id)
            .and_then(|g| g.standings.iter().find(|r| r.team_id == team_id))
            .expect("standing not found")
    }

    #[test]
    fn empty_input_returns_empty() {
        assert!(compute_standings(&[], &HashMap::new()).is_empty());
    }

    #[test]
    fn pending_match_registers_teams_but_no_stats() {
        let ns = names(&[(1, "Alpha"), (2, "Beta")]);
        let groups = compute_standings(&[pending(1, 1, 2)], &ns);
        assert_eq!(groups.len(), 1);
        let g = &groups[0];
        assert_eq!(g.standings.len(), 2);
        let t1 = g.standings.iter().find(|r| r.team_id == 1).unwrap();
        assert_eq!(t1.mp, 0);
        assert_eq!(t1.pts, 0);
    }

    #[test]
    fn win_gives_three_points_loss_zero() {
        let ns = names(&[(1, "Alpha"), (2, "Beta")]);
        let groups = compute_standings(&[m(1, 1, 2, 2, 0)], &ns);
        let winner = standing(&groups, 1, 1);
        let loser = standing(&groups, 1, 2);
        assert_eq!(winner.pts, 3);
        assert_eq!(winner.w, 1);
        assert_eq!(loser.pts, 0);
        assert_eq!(loser.l, 1);
    }

    #[test]
    fn draw_gives_one_point_each() {
        let ns = names(&[(1, "Alpha"), (2, "Beta")]);
        let groups = compute_standings(&[m(1, 1, 2, 1, 1)], &ns);
        for row in &groups[0].standings {
            assert_eq!(row.pts, 1);
            assert_eq!(row.d, 1);
        }
    }

    #[test]
    fn goal_difference_computed_correctly() {
        let ns = names(&[(1, "Alpha"), (2, "Beta")]);
        let groups = compute_standings(&[m(1, 1, 2, 3, 1)], &ns);
        let t1 = standing(&groups, 1, 1);
        let t2 = standing(&groups, 1, 2);
        assert_eq!(t1.gd, 2);
        assert_eq!(t2.gd, -2);
    }

    #[test]
    fn multiple_groups_are_independent() {
        let ns = names(&[(1, "Alpha"), (2, "Beta"), (3, "Gamma"), (4, "Delta")]);
        let groups = compute_standings(&[m(1, 1, 2, 1, 0), m(2, 3, 4, 0, 2)], &ns);
        assert_eq!(groups.len(), 2);
        let winner_a = standing(&groups, 1, 1);
        assert_eq!(winner_a.pts, 3);
        let winner_b = standing(&groups, 2, 4);
        assert_eq!(winner_b.pts, 3);
    }

    #[test]
    fn gd_tiebreaker() {
        // T1: W-D-L scenario; T2 same pts but better GD → T2 should rank higher
        // T1: beat T3 1-0 (+1 GD), draw T4 0-0 → 4 pts, GD +1
        // T2: beat T3 3-0 (+3 GD), draw T4 0-0 → 4 pts, GD +3
        let ns = names(&[(1, "T1"), (2, "T2"), (3, "T3"), (4, "T4")]);
        let groups = compute_standings(&[
            m(1, 1, 3, 1, 0),
            m(1, 2, 3, 3, 0),
            m(1, 1, 4, 0, 0),
            m(1, 2, 4, 0, 0),
            m(1, 3, 4, 0, 0), // T3 vs T4 for completeness
        ], &ns);
        let t1 = standing(&groups, 1, 1);
        let t2 = standing(&groups, 1, 2);
        assert!(t2.pts == t1.pts, "same pts");
        assert!(t2.gd > t1.gd, "T2 better GD");
        // T2 must rank above T1
        let pos_t2 = groups[0].standings.iter().position(|r| r.team_id == 2).unwrap();
        let pos_t1 = groups[0].standings.iter().position(|r| r.team_id == 1).unwrap();
        assert!(pos_t2 < pos_t1, "T2 ranked above T1 via GD tiebreaker");
    }

    #[test]
    fn gf_tiebreaker() {
        // T1 and T2 tied on pts and GD but T2 has more GF
        // T1: beat T3 1-0, beat T4 1-0 → 6 pts, GD +2, GF 2
        // T2: beat T3 2-1, beat T4 2-1 → 6 pts, GD +2, GF 4
        let ns = names(&[(1, "T1"), (2, "T2"), (3, "T3"), (4, "T4")]);
        let groups = compute_standings(&[
            m(1, 1, 3, 1, 0),
            m(1, 1, 4, 1, 0),
            m(1, 2, 3, 2, 1),
            m(1, 2, 4, 2, 1),
            m(1, 3, 4, 0, 0),
        ], &ns);
        let t1 = standing(&groups, 1, 1);
        let t2 = standing(&groups, 1, 2);
        assert_eq!(t1.pts, t2.pts, "same pts");
        assert_eq!(t1.gd, t2.gd, "same GD");
        assert!(t2.gf > t1.gf, "T2 more GF");
        let pos_t2 = groups[0].standings.iter().position(|r| r.team_id == 2).unwrap();
        let pos_t1 = groups[0].standings.iter().position(|r| r.team_id == 1).unwrap();
        assert!(pos_t2 < pos_t1, "T2 ranked above T1 via GF tiebreaker");
    }

    #[test]
    fn h2h_tiebreaker() {
        // T1 and T2: same pts/GD/GF overall, but T1 beat T2 head-to-head
        // T1: beat T2 1-0, draw T3 1-1 → 4 pts
        // T2: lost to T1 0-1, beat T3 2-0 → 3 pts... different pts, let's make it tighter
        // Actually let's do: T1 and T2 both 4 pts, same GD, same GF, T1 beat T2 H2H
        // T1: beat T2 1-0 (+3 pts), draw T3 1-1 (+1 pt) → 4 pts, GD: 1+0=+1, GF: 1+1=2
        // T2: lost to T1 0-1 (0 pts), beat T3 2-0 (+3 pts), draw T4 1-1 (+1 pt) → 4 pts, GD: -1+2+0=+1, GF: 0+2+1=3
        // Not matching. Let's do simpler:
        // T1: beat T3 2-0, T1 draws T4 → some config where T1 and T2 are tied overall but T1 wins H2H
        // Simplest: T1 beat T2, T2 beat T3, T1 draw T3 → T1: 4 pts, T2: 3 pts → not tied
        // For a clean H2H test: 3-team group, all teams get same points via circular results
        // T1 beat T2 1-0 (3 pts), T2 beat T3 1-0 (3 pts), T3 beat T1 1-0 (3 pts) → all 3 pts, all GD=0, all GF=1
        let ns = names(&[(1, "T1"), (2, "T2"), (3, "T3")]);
        let groups = compute_standings(&[
            m(1, 1, 2, 1, 0), // T1 beat T2
            m(1, 2, 3, 1, 0), // T2 beat T3
            m(1, 3, 1, 1, 0), // T3 beat T1
        ], &ns);
        // All tied, H2H results: T1 beat T2, T2 beat T3, T3 beat T1
        // H2H pts among 3: T1=3 (beat T2), T2=3 (beat T3), T3=3 (beat T1) → still all tied
        // After H2H, alphabetical: T1, T2, T3
        let positions: Vec<i64> = groups[0].standings.iter().map(|r| r.team_id).collect();
        // All should be in alphabetical order (T1 < T2 < T3) as final tiebreaker
        assert_eq!(positions, vec![1, 2, 3], "alphabetical fallback after H2H tie: {:?}", positions);
    }

    #[test]
    fn alphabetical_fallback() {
        // Two teams with identical records: draw 1-1 → both 1 pt, GD 0, GF 1
        // Alphabetical: team "Alpha" before "Zeta"
        let ns = names(&[(1, "Zeta"), (2, "Alpha")]);
        let groups = compute_standings(&[m(1, 1, 2, 1, 1)], &ns);
        let positions: Vec<i64> = groups[0].standings.iter().map(|r| r.team_id).collect();
        // "Alpha" (id=2) should come before "Zeta" (id=1)
        assert_eq!(positions, vec![2, 1], "Alpha before Zeta alphabetically");
    }

    #[test]
    fn partial_group_only_counts_played() {
        // 4 teams, only 2 matches played of 6 total
        let ns = names(&[(1, "T1"), (2, "T2"), (3, "T3"), (4, "T4")]);
        let groups = compute_standings(&[
            m(1, 1, 2, 2, 0),       // played
            pending(1, 3, 4),        // pending
            pending(1, 1, 3),
            pending(1, 2, 4),
        ], &ns);
        let t1 = standing(&groups, 1, 1);
        assert_eq!(t1.mp, 1);
        assert_eq!(t1.pts, 3);
        // T3 and T4 have no played matches
        let t3 = standing(&groups, 1, 3);
        assert_eq!(t3.mp, 0);
    }
}

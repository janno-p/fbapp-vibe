use std::collections::{HashMap, HashSet};

use crate::{
    achievements::BadgeDisplay,
    db_types::{KnockoutRound, MatchOutcome},
    polling::scorer::knockout_points_per_team,
};

// ── Leaderboard ───────────────────────────────────────────────────────────────

/// Raw row returned directly from the leaderboard SQL query.
pub struct LeaderboardRawRow {
    pub user_id: i64,
    pub user_name: String,
    pub total_points: i64,
    pub max_achievable: i64,
}

/// Leaderboard entry with computed rank and gap.
pub struct LeaderboardEntry {
    pub rank: usize,
    pub user_id: i64,
    pub user_name: String,
    pub total_points: i64,
    pub max_achievable: i64,
    pub points_behind: i64,
    /// Remaining points this player could still earn (`max_achievable - total_points`).
    /// Display-only value; does NOT drive band assignment.
    pub remaining_possible: i64,
    /// 7-tier ceiling band (1 = lowest, 7 = highest) derived from `max_achievable`.
    /// 1 and 7 get triple chevrons, 2/6 double, 3/5 single, 4 is neutral.
    pub ceiling_band: u8,
    /// Most recently awarded badge for this user, if any.
    pub top_badge: Option<BadgeDisplay>,
    /// Projected additional points from hypothetical outcomes (0 = no scenario active).
    pub projected_delta: i64,
}

impl LeaderboardEntry {
    /// True when a scenario is active and this user has projected additional points.
    pub fn has_projection(&self) -> bool {
        self.projected_delta > 0
    }
}

/// Assigns ranks (1-based) and computes `points_behind` relative to the leader.
///
/// Input must already be sorted by:
///   1. `total_points DESC`
///   2. `max_achievable DESC` (tie-break: higher ceiling first)
///   3. `user_name ASC` (final deterministic tie-break)
///
/// `badges` maps user_id → their most recent badge (from `get_top_badge_per_user`).
/// `deltas` maps user_id → projected extra points from hypothetical outcomes.
pub fn build_leaderboard(
    rows: Vec<LeaderboardRawRow>,
    mut badges: HashMap<i64, BadgeDisplay>,
    deltas: HashMap<i64, i64>,
) -> Vec<LeaderboardEntry> {
    let leader = rows.first().map(|r| r.total_points).unwrap_or(0);
    let mut entries: Vec<LeaderboardEntry> = rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| LeaderboardEntry {
            rank: i + 1,
            points_behind: leader - r.total_points,
            remaining_possible: r.max_achievable - r.total_points,
            ceiling_band: 4, // placeholder; assign_ceiling_bands overwrites below
            top_badge: badges.remove(&r.user_id),
            projected_delta: deltas.get(&r.user_id).copied().unwrap_or(0),
            user_id: r.user_id,
            user_name: r.user_name,
            total_points: r.total_points,
            max_achievable: r.max_achievable,
        })
        .collect();
    assign_ceiling_bands(&mut entries);
    entries
}

/// Assigns a 7-tier `ceiling_band` to each entry based on `max_achievable`.
///
/// Band 7 = highest ceiling, Band 1 = lowest. If all entries share the same
/// `max_achievable` (range = 0), all receive band 4 (middle/neutral).
///
/// Formula: `band = clamp(floor((value - min) * 7 / range), 0, 6) + 1`
pub fn assign_ceiling_bands(entries: &mut [LeaderboardEntry]) {
    if entries.is_empty() {
        return;
    }
    let min_ceiling = entries.iter().map(|e| e.max_achievable).min().unwrap_or(0);
    let max_ceiling = entries.iter().map(|e| e.max_achievable).max().unwrap_or(0);
    let range = max_ceiling - min_ceiling;
    for entry in entries.iter_mut() {
        entry.ceiling_band = if range == 0 {
            4
        } else {
            let band_raw = ((entry.max_achievable - min_ceiling) * 7) / range;
            (band_raw as u8).clamp(0, 6) + 1
        };
    }
}

impl LeaderboardEntry {
    pub fn is_leader(&self) -> bool {
        self.points_behind == 0
    }
}

// ── Scenario modeling ─────────────────────────────────────────────────────────

/// Maximum number of hypothetical match outcomes accepted per request.
pub const MAX_HYPO_MATCHES: usize = 20;

/// Parses raw query params (`hypo[{match_id}]=home|draw|away`) into a map.
/// Non-matching keys, non-integer IDs, and invalid outcome values are silently ignored.
/// At most MAX_HYPO_MATCHES entries are returned; excess entries are dropped.
pub fn parse_hypo_params(params: &HashMap<String, String>) -> HashMap<i64, MatchOutcome> {
    params
        .iter()
        .filter_map(|(k, v)| {
            let id_str = k.strip_prefix("hypo[")?.strip_suffix(']')?;
            let match_id: i64 = id_str.parse().ok()?;
            let outcome = MatchOutcome::from_slug(v)?;
            Some((match_id, outcome))
        })
        .take(MAX_HYPO_MATCHES)
        .collect()
}

/// Filters a parsed hypo map to only include match IDs in the given whitelist.
/// Called by the handler with the set of valid unplayed group-stage match IDs,
/// ensuring knockout or nonexistent match IDs are rejected.
pub fn filter_hypo_by_whitelist(
    hypo: HashMap<i64, MatchOutcome>,
    whitelist: &HashSet<i64>,
) -> HashMap<i64, MatchOutcome> {
    hypo.into_iter()
        .filter(|(id, _)| whitelist.contains(id))
        .collect()
}

/// Computes projected point deltas for each user given hypothetical outcomes.
///
/// For each hypothetical match outcome:
///   - +2 if user predicted correctly AND is_confident
///   - +1 if user predicted correctly AND NOT is_confident
///   - +0 otherwise
///
/// Returns `HashMap<user_id, projected_delta>` — only users with non-zero deltas are included.
pub fn compute_projected_delta(
    predictions: &[(i64, super::db::HypoPrediction)], // (match_id, prediction)
    hypo_outcomes: &HashMap<i64, MatchOutcome>,
) -> HashMap<i64, i64> {
    let mut deltas: HashMap<i64, i64> = HashMap::new();

    for (match_id, pred) in predictions {
        if let Some(hypo) = hypo_outcomes.get(match_id)
            && &pred.predicted_outcome == hypo
        {
            let pts = if pred.is_confident { 2 } else { 1 };
            *deltas.entry(pred.user_id).or_insert(0) += pts;
        }
    }

    deltas
}

// ── Match breakdown ───────────────────────────────────────────────────────────

pub struct MatchInfo {
    pub id: i64,
    pub home_name: String,
    pub away_name: String,
    pub home_flag: String,
    pub away_flag: String,
    pub scheduled_at: time::OffsetDateTime,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
    pub outcome: Option<MatchOutcome>,
}

impl MatchInfo {
    pub fn is_played(&self) -> bool {
        self.outcome.is_some()
    }

    pub fn result_label(&self) -> String {
        match (self.home_score, self.away_score) {
            (Some(h), Some(a)) => format!("{h} – {a}"),
            _ => "—".to_string(),
        }
    }

    pub fn outcome_label(&self) -> &str {
        match &self.outcome {
            Some(MatchOutcome::Home) => "Home win",
            Some(MatchOutcome::Draw) => "Draw",
            Some(MatchOutcome::Away) => "Away win",
            None => "—",
        }
    }

    pub fn formatted_kickoff(&self) -> String {
        let fmt = time::format_description::parse("[day] [month repr:short] [hour]:[minute] UTC")
            .expect("static format is valid");
        self.scheduled_at
            .format(&fmt)
            .unwrap_or_else(|_| "TBD".to_string())
    }

    pub fn scheduled_at_epoch_ms(&self) -> i64 {
        self.scheduled_at.unix_timestamp() * 1000
    }
}

/// One league member's prediction row for a group stage match breakdown.
pub struct MatchBreakdownRow {
    pub user_id: i64,
    pub user_name: String,
    pub predicted_outcome: Option<MatchOutcome>,
    pub is_confident: bool,
    pub points_awarded: Option<i32>,
}

impl MatchBreakdownRow {
    pub fn prediction_label(&self) -> &str {
        outcome_label(&self.predicted_outcome)
    }

    /// True when points were awarded and they are greater than zero.
    pub fn points_positive(&self) -> bool {
        self.points_awarded.map(|p| p > 0).unwrap_or(false)
    }
}

fn outcome_label(o: &Option<MatchOutcome>) -> &'static str {
    match o {
        Some(MatchOutcome::Home) => "Home",
        Some(MatchOutcome::Draw) => "Draw",
        Some(MatchOutcome::Away) => "Away",
        None => "—",
    }
}

// ── Nearest match ─────────────────────────────────────────────────────────────

pub struct NearestMatch {
    pub id: i64,
    pub home_name: String,
    pub away_name: String,
    pub home_flag: String,
    pub away_flag: String,
    pub scheduled_at: time::OffsetDateTime,
    pub outcome: Option<MatchOutcome>,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
}

impl NearestMatch {
    pub fn is_played(&self) -> bool {
        self.outcome.is_some()
    }

    pub fn score_label(&self) -> String {
        match (self.home_score, self.away_score) {
            (Some(h), Some(a)) => format!("{h} – {a}"),
            _ => "— vs —".to_string(),
        }
    }
}

// ── Comparison ────────────────────────────────────────────────────────────────

/// League member for the comparison picker.
pub struct LeagueMember {
    pub id: i64,
    pub name: String,
}

/// One group stage match row in the comparison view.
pub struct CompareGroupRow {
    pub home_name: String,
    pub away_name: String,
    pub scheduled_at: time::OffsetDateTime,
    pub actual_outcome: Option<MatchOutcome>,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
    pub a_prediction: Option<MatchOutcome>,
    pub a_points: Option<i32>,
    pub b_prediction: Option<MatchOutcome>,
    pub b_points: Option<i32>,
}

impl CompareGroupRow {
    pub fn a_label(&self) -> &str {
        outcome_label(&self.a_prediction)
    }

    pub fn b_label(&self) -> &str {
        outcome_label(&self.b_prediction)
    }

    pub fn actual_label(&self) -> &str {
        outcome_label(&self.actual_outcome)
    }

    pub fn score_label(&self) -> String {
        match (self.home_score, self.away_score) {
            (Some(h), Some(a)) => format!("{h}–{a}"),
            _ => String::new(),
        }
    }

    /// Returns `Some(true)` if A predicted correctly, `Some(false)` if wrong, `None` if unplayed.
    pub fn a_correct(&self) -> Option<bool> {
        match (&self.a_prediction, &self.actual_outcome) {
            (Some(pred), Some(actual)) => Some(pred == actual),
            _ => None,
        }
    }

    /// Returns `Some(true)` if B predicted correctly, `Some(false)` if wrong, `None` if unplayed.
    pub fn b_correct(&self) -> Option<bool> {
        match (&self.b_prediction, &self.actual_outcome) {
            (Some(pred), Some(actual)) => Some(pred == actual),
            _ => None,
        }
    }
}

// ── Fixtures ──────────────────────────────────────────────────────────────────

pub struct FixtureRow {
    pub id: i64,
    pub home_name: String,
    pub away_name: String,
    pub home_flag: String,
    pub away_flag: String,
    pub scheduled_at: time::OffsetDateTime,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
    pub outcome: Option<MatchOutcome>,
    pub group_name: Option<String>,
    pub round: Option<KnockoutRound>,
}

impl FixtureRow {
    pub fn is_played(&self) -> bool {
        self.outcome.is_some()
    }

    pub fn result_label(&self) -> String {
        match (self.home_score, self.away_score) {
            (Some(h), Some(a)) => format!("{h} – {a}"),
            _ => "TBD".to_string(),
        }
    }

    pub fn formatted_kickoff(&self) -> String {
        let fmt = time::format_description::parse("[day] [month repr:short] [hour]:[minute] UTC")
            .expect("static format is valid");
        self.scheduled_at
            .format(&fmt)
            .unwrap_or_else(|_| "TBD".to_string())
    }

    pub fn scheduled_at_epoch_ms(&self) -> i64 {
        self.scheduled_at.unix_timestamp() * 1000
    }
}

pub struct FixtureGroup {
    pub label: String,
    pub matches: Vec<FixtureRow>,
}

/// Groups a pre-sorted flat list of fixture rows into labelled sections.
///
/// Input must already be sorted: group stage first (by group name ASC),
/// then knockout stage in round order. Consecutive rows with the same
/// stage label are collapsed into one group.
pub fn group_fixtures(rows: Vec<FixtureRow>) -> Vec<FixtureGroup> {
    let mut groups: Vec<FixtureGroup> = Vec::new();
    for row in rows {
        let label = if let Some(ref g) = row.group_name {
            format!("Group {g}")
        } else if let Some(ref r) = row.round {
            r.label().to_string()
        } else {
            "Other".to_string()
        };
        if let Some(g) = groups.last_mut()
            && g.label == label
        {
            g.matches.push(row);
        } else {
            groups.push(FixtureGroup {
                label,
                matches: vec![row],
            });
        }
    }
    groups
}

// ── Consensus ─────────────────────────────────────────────────────────────────

/// Distribution of league member predictions for a single group stage match.
///
/// The percentage denominator is `total_predictors()` (excludes members who
/// did not submit a prediction). Division by zero is guarded — all percentage
/// methods return 0 when no one has predicted.
pub struct MatchConsensus {
    pub home_count: i64,
    pub draw_count: i64,
    pub away_count: i64,
    /// Members who are in the league but have no prediction for this match.
    pub no_prediction_count: i64,
}

impl MatchConsensus {
    pub fn total_predictors(&self) -> i64 {
        self.home_count + self.draw_count + self.away_count
    }

    pub fn home_percentage(&self) -> u32 {
        percentage(self.home_count, self.total_predictors())
    }

    pub fn draw_percentage(&self) -> u32 {
        percentage(self.draw_count, self.total_predictors())
    }

    pub fn away_percentage(&self) -> u32 {
        percentage(self.away_count, self.total_predictors())
    }
}

fn percentage(count: i64, total: i64) -> u32 {
    if total == 0 {
        return 0;
    }
    ((count as f64 / total as f64) * 100.0).round() as u32
}

// ── Group standings view ──────────────────────────────────────────────────────

/// One group's standings for display, with the group name resolved.
pub struct GroupStandingsView {
    pub group_name: String,
    pub standings: Vec<crate::group_standings::TeamStanding>,
}

// ── Future prospects (pure) ───────────────────────────────────────────────────

/// Returns the maximum points a user can still achieve.
///
/// - `earned`: already-awarded `points_awarded` across all prediction tables
/// - `group_unplayed`: count of group predictions where the match has no outcome yet
/// - `knockout_pending`: `(round, count)` pairs — unscored knockout round predictions
/// - `top_scorer_pending_goals`: best current goal tally among unscored top-scorer picks;
///   `None` when all top-scorer picks are already finalised
pub fn max_achievable_points(
    earned: i32,
    group_unplayed: i32,
    knockout_pending: &[(KnockoutRound, i32)],
    top_scorer_pending_goals: Option<i32>,
) -> i32 {
    let knockout_possible: i32 = knockout_pending
        .iter()
        .map(|(round, count)| knockout_points_per_team(round) * count)
        .sum();
    let top_scorer_possible = top_scorer_pending_goals.map(|g| 5 + g).unwrap_or(0);
    earned + group_unplayed + knockout_possible + top_scorer_possible
}

// ── Member stats ─────────────────────────────────────────────────────────────

/// Raw row for a member's group stage prediction on a played match.
pub struct MemberGroupPredRow {
    pub predicted_outcome: Option<MatchOutcome>,
    pub actual_outcome: MatchOutcome,
}

impl MemberGroupPredRow {
    pub fn is_correct(&self) -> bool {
        self.predicted_outcome.as_ref() == Some(&self.actual_outcome)
    }
}

/// Aggregated per-user prediction stats for the member stats page.
pub struct MemberStats {
    pub user_id: i64,
    pub user_name: String,
    pub league_joined_at: time::OffsetDateTime,
    pub total_points: i64,
    pub rank: usize,
    pub group_correct: i64,
    pub group_total: i64,
    pub knockout_correct: i64,
    pub knockout_total: i64,
    pub top_scorer_points: i64,
    pub current_streak: usize,
    pub best_streak: usize,
}

impl MemberStats {
    pub fn group_accuracy_pct(&self) -> u32 {
        if self.group_total == 0 {
            return 0;
        }
        ((self.group_correct as f64 / self.group_total as f64) * 100.0).round() as u32
    }

    pub fn formatted_join_date(&self) -> String {
        let fmt = time::format_description::parse("[day] [month repr:short] [year]")
            .expect("static format is valid");
        self.league_joined_at
            .format(&fmt)
            .unwrap_or_else(|_| "—".to_string())
    }
}

/// Computes `(current_streak, best_streak)` from an ordered slice of booleans.
///
/// `predictions` must be ordered chronologically (oldest first). `true` means
/// the prediction was correct. An empty slice returns `(0, 0)`.
pub fn compute_streaks(predictions: &[bool]) -> (usize, usize) {
    let mut current = 0usize;
    let mut best = 0usize;
    for &correct in predictions {
        if correct {
            current += 1;
            if current > best {
                best = current;
            }
        } else {
            current = 0;
        }
    }
    (current, best)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── MatchConsensus ────────────────────────────────────────────────────────

    fn consensus(h: i64, d: i64, a: i64, n: i64) -> MatchConsensus {
        MatchConsensus {
            home_count: h,
            draw_count: d,
            away_count: a,
            no_prediction_count: n,
        }
    }

    #[test]
    fn consensus_returns_zero_when_no_predictors() {
        let c = consensus(0, 0, 0, 3);
        assert_eq!(c.total_predictors(), 0);
        assert_eq!(c.home_percentage(), 0);
        assert_eq!(c.draw_percentage(), 0);
        assert_eq!(c.away_percentage(), 0);
    }

    #[test]
    fn consensus_even_split_three_outcomes() {
        let c = consensus(1, 1, 1, 0);
        assert_eq!(c.total_predictors(), 3);
        assert_eq!(c.home_percentage(), 33);
        assert_eq!(c.draw_percentage(), 33);
        assert_eq!(c.away_percentage(), 33);
    }

    #[test]
    fn consensus_majority_home_rounds_correctly() {
        // 2/7, 2/7, 3/7 → 29%, 29%, 43%
        let c = consensus(2, 2, 3, 0);
        assert_eq!(c.home_percentage(), 29);
        assert_eq!(c.draw_percentage(), 29);
        assert_eq!(c.away_percentage(), 43);
    }

    #[test]
    fn consensus_all_home_gives_100_percent() {
        let c = consensus(5, 0, 0, 2);
        assert_eq!(c.home_percentage(), 100);
        assert_eq!(c.draw_percentage(), 0);
        assert_eq!(c.away_percentage(), 0);
    }

    #[test]
    fn consensus_no_prediction_count_does_not_affect_percentages() {
        // 3 predictors, 2 non-predictors — denominator is 3
        let c = consensus(3, 0, 0, 2);
        assert_eq!(c.total_predictors(), 3);
        assert_eq!(c.home_percentage(), 100);
    }

    // ── max_achievable_points ─────────────────────────────────────────────────

    #[test]
    fn no_pending_returns_earned() {
        assert_eq!(max_achievable_points(30, 0, &[], None), 30);
    }

    #[test]
    fn all_unplayed_group_only() {
        assert_eq!(max_achievable_points(0, 48, &[], None), 48);
    }

    #[test]
    fn mixed_earned_and_group_pending() {
        assert_eq!(max_achievable_points(10, 5, &[], None), 15);
    }

    #[test]
    fn knockout_pending_uses_correct_points() {
        let pending = [(KnockoutRound::R16, 8)]; // 8 × 3 = 24
        assert_eq!(max_achievable_points(0, 0, &pending, None), 24);
    }

    #[test]
    fn top_scorer_pending_adds_five_plus_goals() {
        assert_eq!(max_achievable_points(0, 0, &[], Some(7)), 12);
    }

    #[test]
    fn full_tournament_pending() {
        let pending = [
            (KnockoutRound::R32, 32),   // 64
            (KnockoutRound::R16, 16),   // 48
            (KnockoutRound::Qf, 8),     // 32
            (KnockoutRound::Sf, 4),     // 24
            (KnockoutRound::Final, 2),  // 16
            (KnockoutRound::Winner, 1), // 10
        ];
        // knockout = 194, group = 48, top scorer = 5+5 = 10, total = 252
        assert_eq!(max_achievable_points(0, 48, &pending, Some(5)), 252);
    }

    #[test]
    fn short_bracket_pending_excludes_r32() {
        // EURO 2024 style: starts at R16, no R32 round
        let pending = [
            (KnockoutRound::R16, 16),   // 48
            (KnockoutRound::Qf, 8),     // 32
            (KnockoutRound::Sf, 4),     // 24
            (KnockoutRound::Final, 2),  // 16
            (KnockoutRound::Winner, 1), // 10
        ];
        // knockout = 130, group = 24, top scorer = 0, total = 154
        assert_eq!(max_achievable_points(0, 24, &pending, None), 154);
    }

    // ── build_leaderboard ─────────────────────────────────────────────────────

    #[test]
    fn build_leaderboard_assigns_ranks_and_gap() {
        let rows = vec![
            LeaderboardRawRow {
                user_id: 1,
                user_name: "Alice".to_string(),
                total_points: 20,
                max_achievable: 30,
            },
            LeaderboardRawRow {
                user_id: 2,
                user_name: "Bob".to_string(),
                total_points: 15,
                max_achievable: 25,
            },
        ];
        let entries = build_leaderboard(rows, HashMap::new(), HashMap::new());
        assert_eq!(entries[0].rank, 1);
        assert_eq!(entries[0].points_behind, 0);
        assert_eq!(entries[1].rank, 2);
        assert_eq!(entries[1].points_behind, 5);
    }

    #[test]
    fn build_leaderboard_empty_vec() {
        assert!(build_leaderboard(vec![], HashMap::new(), HashMap::new()).is_empty());
    }

    #[test]
    fn tiebreak_higher_max_achievable_ranks_first() {
        let rows = vec![
            LeaderboardRawRow {
                user_id: 1,
                user_name: "Alice".to_string(),
                total_points: 10,
                max_achievable: 20,
            },
            LeaderboardRawRow {
                user_id: 2,
                user_name: "Bob".to_string(),
                total_points: 10,
                max_achievable: 15,
            },
        ];
        let entries = build_leaderboard(rows, HashMap::new(), HashMap::new());
        assert_eq!(
            entries[0].user_id, 1,
            "Alice has higher ceiling, should rank first"
        );
        assert_eq!(entries[1].user_id, 2);
    }

    // ── compute_streaks ───────────────────────────────────────────────────────

    #[test]
    fn streaks_empty_slice_returns_zero() {
        assert_eq!(compute_streaks(&[]), (0, 0));
    }

    #[test]
    fn streaks_all_correct() {
        assert_eq!(compute_streaks(&[true, true, true]), (3, 3));
    }

    #[test]
    fn streaks_all_wrong() {
        assert_eq!(compute_streaks(&[false, false, false]), (0, 0));
    }

    #[test]
    fn streaks_alternating_ends_on_wrong() {
        // T F T F → current=0, best=1
        assert_eq!(compute_streaks(&[true, false, true, false]), (0, 1));
    }

    #[test]
    fn streaks_trailing_correct_streak() {
        // F T T T → current=3, best=3
        assert_eq!(compute_streaks(&[false, true, true, true]), (3, 3));
    }

    #[test]
    fn streaks_best_is_longer_than_current() {
        // T T T F T T → current=2, best=3
        assert_eq!(
            compute_streaks(&[true, true, true, false, true, true]),
            (2, 3)
        );
    }

    // ── parse_hypo_params ─────────────────────────────────────────────────────

    fn hypo_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn parse_hypo_valid_subset_accepted() {
        let params = hypo_map(&[("hypo[1]", "home"), ("hypo[2]", "draw"), ("other", "x")]);
        let result = parse_hypo_params(&params);
        assert_eq!(result.get(&1), Some(&MatchOutcome::Home));
        assert_eq!(result.get(&2), Some(&MatchOutcome::Draw));
        assert!(!result.contains_key(&0), "non-hypo key ignored");
    }

    #[test]
    fn parse_hypo_invalid_id_filtered_out() {
        let params = hypo_map(&[("hypo[abc]", "home"), ("hypo[1]", "away")]);
        let result = parse_hypo_params(&params);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&1), Some(&MatchOutcome::Away));
    }

    #[test]
    fn parse_hypo_invalid_value_ignored() {
        let params = hypo_map(&[("hypo[1]", "win"), ("hypo[2]", "home")]);
        let result = parse_hypo_params(&params);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&2), Some(&MatchOutcome::Home));
    }

    #[test]
    fn parse_hypo_all_valid_outcomes_accepted() {
        let params = hypo_map(&[
            ("hypo[1]", "home"),
            ("hypo[2]", "draw"),
            ("hypo[3]", "away"),
        ]);
        let result = parse_hypo_params(&params);
        assert_eq!(result.get(&1), Some(&MatchOutcome::Home));
        assert_eq!(result.get(&2), Some(&MatchOutcome::Draw));
        assert_eq!(result.get(&3), Some(&MatchOutcome::Away));
    }

    #[test]
    fn parse_hypo_truncates_at_max() {
        // Build 25 valid hypo params
        let pairs: Vec<(String, String)> = (1i64..=25)
            .map(|i| (format!("hypo[{i}]"), "home".to_string()))
            .collect();
        let params: HashMap<String, String> = pairs.into_iter().collect();
        let result = parse_hypo_params(&params);
        assert!(
            result.len() <= MAX_HYPO_MATCHES,
            "must not exceed {MAX_HYPO_MATCHES}"
        );
    }

    // ── filter_hypo_by_whitelist ──────────────────────────────────────────────

    #[test]
    fn filter_hypo_valid_id_passes_through() {
        let whitelist: HashSet<i64> = [1, 2, 3].into_iter().collect();
        let hypo = [(1i64, MatchOutcome::Home), (2, MatchOutcome::Draw)]
            .into_iter()
            .collect();
        let result = filter_hypo_by_whitelist(hypo, &whitelist);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_hypo_knockout_id_rejected() {
        // Whitelist contains only group-stage IDs; knockout ID 99 is absent
        let whitelist: HashSet<i64> = [1, 2, 3].into_iter().collect();
        let hypo = [(99i64, MatchOutcome::Home), (1, MatchOutcome::Away)]
            .into_iter()
            .collect();
        let result = filter_hypo_by_whitelist(hypo, &whitelist);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&1), Some(&MatchOutcome::Away));
        assert!(!result.contains_key(&99));
    }

    #[test]
    fn filter_hypo_nonexistent_id_rejected() {
        let whitelist: HashSet<i64> = [5].into_iter().collect();
        let hypo = [(999i64, MatchOutcome::Draw)].into_iter().collect();
        let result = filter_hypo_by_whitelist(hypo, &whitelist);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_hypo_empty_whitelist_drops_all() {
        let whitelist: HashSet<i64> = HashSet::new();
        let hypo = [(1i64, MatchOutcome::Home), (2, MatchOutcome::Away)]
            .into_iter()
            .collect();
        let result = filter_hypo_by_whitelist(hypo, &whitelist);
        assert!(result.is_empty());
    }

    // ── assign_ceiling_bands ──────────────────────────────────────────────────

    fn entry_with_ceiling(max_achievable: i64) -> LeaderboardEntry {
        LeaderboardEntry {
            rank: 1,
            user_id: 1,
            user_name: "Test".to_string(),
            total_points: 0,
            max_achievable,
            points_behind: 0,
            remaining_possible: max_achievable,
            ceiling_band: 0,
            top_badge: None,
            projected_delta: 0,
        }
    }

    #[test]
    fn ceiling_band_all_equal_returns_4() {
        let mut entries = vec![
            entry_with_ceiling(100),
            entry_with_ceiling(100),
            entry_with_ceiling(100),
        ];
        assign_ceiling_bands(&mut entries);
        for e in &entries {
            assert_eq!(e.ceiling_band, 4, "all equal → band 4");
        }
    }

    #[test]
    fn ceiling_band_spread_assigns_1_and_7() {
        let mut entries = vec![entry_with_ceiling(0), entry_with_ceiling(100)];
        assign_ceiling_bands(&mut entries);
        assert_eq!(entries[0].ceiling_band, 1, "min ceiling → band 1");
        assert_eq!(entries[1].ceiling_band, 7, "max ceiling → band 7");
    }

    #[test]
    fn ceiling_band_single_entry_gets_4() {
        let mut entries = vec![entry_with_ceiling(50)];
        assign_ceiling_bands(&mut entries);
        assert_eq!(entries[0].ceiling_band, 4);
    }

    #[test]
    fn ceiling_band_values_in_1_to_7_range() {
        let ceilings = [10i64, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        let mut entries: Vec<_> = ceilings.iter().map(|&c| entry_with_ceiling(c)).collect();
        assign_ceiling_bands(&mut entries);
        for e in &entries {
            assert!(e.ceiling_band >= 1 && e.ceiling_band <= 7, "band must be 1–7");
        }
    }

    #[test]
    fn ceiling_band_middle_value_gets_middle_band() {
        // Three entries: min=0, mid=50, max=100. Mid should get band 3 or 4.
        let mut entries = vec![
            entry_with_ceiling(0),
            entry_with_ceiling(50),
            entry_with_ceiling(100),
        ];
        assign_ceiling_bands(&mut entries);
        let mid_band = entries[1].ceiling_band;
        assert!(
            mid_band >= 3 && mid_band <= 5,
            "middle ceiling should yield mid-range band, got {mid_band}"
        );
    }

    #[test]
    fn remaining_possible_is_max_minus_total() {
        let rows = vec![
            LeaderboardRawRow {
                user_id: 1,
                user_name: "A".to_string(),
                total_points: 30,
                max_achievable: 100,
            },
            LeaderboardRawRow {
                user_id: 2,
                user_name: "B".to_string(),
                total_points: 80,
                max_achievable: 90,
            },
        ];
        let entries = build_leaderboard(rows, HashMap::new(), HashMap::new());
        assert_eq!(entries[0].remaining_possible, 70); // 100 - 30
        assert_eq!(entries[1].remaining_possible, 10); // 90 - 80
    }

    #[test]
    fn build_leaderboard_calls_assign_ceiling_bands() {
        let rows = vec![
            LeaderboardRawRow {
                user_id: 1,
                user_name: "Low".to_string(),
                total_points: 10,
                max_achievable: 20,
            },
            LeaderboardRawRow {
                user_id: 2,
                user_name: "High".to_string(),
                total_points: 50,
                max_achievable: 80,
            },
        ];
        let entries = build_leaderboard(rows, HashMap::new(), HashMap::new());
        // Low ceiling → band 1, high ceiling → band 7
        assert_eq!(entries[0].ceiling_band, 1);
        assert_eq!(entries[1].ceiling_band, 7);
    }

    #[test]
    fn tiebreak_equal_points_and_ceiling_sorts_alphabetically() {
        // Input is pre-sorted by SQL: same points, same ceiling → alphabetical ASC.
        // Alice (id=2) comes before Zara (id=1) because the DB sorted them that way.
        // build_leaderboard must preserve that order.
        let rows = vec![
            LeaderboardRawRow {
                user_id: 2,
                user_name: "Alice".to_string(),
                total_points: 10,
                max_achievable: 20,
            },
            LeaderboardRawRow {
                user_id: 1,
                user_name: "Zara".to_string(),
                total_points: 10,
                max_achievable: 20,
            },
        ];
        let entries = build_leaderboard(rows, HashMap::new(), HashMap::new());
        assert_eq!(entries[0].user_id, 2, "Alice should be rank 1");
        assert_eq!(entries[1].user_id, 1, "Zara should be rank 2");
    }
}

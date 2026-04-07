use crate::{
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
}

/// Assigns ranks (1-based) and computes `points_behind` relative to the leader.
/// Input must already be sorted by `total_points DESC`.
pub fn build_leaderboard(rows: Vec<LeaderboardRawRow>) -> Vec<LeaderboardEntry> {
    let leader = rows.first().map(|r| r.total_points).unwrap_or(0);
    rows.into_iter()
        .enumerate()
        .map(|(i, r)| LeaderboardEntry {
            rank: i + 1,
            points_behind: leader - r.total_points,
            user_id: r.user_id,
            user_name: r.user_name,
            total_points: r.total_points,
            max_achievable: r.max_achievable,
        })
        .collect()
}

impl LeaderboardEntry {
    pub fn is_leader(&self) -> bool {
        self.points_behind == 0
    }
}

// ── Match breakdown ───────────────────────────────────────────────────────────

pub struct MatchInfo {
    pub id: i64,
    pub home_name: String,
    pub away_name: String,
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
}

/// One league member's prediction row for a group stage match breakdown.
pub struct MatchBreakdownRow {
    pub user_id: i64,
    pub user_name: String,
    pub predicted_outcome: Option<MatchOutcome>,
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
        let entries = build_leaderboard(rows);
        assert_eq!(entries[0].rank, 1);
        assert_eq!(entries[0].points_behind, 0);
        assert_eq!(entries[1].rank, 2);
        assert_eq!(entries[1].points_behind, 5);
    }

    #[test]
    fn build_leaderboard_empty_vec() {
        assert!(build_leaderboard(vec![]).is_empty());
    }
}

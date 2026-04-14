use serde::Deserialize;
use std::collections::HashMap;

use crate::db_types::{KnockoutRound, MatchOutcome};

// ── Page view models ──────────────────────────────────────────────────────────

pub struct MatchRow {
    pub id: i64,
    pub group_name: String,
    pub home_team_name: Option<String>,
    pub away_team_name: Option<String>,
    pub home_crest_url: String,
    pub away_crest_url: String,
    pub scheduled_at: time::OffsetDateTime,
    pub predicted_outcome: Option<MatchOutcome>,
    /// Actual match result; `None` means match not yet played.
    pub actual_outcome: Option<MatchOutcome>,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
    /// Whether the user marked this prediction as a confident pick.
    pub is_confident: bool,
}

impl MatchRow {
    pub fn home_name(&self) -> &str {
        self.home_team_name.as_deref().unwrap_or("TBD")
    }
    pub fn away_name(&self) -> &str {
        self.away_team_name.as_deref().unwrap_or("TBD")
    }
    pub fn is_home_selected(&self) -> bool {
        matches!(self.predicted_outcome, Some(MatchOutcome::Home))
    }
    pub fn is_draw_selected(&self) -> bool {
        matches!(self.predicted_outcome, Some(MatchOutcome::Draw))
    }
    pub fn is_away_selected(&self) -> bool {
        matches!(self.predicted_outcome, Some(MatchOutcome::Away))
    }

    pub fn formatted_kickoff(&self) -> String {
        let fmt = time::format_description::parse("[day] [month repr:short] [hour]:[minute] UTC")
            .expect("static format is valid");
        self.scheduled_at
            .format(&fmt)
            .unwrap_or_else(|_| "TBD".to_string())
    }

    pub fn is_played(&self) -> bool {
        self.actual_outcome.is_some()
    }

    pub fn score_label(&self) -> String {
        match (self.home_score, self.away_score) {
            (Some(h), Some(a)) => format!("{h} – {a}"),
            _ => String::new(),
        }
    }

    /// Returns prediction correctness: `Some(true)` correct, `Some(false)` wrong, `None` pending.
    pub fn correctness(&self) -> Option<bool> {
        match (&self.predicted_outcome, &self.actual_outcome) {
            (Some(pred), Some(actual)) => Some(pred == actual),
            _ => None,
        }
    }
}

pub struct GroupWithMatches {
    pub name: String,
    pub matches: Vec<MatchRow>,
}

pub struct TeamInfo {
    pub id: i64,
    pub name: String,
    pub short_name: String,
    pub crest_url: String,
}

pub struct PlayerInfo {
    pub id: i64,
    pub name: String,
    pub team_name: String,
    pub goals_scored: i32,
}

pub struct KnockoutRoundState {
    pub round: KnockoutRound,
    pub predicted_team_ids: Vec<i64>,
}

impl KnockoutRoundState {
    pub fn has_team(&self, team_id: &i64) -> bool {
        self.predicted_team_ids.contains(team_id)
    }
}

// ── Review view models ────────────────────────────────────────────────────────

pub struct GroupReviewRow {
    pub group_name: String,
    pub home_name: String,
    pub away_name: String,
    pub scheduled_at: time::OffsetDateTime,
    pub predicted_outcome: MatchOutcome,
    pub actual_outcome: Option<MatchOutcome>,
    pub points_awarded: Option<i32>,
}

impl GroupReviewRow {
    pub fn formatted_kickoff(&self) -> String {
        let fmt = time::format_description::parse("[day] [month repr:short] [hour]:[minute] UTC")
            .expect("static format is valid");
        self.scheduled_at
            .format(&fmt)
            .unwrap_or_else(|_| "TBD".to_string())
    }

    pub fn outcome_label(outcome: &MatchOutcome) -> &'static str {
        match outcome {
            MatchOutcome::Home => "Home",
            MatchOutcome::Draw => "Draw",
            MatchOutcome::Away => "Away",
        }
    }

    /// Returns (correct, wrong, pending) — used by templates to pick a colour.
    pub fn score_state(&self) -> &'static str {
        match (&self.actual_outcome, self.points_awarded) {
            (None, _) => "pending",
            (Some(actual), _) if actual == &self.predicted_outcome => "correct",
            _ => "wrong",
        }
    }

    pub fn points_display(&self) -> String {
        match self.points_awarded {
            Some(p) => p.to_string(),
            None => "—".to_string(),
        }
    }
}

pub struct KnockoutReviewRow {
    pub round: KnockoutRound,
    pub team_name: String,
    pub points_awarded: Option<i32>,
}

impl KnockoutReviewRow {
    pub fn round_label(&self) -> &'static str {
        self.round.label()
    }

    pub fn points_display(&self) -> String {
        match self.points_awarded {
            Some(p) => p.to_string(),
            None => "—".to_string(),
        }
    }

    /// Returns `"correct"`, `"wrong"`, or `"pending"` — used by templates to pick a colour.
    pub fn score_state(&self) -> &'static str {
        match self.points_awarded {
            None => "pending",
            Some(0) => "wrong",
            Some(_) => "correct",
        }
    }
}

#[cfg(test)]
mod knockout_review_tests {
    use super::*;

    fn row(points_awarded: Option<i32>) -> KnockoutReviewRow {
        KnockoutReviewRow {
            round: KnockoutRound::Qf,
            team_name: "Test FC".to_string(),
            points_awarded,
        }
    }

    #[test]
    fn score_state_pending_when_no_points() {
        assert_eq!(row(None).score_state(), "pending");
    }

    #[test]
    fn score_state_wrong_when_zero_points() {
        assert_eq!(row(Some(0)).score_state(), "wrong");
    }

    #[test]
    fn score_state_correct_when_positive_points() {
        assert_eq!(row(Some(8)).score_state(), "correct");
    }
}

pub struct TopScorerReviewRow {
    pub player_name: String,
    pub team_name: String,
    pub goals_scored: i32,
    pub points_awarded: Option<i32>,
}

impl TopScorerReviewRow {
    pub fn points_display(&self) -> String {
        match self.points_awarded {
            Some(p) => p.to_string(),
            None => "—".to_string(),
        }
    }
}

// ── Form payloads ─────────────────────────────────────────────────────────────

/// Group stage form: dynamic keys `match_{id}` → "home" | "draw" | "away"
pub type GroupStageForm = HashMap<String, String>;

#[derive(Deserialize)]
pub struct KnockoutForm {
    #[serde(default)]
    pub team_ids: Vec<i64>,
}

#[derive(Deserialize)]
pub struct TopScorerForm {
    #[serde(default)]
    pub player_ids: Vec<i64>,
}

use crate::db_types::{KnockoutRound, MatchOutcome};

/// Points awarded for a correct group stage match prediction.
pub fn group_stage_points(predicted: &MatchOutcome, actual: &MatchOutcome) -> i32 {
    if predicted == actual {
        1
    } else {
        0
    }
}

/// Points awarded per team for a correctly predicted knockout round advancement.
pub fn knockout_points_per_team(round: &KnockoutRound) -> i32 {
    match round {
        KnockoutRound::R32 => 2,
        KnockoutRound::R16 => 3,
        KnockoutRound::Qf => 4,
        KnockoutRound::Sf => 6,
        KnockoutRound::Final => 8,
        KnockoutRound::Winner => 10,
    }
}

/// Points awarded when one of the user's 3 picks is the tournament's top scorer.
/// Reward = 5 bonus + the number of goals the player scored.
pub fn top_scorer_points(goals_scored: i32) -> i32 {
    5 + goals_scored
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Group stage ───────────────────────────────────────────────────────────

    #[test]
    fn group_stage_correct_prediction_scores_one() {
        assert_eq!(
            group_stage_points(&MatchOutcome::Home, &MatchOutcome::Home),
            1
        );
        assert_eq!(
            group_stage_points(&MatchOutcome::Draw, &MatchOutcome::Draw),
            1
        );
        assert_eq!(
            group_stage_points(&MatchOutcome::Away, &MatchOutcome::Away),
            1
        );
    }

    #[test]
    fn group_stage_wrong_prediction_scores_zero() {
        assert_eq!(
            group_stage_points(&MatchOutcome::Home, &MatchOutcome::Away),
            0
        );
        assert_eq!(
            group_stage_points(&MatchOutcome::Draw, &MatchOutcome::Home),
            0
        );
        assert_eq!(
            group_stage_points(&MatchOutcome::Away, &MatchOutcome::Draw),
            0
        );
    }

    #[test]
    fn group_stage_scorer_is_idempotent() {
        let score = group_stage_points(&MatchOutcome::Home, &MatchOutcome::Home);
        assert_eq!(
            score,
            group_stage_points(&MatchOutcome::Home, &MatchOutcome::Home)
        );
    }

    // ── Knockout ──────────────────────────────────────────────────────────────

    #[test]
    fn knockout_points_per_round() {
        assert_eq!(knockout_points_per_team(&KnockoutRound::R32), 2);
        assert_eq!(knockout_points_per_team(&KnockoutRound::R16), 3);
        assert_eq!(knockout_points_per_team(&KnockoutRound::Qf), 4);
        assert_eq!(knockout_points_per_team(&KnockoutRound::Sf), 6);
        assert_eq!(knockout_points_per_team(&KnockoutRound::Final), 8);
        assert_eq!(knockout_points_per_team(&KnockoutRound::Winner), 10);
    }

    #[test]
    fn knockout_points_increase_with_round() {
        let rounds = KnockoutRound::all();
        for window in rounds.windows(2) {
            assert!(
                knockout_points_per_team(&window[0]) < knockout_points_per_team(&window[1]),
                "expected points to increase from {:?} to {:?}",
                window[0],
                window[1]
            );
        }
    }

    // ── Top scorer ────────────────────────────────────────────────────────────

    #[test]
    fn top_scorer_points_is_five_plus_goals() {
        assert_eq!(top_scorer_points(0), 5);
        assert_eq!(top_scorer_points(5), 10);
        assert_eq!(top_scorer_points(8), 13);
    }

    #[test]
    fn top_scorer_scorer_is_idempotent() {
        let goals = 6;
        assert_eq!(top_scorer_points(goals), top_scorer_points(goals));
    }
}

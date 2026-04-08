use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

mod db;
mod handlers;
pub mod models;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/predictions", get(handlers::predictions_page))
        .route("/predictions/group", post(handlers::save_group))
        .route(
            "/predictions/knockout/{round}",
            post(handlers::save_knockout),
        )
        .route("/predictions/top-scorer", post(handlers::save_top_scorer))
        .route(
            "/leagues/{id}/predictions/review",
            get(handlers::predictions_review),
        )
}

// ── Unit tests for round validation ──────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::db_types::KnockoutRound;

    #[test]
    fn round_team_counts_are_correct() {
        let cases = [
            (KnockoutRound::R32, 32),
            (KnockoutRound::R16, 16),
            (KnockoutRound::Qf, 8),
            (KnockoutRound::Sf, 4),
            (KnockoutRound::Final, 2),
            (KnockoutRound::Winner, 1),
        ];
        for (round, expected) in cases {
            assert_eq!(
                round.expected_team_count(),
                expected,
                "{} should require {expected} teams",
                round.label()
            );
        }
    }

    #[test]
    fn invalid_count_differs_from_all_rounds() {
        // A count of 0 is invalid for every round
        for round in KnockoutRound::all() {
            assert_ne!(
                round.expected_team_count(),
                0,
                "0 is never a valid team count"
            );
        }
        // A count of 3 is invalid for every round
        for round in KnockoutRound::all() {
            assert_ne!(round.expected_team_count(), 3, "3 is never a valid count");
        }
    }

    #[test]
    fn round_slugs_round_trip() {
        for round in KnockoutRound::all() {
            let slug = round.slug();
            let parsed = KnockoutRound::from_slug(slug).expect("slug must round-trip");
            assert_eq!(round.slug(), parsed.slug());
        }
    }
}

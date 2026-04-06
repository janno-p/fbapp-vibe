// DB enum mirrors are defined for all types up front; not all are used yet.
#![allow(dead_code)]

/// Rust mirror of the `match_outcome` Postgres enum.
#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "match_outcome", rename_all = "lowercase")]
pub enum MatchOutcome {
    Home,
    Draw,
    Away,
}

/// Rust mirror of the `knockout_round` Postgres enum.
#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "knockout_round")]
pub enum KnockoutRound {
    #[sqlx(rename = "r32")]
    R32,
    #[sqlx(rename = "r16")]
    R16,
    #[sqlx(rename = "qf")]
    Qf,
    #[sqlx(rename = "sf")]
    Sf,
    #[sqlx(rename = "final")]
    Final,
    #[sqlx(rename = "winner")]
    Winner,
}

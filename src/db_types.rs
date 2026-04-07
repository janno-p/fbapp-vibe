/// Rust mirror of the `match_outcome` Postgres enum.
#[derive(Debug, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "match_outcome", rename_all = "lowercase")]
pub enum MatchOutcome {
    Home,
    Draw,
    Away,
}

impl MatchOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Draw => "draw",
            Self::Away => "away",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        match s {
            "home" => Some(Self::Home),
            "draw" => Some(Self::Draw),
            "away" => Some(Self::Away),
            _ => None,
        }
    }
}

impl std::fmt::Display for MatchOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
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

impl KnockoutRound {
    pub fn slug(&self) -> &'static str {
        match self {
            Self::R32 => "r32",
            Self::R16 => "r16",
            Self::Qf => "qf",
            Self::Sf => "sf",
            Self::Final => "final",
            Self::Winner => "winner",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::R32 => "Round of 32",
            Self::R16 => "Round of 16",
            Self::Qf => "Quarter-finals",
            Self::Sf => "Semi-finals",
            Self::Final => "Final",
            Self::Winner => "Winner",
        }
    }

    pub fn expected_team_count(&self) -> usize {
        match self {
            Self::R32 => 32,
            Self::R16 => 16,
            Self::Qf => 8,
            Self::Sf => 4,
            Self::Final => 2,
            Self::Winner => 1,
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        match s {
            "r32" => Some(Self::R32),
            "r16" => Some(Self::R16),
            "qf" => Some(Self::Qf),
            "sf" => Some(Self::Sf),
            "final" => Some(Self::Final),
            "winner" => Some(Self::Winner),
            _ => None,
        }
    }

    pub fn all() -> &'static [KnockoutRound] {
        &[
            Self::R32,
            Self::R16,
            Self::Qf,
            Self::Sf,
            Self::Final,
            Self::Winner,
        ]
    }
}

impl std::fmt::Display for KnockoutRound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

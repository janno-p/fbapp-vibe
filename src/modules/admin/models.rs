use serde::Deserialize;

pub struct Tournament {
    pub id: i64,
    pub external_id: String,
    pub name: String,
    pub season: String,
    pub is_active: bool,
    pub predictions_locked_at: Option<time::OffsetDateTime>,
}

impl Tournament {
    pub fn is_predictions_locked(&self) -> bool {
        self.predictions_locked_at
            .map(|t| t <= time::OffsetDateTime::now_utc())
            .unwrap_or(false)
    }
}

/// Form data for registering a new tournament.
#[derive(Deserialize)]
pub struct RegisterTournamentForm {
    pub code: String,
    pub external_id: String,
    pub name: String,
    pub season: String,
}

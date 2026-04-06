use serde::Deserialize;

#[derive(Debug)]
pub struct League {
    pub id: i64,
    pub name: String,
    pub invite_token: String,
    pub created_by: i64,
    pub created_at: time::OffsetDateTime,
}

pub struct LeagueWithCount {
    pub id: i64,
    pub name: String,
    pub invite_token: String,
    pub member_count: i64,
}

#[derive(Deserialize)]
pub struct CreateLeagueForm {
    pub name: String,
}

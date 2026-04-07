use serde::Deserialize;

#[derive(Debug)]
pub struct League {
    pub id: i64,
    pub name: String,
    pub invite_token: String,
    pub created_by: i64,
    pub created_at: time::OffsetDateTime,
}

pub struct LeagueMember {
    pub name: String,
    pub joined_at: time::OffsetDateTime,
}

impl LeagueMember {
    pub fn formatted_joined_at(&self) -> String {
        let fmt = time::format_description::parse("[day] [month repr:short] [year]")
            .expect("valid format");
        self.joined_at.format(&fmt).unwrap_or_default()
    }
}

pub struct LeagueOverview {
    pub id: i64,
    pub name: String,
    pub created_by: i64,
    pub created_at: time::OffsetDateTime,
    /// `Some` only when the viewer is the creator or an admin.
    pub invite_token: Option<String>,
    pub members: Vec<LeagueMember>,
}

impl LeagueOverview {
    pub fn formatted_created_at(&self) -> String {
        let fmt = time::format_description::parse("[day] [month repr:short] [year]")
            .expect("valid format");
        self.created_at.format(&fmt).unwrap_or_default()
    }
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

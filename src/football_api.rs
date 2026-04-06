use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio::sync::Mutex;

use crate::db_types::MatchOutcome;

const BASE_URL: &str = "https://api.football-data.org/v4";

/// Minimum interval between API requests: 7 s (free tier allows 10 req/min = 6 s min, +1 s buffer).
const MIN_REQUEST_INTERVAL: Duration = Duration::from_secs(7);

// ── Rate limiter ──────────────────────────────────────────────────────────────

struct RateLimiter {
    last_request: Mutex<Option<Instant>>,
    min_interval: Duration,
}

impl RateLimiter {
    fn new(min_interval: Duration) -> Self {
        Self {
            last_request: Mutex::new(None),
            min_interval,
        }
    }

    async fn acquire(&self) {
        let mut last = self.last_request.lock().await;
        if let Some(t) = *last {
            let elapsed = t.elapsed();
            if elapsed < self.min_interval {
                tokio::time::sleep(self.min_interval - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Season {
    pub id: i64,
    /// Format: "YYYY-MM-DD"
    #[serde(rename = "startDate")]
    pub start_date: String,
}

impl Season {
    pub fn year(&self) -> &str {
        &self.start_date[..4.min(self.start_date.len())]
    }
}

#[derive(Debug, Deserialize)]
pub struct Competition {
    pub id: i64,
    pub name: String,
    pub code: String,
    #[serde(rename = "currentSeason")]
    pub current_season: Option<Season>,
}

impl Competition {
    pub fn season_year(&self) -> &str {
        self.current_season
            .as_ref()
            .map(|s| s.year())
            .unwrap_or("Unknown")
    }
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub id: i64,
    pub name: String,
    pub position: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Team {
    pub id: i64,
    pub name: String,
    #[serde(rename = "shortName")]
    pub short_name: String,
    pub tla: String,
    pub crest: Option<String>,
    #[serde(default)]
    pub squad: Vec<Player>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchStatus {
    Scheduled,
    InPlay,
    Paused,
    Finished,
    Suspended,
    Postponed,
    Cancelled,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchWinner {
    HomeTeam,
    AwayTeam,
    Draw,
}

impl MatchWinner {
    pub fn to_outcome(&self) -> MatchOutcome {
        match self {
            MatchWinner::HomeTeam => MatchOutcome::Home,
            MatchWinner::AwayTeam => MatchOutcome::Away,
            MatchWinner::Draw => MatchOutcome::Draw,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScoreDetail {
    pub home: Option<i32>,
    pub away: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MatchScore {
    pub winner: Option<MatchWinner>,
    #[serde(rename = "fullTime")]
    pub full_time: ScoreDetail,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MatchTeam {
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Match {
    pub id: i64,
    #[serde(rename = "utcDate")]
    pub utc_date: String,
    pub status: MatchStatus,
    pub stage: String,
    pub group: Option<String>,
    #[serde(rename = "homeTeam")]
    pub home_team: MatchTeam,
    #[serde(rename = "awayTeam")]
    pub away_team: MatchTeam,
    pub score: MatchScore,
}

#[derive(Debug, Deserialize)]
pub struct ScorerPlayer {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ScorerTeam {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Scorer {
    pub player: ScorerPlayer,
    pub team: ScorerTeam,
    pub goals: Option<i32>,
}

// ── Private response wrappers ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct CompetitionsResponse {
    competitions: Vec<Competition>,
}

#[derive(Deserialize)]
struct TeamsResponse {
    teams: Vec<Team>,
}

#[derive(Deserialize)]
struct MatchesResponse {
    matches: Vec<Match>,
}

#[derive(Deserialize)]
struct ScorersResponse {
    scorers: Vec<Scorer>,
}

// ── Client ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct FootballApiClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
    rate_limiter: Arc<RateLimiter>,
}

impl FootballApiClient {
    pub fn new(api_key: String) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            http,
            api_key,
            base_url: BASE_URL.to_string(),
            rate_limiter: Arc::new(RateLimiter::new(MIN_REQUEST_INTERVAL)),
        })
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> anyhow::Result<T> {
        self.rate_limiter.acquire().await;
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(&url)
            .header("X-Auth-Token", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("football API request to {path} failed with {status}: {body}");
        }

        response.json::<T>().await.map_err(Into::into)
    }

    pub async fn list_competitions(&self) -> anyhow::Result<Vec<Competition>> {
        self.get::<CompetitionsResponse>("/competitions")
            .await
            .map(|r| r.competitions)
    }

    /// Returns teams including their squad (player list) for the given competition code.
    pub async fn get_teams(&self, code: &str) -> anyhow::Result<Vec<Team>> {
        self.get::<TeamsResponse>(&format!("/competitions/{code}/teams"))
            .await
            .map(|r| r.teams)
    }

    /// Returns all matches (fixtures + results) for the given competition code.
    pub async fn get_matches(&self, code: &str) -> anyhow::Result<Vec<Match>> {
        self.get::<MatchesResponse>(&format!("/competitions/{code}/matches"))
            .await
            .map(|r| r.matches)
    }

    /// Returns the top scorers for the given competition code.
    pub async fn get_scorers(&self, code: &str) -> anyhow::Result<Vec<Scorer>> {
        self.get::<ScorersResponse>(&format!("/competitions/{code}/scorers"))
            .await
            .map(|r| r.scorers)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rate_limiter_records_last_request_time() {
        let rl = RateLimiter::new(Duration::from_millis(10));
        assert!(rl.last_request.lock().await.is_none());
        rl.acquire().await;
        assert!(rl.last_request.lock().await.is_some());
    }

    #[tokio::test]
    async fn rate_limiter_waits_between_requests() {
        let interval = Duration::from_millis(50);
        let rl = RateLimiter::new(interval);
        rl.acquire().await;
        let before = Instant::now();
        rl.acquire().await;
        // Second acquire should have slept for ~50ms
        assert!(before.elapsed() >= interval - Duration::from_millis(5));
    }

    #[tokio::test]
    async fn rate_limiter_does_not_wait_after_interval_elapsed() {
        let interval = Duration::from_millis(30);
        let rl = RateLimiter::new(interval);
        rl.acquire().await;
        tokio::time::sleep(interval + Duration::from_millis(20)).await;
        let before = Instant::now();
        rl.acquire().await;
        // Should not have waited since interval already elapsed
        assert!(before.elapsed() < interval);
    }

    #[tokio::test]
    #[ignore = "requires FOOTBALL_API_KEY environment variable and makes real HTTP calls"]
    async fn integration_list_competitions() {
        let api_key = std::env::var("FOOTBALL_API_KEY")
            .expect("FOOTBALL_API_KEY must be set to run this test");
        let client = FootballApiClient::new(api_key).expect("failed to build client");
        let competitions = client
            .list_competitions()
            .await
            .expect("failed to list competitions");
        assert!(!competitions.is_empty());
        let codes: Vec<&str> = competitions.iter().map(|c| c.code.as_str()).collect();
        assert!(
            codes.contains(&"WC") || codes.contains(&"EC"),
            "expected WC or EC in free tier, got: {codes:?}"
        );
    }
}

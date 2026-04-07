use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_url: String,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub football_api_key: String,
    /// Polling interval in seconds when no match is live (default: 120).
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u64,
    /// Polling interval in seconds when a match is in progress (default: 30).
    #[serde(default = "default_poll_interval_live_secs")]
    pub poll_interval_live_secs: u64,
    /// Session inactivity timeout in hours (default: 24).
    #[serde(default = "default_session_duration_hours")]
    pub session_duration_hours: u64,
}

fn default_poll_interval_secs() -> u64 {
    120
}

fn default_poll_interval_live_secs() -> u64 {
    30
}

fn default_session_duration_hours() -> u64 {
    24
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

impl Config {
    pub fn load() -> Result<Self, envy::Error> {
        dotenvy::dotenv().ok();
        envy::from_env::<Config>()
    }
}

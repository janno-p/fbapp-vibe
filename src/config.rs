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
    pub session_secret: String,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub football_api_key: String,
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

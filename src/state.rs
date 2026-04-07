use oauth2::{basic::BasicClient, EndpointNotSet, EndpointSet};
use sqlx::PgPool;

use crate::config::Config;
use crate::football_api::FootballApiClient;

/// OAuth client type after auth URL and token URL have been configured.
pub type OAuthClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub oauth_client: OAuthClient,
    pub football_api: FootballApiClient,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        config: Config,
        oauth_client: OAuthClient,
        football_api: FootballApiClient,
    ) -> Self {
        Self {
            pool,
            config,
            oauth_client,
            football_api,
        }
    }
}

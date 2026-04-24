use oauth2::{EndpointNotSet, EndpointSet, basic::BasicClient};
use sqlx::PgPool;

use crate::config::{Config, OAuthEndpoints};
use crate::football_api::FootballApiClient;

/// OAuth client type after auth URL and token URL have been configured.
pub type OAuthClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub oauth_client: OAuthClient,
    pub oauth_endpoints: OAuthEndpoints,
    pub football_api: FootballApiClient,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        config: Config,
        oauth_client: OAuthClient,
        oauth_endpoints: OAuthEndpoints,
        football_api: FootballApiClient,
    ) -> Self {
        Self {
            pool,
            config,
            oauth_client,
            oauth_endpoints,
            football_api,
        }
    }
}

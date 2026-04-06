use oauth2::{basic::BasicClient, EndpointNotSet, EndpointSet};
use sqlx::PgPool;

use crate::config::Config;

/// OAuth client type after auth URL and token URL have been configured.
pub type OAuthClient = BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub oauth_client: OAuthClient,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config, oauth_client: OAuthClient) -> Self {
        Self {
            pool,
            config,
            oauth_client,
        }
    }
}

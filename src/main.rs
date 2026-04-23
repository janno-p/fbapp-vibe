use std::net::SocketAddr;

use axum_login::AuthManagerLayerBuilder;
use axum_server::tls_rustls::RustlsConfig;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use tower_sessions::{Expiry, SessionManagerLayer, cookie::SameSite};
use tower_sessions_sqlx_store::PostgresStore;

use fbapp_vibe::{
    config::Config,
    football_api,
    modules::auth::AuthBackend,
    routes, session_cleanup,
    state::{AppState, OAuthClient},
    tracing_setup,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    let otlp_enabled = tracing_setup::init_tracing();

    let config = Config::load().map_err(|e| anyhow::anyhow!("failed to load config: {e}"))?;

    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    sqlx::migrate!().run(&pool).await?;

    let tls = match (&config.tls_cert_path, &config.tls_key_path) {
        (Some(cert), Some(key)) => Some(RustlsConfig::from_pem_file(cert, key).await?),
        _ => None,
    };

    // Session store backed by PostgreSQL
    let session_store = PostgresStore::new(pool.clone());
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(tls.is_some())
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(
            config.session_duration_hours as i64,
        )));

    // Auth layer
    let auth_backend = AuthBackend::new(pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    // OAuth client
    let oauth_client = build_oauth_client(&config)?;

    // Football API client
    let football_api = football_api::FootballApiClient::new(config.football_api_key.clone())
        .map_err(|e| anyhow::anyhow!("failed to build football API client: {e}"))?;

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let state = AppState::new(pool.clone(), config, oauth_client, football_api);
    tokio::spawn(fbapp_vibe::polling::run(state.clone()));
    tokio::spawn(session_cleanup::run(pool.clone()));
    let app = routes::router(state).layer(auth_layer);

    tracing::info!(
        "listening on {}",
        if tls.is_some() {
            format!("https://{addr}")
        } else {
            format!("http://{addr}")
        }
    );
    if let Some(tls_config) = tls {
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }

    if otlp_enabled {
        tracing_setup::shutdown_tracing();
    }

    Ok(())
}

fn build_oauth_client(config: &Config) -> anyhow::Result<OAuthClient> {
    let client = BasicClient::new(ClientId::new(config.google_client_id.clone()))
        .set_client_secret(ClientSecret::new(config.google_client_secret.clone()))
        .set_auth_uri(AuthUrl::new(
            "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        )?)
        .set_token_uri(TokenUrl::new(
            "https://oauth2.googleapis.com/token".to_string(),
        )?)
        .set_redirect_uri(RedirectUrl::new(config.google_redirect_url.clone())?);
    Ok(client)
}

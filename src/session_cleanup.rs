use std::time::Duration;

use sqlx::PgPool;
use tracing::{debug, warn};

/// Background task entry point. Loops forever, deleting expired session rows
/// once per hour. Errors are logged as warnings and the loop continues.
pub async fn run(pool: PgPool) {
    debug!("session cleanup task started");
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
        match delete_expired(&pool).await {
            Ok(rows) => debug!(rows, "deleted expired sessions"),
            Err(e) => warn!("session cleanup failed: {e:#}"),
        }
    }
}

async fn delete_expired(pool: &PgPool) -> sqlx::Result<u64> {
    let result = sqlx::query("DELETE FROM tower_sessions WHERE expiry_date < NOW()")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

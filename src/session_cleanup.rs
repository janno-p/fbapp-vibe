use std::time::Duration;

use sqlx::PgPool;
use tracing::{debug, info, warn};

/// Background task entry point. Loops forever, deleting expired session rows
/// once per hour. Errors are logged as warnings and the loop continues.
pub async fn run(pool: PgPool) {
    debug!("session cleanup task started");
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
        match delete_expired(&pool).await {
            Ok(rows) => info!(rows, "deleted expired sessions"),
            Err(e) => warn!("session cleanup failed: {e:#}"),
        }
    }
}

async fn delete_expired(pool: &PgPool) -> sqlx::Result<u64> {
    let result = sqlx::query("DELETE FROM tower_sessions.session WHERE expiry_date <= NOW()")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    // R5.1 — delete_expired removes rows whose expiry_date is in the past.
    #[sqlx::test(migrations = "./migrations")]
    async fn deletes_expired_sessions(pool: PgPool) {
        // Insert one expired session and one future session.
        sqlx::query(
            "INSERT INTO tower_sessions.session (id, data, expiry_date)
             VALUES ('expired-id', '{}', NOW() - INTERVAL '1 hour')",
        )
        .execute(&pool)
        .await
        .expect("insert expired session");

        sqlx::query(
            "INSERT INTO tower_sessions.session (id, data, expiry_date)
             VALUES ('future-id', '{}', NOW() + INTERVAL '1 hour')",
        )
        .execute(&pool)
        .await
        .expect("insert future session");

        let deleted = delete_expired(&pool).await.expect("delete_expired");

        assert_eq!(deleted, 1, "only the expired session should be deleted");

        let remaining: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tower_sessions.session WHERE id = 'future-id'",
        )
        .fetch_one(&pool)
        .await
        .expect("count remaining");

        assert_eq!(remaining, 1, "non-expired session must survive");
    }

    // R5.1 — delete_expired returns 0 when no sessions are expired.
    #[sqlx::test(migrations = "./migrations")]
    async fn returns_zero_when_nothing_to_delete(pool: PgPool) {
        let deleted = delete_expired(&pool).await.expect("delete_expired");
        assert_eq!(deleted, 0);
    }
}

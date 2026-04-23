use sqlx::PgPool;

use super::models::User;

pub async fn find_or_create_user(
    pool: &PgPool,
    google_id: &str,
    email: &str,
    name: &str,
    avatar_url: Option<&str>,
) -> anyhow::Result<User> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (google_id, email, name, avatar_url)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (google_id) DO UPDATE
            SET email      = EXCLUDED.email,
                name       = EXCLUDED.name,
                avatar_url = EXCLUDED.avatar_url
        RETURNING id, google_id, email, name, avatar_url, is_admin
        "#,
        google_id,
        email,
        name,
        avatar_url,
    )
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    // R2.1, R1.4 — creates a new user row from Google profile data.
    #[sqlx::test(migrations = "./migrations")]
    async fn creates_new_user(pool: PgPool) {
        let user = find_or_create_user(
            &pool,
            "g-001",
            "alice@example.com",
            "Alice",
            Some("https://a.example.com/pic.jpg"),
        )
        .await
        .expect("create user");

        assert!(user.id > 0);
        assert_eq!(user.google_id, "g-001");
        assert_eq!(user.email, "alice@example.com");
        assert_eq!(user.name, "Alice");
        assert_eq!(
            user.avatar_url,
            Some("https://a.example.com/pic.jpg".to_string())
        );
        assert!(!user.is_admin);
    }

    // R1.4 — avatar_url may be None.
    #[sqlx::test(migrations = "./migrations")]
    async fn creates_user_without_avatar(pool: PgPool) {
        let user = find_or_create_user(&pool, "g-002", "bob@example.com", "Bob", None)
            .await
            .expect("create user");
        assert!(user.avatar_url.is_none());
    }

    // R1.4 — re-inserting with the same google_id updates email/name/avatar_url.
    #[sqlx::test(migrations = "./migrations")]
    async fn updates_profile_on_google_id_conflict(pool: PgPool) {
        find_or_create_user(&pool, "g-003", "old@example.com", "Old Name", None)
            .await
            .expect("initial insert");

        let updated = find_or_create_user(
            &pool,
            "g-003",
            "new@example.com",
            "New Name",
            Some("https://n.example.com/pic.jpg"),
        )
        .await
        .expect("upsert");

        assert_eq!(updated.email, "new@example.com");
        assert_eq!(updated.name, "New Name");
        assert_eq!(
            updated.avatar_url,
            Some("https://n.example.com/pic.jpg".to_string())
        );
    }

    // R1.4 — upsert must not reset is_admin to false.
    #[sqlx::test(migrations = "./migrations")]
    async fn preserves_is_admin_flag_on_conflict(pool: PgPool) {
        let user = find_or_create_user(&pool, "g-004", "admin@example.com", "Admin", None)
            .await
            .expect("create user");

        sqlx::query!("UPDATE users SET is_admin = true WHERE id = $1", user.id)
            .execute(&pool)
            .await
            .expect("grant admin");

        let refreshed = find_or_create_user(&pool, "g-004", "admin@example.com", "Admin", None)
            .await
            .expect("upsert");

        assert!(refreshed.is_admin, "is_admin must survive an upsert");
    }
}

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
        RETURNING id, google_id, email, name, avatar_url
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

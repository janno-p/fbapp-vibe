use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    #[allow(dead_code)]
    pub google_id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub is_admin: bool,
}

impl axum_login::AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.email.as_bytes()
    }
}

/// Credentials type — unused for OAuth (login is called directly after token exchange).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Credentials;

/// Google userinfo API response.
#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
}

#[cfg(test)]
mod tests {
    use axum_login::AuthUser as _;

    use super::*;

    fn sample_user() -> User {
        User {
            id: 42,
            google_id: "google-abc".to_string(),
            email: "alice@example.com".to_string(),
            name: "Alice".to_string(),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            is_admin: false,
        }
    }

    // R2.1 — User struct has all six required fields with correct types.
    #[test]
    fn user_has_all_required_fields() {
        let u = sample_user();
        assert_eq!(u.id, 42_i64);
        assert_eq!(u.google_id, "google-abc");
        assert_eq!(u.email, "alice@example.com");
        assert_eq!(u.name, "Alice");
        assert_eq!(
            u.avatar_url,
            Some("https://example.com/avatar.jpg".to_string())
        );
        assert!(!u.is_admin);
    }

    // R2.1 — avatar_url is optional.
    #[test]
    fn user_avatar_url_is_optional() {
        let u = User {
            avatar_url: None,
            ..sample_user()
        };
        assert!(u.avatar_url.is_none());
    }

    // R2.2 — AuthUser::id() returns the user's database id.
    #[test]
    fn auth_user_id_returns_user_id() {
        let u = sample_user();
        assert_eq!(u.id(), 42_i64);
    }

    // R2.3 — session_auth_hash is derived from the email field.
    #[test]
    fn session_auth_hash_is_email_bytes() {
        let u = sample_user();
        assert_eq!(u.session_auth_hash(), b"alice@example.com");
    }

    // R3.2 — Hash changes when email changes, invalidating the session.
    #[test]
    fn session_auth_hash_changes_when_email_changes() {
        let u1 = sample_user();
        let u2 = User {
            email: "other@example.com".to_string(),
            ..sample_user()
        };
        assert_ne!(u1.session_auth_hash(), u2.session_auth_hash());
    }

    // R1.7 — GoogleUserInfo deserialises all required fields.
    #[test]
    fn google_user_info_deserialises() {
        let json = r#"{"id":"123","email":"bob@example.com","name":"Bob","picture":"https://p.example.com/b.jpg"}"#;
        let info: GoogleUserInfo = serde_json::from_str(json).expect("deserialise");
        assert_eq!(info.id, "123");
        assert_eq!(info.email, "bob@example.com");
        assert_eq!(info.name, "Bob");
        assert_eq!(
            info.picture,
            Some("https://p.example.com/b.jpg".to_string())
        );
    }

    // R1.7 — GoogleUserInfo picture field is optional.
    #[test]
    fn google_user_info_picture_is_optional() {
        let json = r#"{"id":"1","email":"c@example.com","name":"C"}"#;
        let info: GoogleUserInfo = serde_json::from_str(json).expect("deserialise");
        assert!(info.picture.is_none());
    }
}

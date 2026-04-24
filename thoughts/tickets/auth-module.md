---
title: Auth module with Google OAuth and landing pages
source: .claude/tasks/done/0002-auth-module.md
source_id: 0002
source_status: done
source_title: Auth module with Google OAuth and landing pages
status: done
type: feature
adrs: [0005, 0007, 0008, 0009, 0010, 0015]
refs: []
created: 2026-04-06
started: 2026-04-06
completed: 2026-04-06
---

## Summary

Implement Google OAuth authentication and two landing pages: a public home page for unauthenticated visitors (with a "Sign in with Google" button) and a protected dashboard page for authenticated users. This establishes the auth foundation all future features build upon.

## Acceptance Criteria

- [ ] `GET /` returns the public landing page with a "Sign in with Google" button when unauthenticated
- [ ] `GET /` redirects to `/dashboard` when the user is already authenticated
- [ ] `GET /auth/login` redirects to Google's OAuth consent screen
- [ ] `GET /auth/callback` exchanges the code, fetches the Google profile, creates or finds the user in the DB, creates a session, and redirects to `/dashboard`
- [ ] `GET /auth/callback` returns `400 Bad Request` if the `state` parameter does not match
- [ ] `POST /auth/logout` destroys the session and redirects to `/`
- [ ] `GET /dashboard` returns the authenticated landing page showing the user's name and avatar
- [ ] `GET /dashboard` returns `401 Unauthorized` when the user is not authenticated
- [ ] Users table exists with columns: `id`, `google_id`, `email`, `name`, `avatar_url`, `created_at`
- [ ] Sessions table exists (created by `tower-sessions-sqlx-store` migration)
- [ ] `cargo build` succeeds with zero warnings and zero clippy errors

## Implementation Context

### Relevant files to create / modify

```
migrations/
  0002_create_users.sql          # users table
  0003_create_sessions.sql       # sessions table (tower-sessions schema)

src/
  config.rs                      # add: google_client_id, google_client_secret,
                                 #      google_redirect_url, session_secret
  main.rs                        # add: SessionManagerLayer, AuthManagerLayer to router
  modules/
    mod.rs                       # register auth module
    auth/
      mod.rs                     # pub fn router() -> Router
      handlers.rs                # login, callback, logout, home, dashboard
      db.rs                      # find_or_create_user
      models.rs                  # User struct + AuthUser impl
  routes.rs                      # merge auth::router()

templates/
  home/
    index.html                   # public landing page (extends base.html)
  dashboard/
    index.html                   # authenticated landing page (extends base.html)
```

### ADR constraints

- **ADR-0007**: `auth::router()` is the only symbol exported from `src/modules/auth/mod.rs`; `handlers.rs` and `db.rs` are private to the module
- **ADR-0008**: All new config fields added to `Config` struct in `src/config.rs`; new vars added to `.env.example`
- **ADR-0009**: Handlers return `Result<impl IntoResponse, AppError>`; add `AppError::Unauthorized` variant
- **ADR-0015**: Use `oauth2` crate with PKCE and `state` parameter; use `reqwest` for Google userinfo; sessions via `tower-sessions` + `tower-sessions-sqlx-store`

### New Cargo.toml dependencies

```toml
oauth2 = "4"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
tower-sessions = "0.12"
tower-sessions-sqlx-store = { version = "0.14", features = ["postgres"] }
axum-login = "0.16"
serde_json = "1"
```

### Config additions

```rust
// src/config.rs — add to Config struct
pub google_client_id: String,
pub google_client_secret: String,
pub google_redirect_url: String,
pub session_secret: String,
```

### Migrations

**`migrations/0002_create_users.sql`**
```sql
CREATE TABLE users (
    id          BIGSERIAL PRIMARY KEY,
    google_id   TEXT NOT NULL UNIQUE,
    email       TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    avatar_url  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**`migrations/0003_create_sessions.sql`**
```sql
CREATE TABLE tower_sessions (
    id           TEXT PRIMARY KEY,
    data         BYTEA NOT NULL,
    expiry_date  TIMESTAMPTZ NOT NULL
);
```

### User model and AuthUser trait

```rust
// src/modules/auth/models.rs
use axum_login::AuthUser;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub google_id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

impl AuthUser for User {
    type Id = i64;
    fn id(&self) -> Self::Id { self.id }
    fn session_auth_hash(&self) -> &[u8] { self.email.as_bytes() }
}
```

### DB query

```rust
// src/modules/auth/db.rs
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
            SET email = EXCLUDED.email,
                name  = EXCLUDED.name,
                avatar_url = EXCLUDED.avatar_url
        RETURNING id, google_id, email, name, avatar_url
        "#,
        google_id, email, name, avatar_url
    )
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}
```

### OAuth handler flow

```rust
// src/modules/auth/handlers.rs

// GET /auth/login
// 1. Build OAuth client from config
// 2. Generate PKCE challenge + verifier
// 3. Generate random state
// 4. Store (state, pkce_verifier) in session
// 5. Redirect to Google authorization URL

// GET /auth/callback?code=...&state=...
// 1. Extract state + code from query params
// 2. Load (expected_state, pkce_verifier) from session
// 3. Verify state matches — return 400 if not
// 4. Exchange code + pkce_verifier for token
// 5. GET https://www.googleapis.com/oauth2/v2/userinfo with bearer token
// 6. Call db::find_or_create_user with profile data
// 7. Log in via auth_session.login(&user)
// 8. Redirect to /dashboard

// POST /auth/logout
// 1. auth_session.logout()
// 2. Redirect to /
```

### AppError additions

```rust
// src/error.rs — add variants
#[error("unauthorized")]
Unauthorized,
```
Map `Unauthorized` → `StatusCode::UNAUTHORIZED` in `IntoResponse`.

### SessionManagerLayer wiring in main.rs

```rust
// Build session store
let session_store = SqliteStore::new(pool.clone()); // replace with SqlxStore for postgres
// (use tower_sessions_sqlx_store::PostgresStore)
let session_layer = SessionManagerLayer::new(session_store)
    .with_secure(true)
    .with_same_site(SameSite::Lax);

// Build auth layer
let backend = AuthBackend::new(pool.clone());
let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

// Apply to router
let app = routes::router(state).layer(auth_layer);
```

### Templates

**`templates/home/index.html`** — extends `layout/base.html`:
- Centered hero section
- App name and one-line description
- "Sign in with Google" button linking to `/auth/login`

**`templates/dashboard/index.html`** — extends `layout/base.html`:
- Welcome message with `{{ user.name }}`
- User avatar if `avatar_url` is set
- Logout button (`<form method="post" action="/auth/logout">`)

### .env.example additions

```
GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-client-secret
GOOGLE_REDIRECT_URL=http://localhost:3000/auth/callback
SESSION_SECRET=change-me-to-a-random-64-byte-secret
```

## Outcome

Full Google OAuth flow implemented. Key implementation notes:

- `AuthBackend` and `AuthSession` type alias defined in `src/modules/auth/mod.rs`; `AuthBackend` is exported so `main.rs` can build the `AuthManagerLayer`
- `AppState` extended with `oauth_client: BasicClient`; `build_oauth_client()` in `main.rs` propagates URL parse errors via `?` — no `unwrap()`
- Session layer uses `with_secure(false)` for local HTTP development; must be set to `true` in production behind HTTPS
- `dashboard/index.html` uses Askama's `{% if let Some(avatar) = user.avatar_url %}` for optional avatar rendering; falls back to an initial letter avatar
- `AppError` gained `Unauthorized` (401) and `BadRequest` (400) variants; unexpected errors log at `error`, expected errors at `warn`
- `tower_sessions_sqlx_store::PostgresStore` uses the sessions table created by migration `0003`

Canonical auth regression references:
- HTTP-level coverage: `tests/auth_routes.rs:286`, `tests/auth_routes.rs:293`, `tests/auth_routes.rs:307`, `tests/auth_routes.rs:458`, `tests/auth_routes.rs:484`, `tests/auth_routes.rs:500`
- Runtime semantics: `src/modules/auth/handlers.rs:58`, `src/modules/auth/handlers.rs:172`, `src/modules/admin/mod.rs:34`, `src/error.rs:44`

Follow-up tasks: _none_

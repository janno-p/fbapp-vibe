# ADR-0011: Session-Based Authentication with tower-sessions and axum-login 🔐

## Status

✅ Accepted

## Date

2026-04-05

## Context

The application requires user authentication. Two fundamental approaches were evaluated — session-based and JWT — along with the Rust crates available to implement each.

### Strategy Comparison

| | **Session-based** | **JWT** |
|--|------------------|--------|
| State storage | Server-side (DB or memory) | Stateless — token stored client-side |
| Revocation | ✅ Instant — delete session from DB | ❌ Hard — must wait for token expiry |
| HTMX / server-rendered fit | ✅ Natural — cookie sent automatically | ❌ Awkward — requires JS to attach `Authorization` header |
| Scalability | Requires shared session store | Stateless, trivially horizontal |
| Complexity | Low | Medium — token refresh, rotation, secure storage |
| Best for | Server-rendered web apps | SPAs, mobile apps, third-party API consumers |

### Implementation Options (session-based)

| | **`tower-sessions` + `axum-login`** | **`tower-sessions` only** | **Roll your own** |
|--|-------------------------------------|--------------------------|-----------------|
| Session lifecycle | ✅ Managed | ✅ Managed | Manual |
| DB-backed store | ✅ `tower-sessions-sqlx-store` | ✅ Same | Manual |
| Typed user extraction | ✅ `AuthSession` extractor | Manual per handler | Manual |
| Password hashing | `argon2` (separate crate) | Same | Same |
| Axum integration | ✅ First-class | ✅ First-class | Manual middleware |
| Complexity | Low–medium | Low + manual auth logic | High |

Password hashing algorithm options:

| | **`argon2`** | **`bcrypt`** | **`scrypt`** |
|--|-------------|-------------|-------------|
| Current recommendation | ✅ OWASP recommended | Widely used, older | Memory-hard, less common |
| Memory-hard | ✅ Yes | ❌ No | ✅ Yes |
| Rust crate | `argon2` (RustCrypto) | `bcrypt` | `scrypt` |

## Decision

We will use **session-based authentication** 🔐 implemented with:

- **`tower-sessions`** — session lifecycle management with a SQLx-backed store
- **`tower-sessions-sqlx-store`** — persists sessions in PostgreSQL (consistent with ADR-0005)
- **`axum-login`** — typed `AuthSession` extractor and user loading layer for Axum
- **`argon2`** — password hashing (OWASP recommended)

## Rationale

1. 🍪 **Natural fit for HTMX**: The browser automatically includes session cookies on every request, including HTMX partial requests. No JavaScript is required to attach authentication credentials, and no token refresh logic is needed.

2. ⚡ **Instant session revocation**: Sessions are stored in PostgreSQL. Logging out, revoking access, or force-expiring all sessions for a user is a single `DELETE` query — impossible to achieve cleanly with stateless JWTs.

3. 🔌 **SQLx store — no new infrastructure**: `tower-sessions-sqlx-store` persists sessions in the existing PostgreSQL database (ADR-0005). There is no need to introduce Redis or another session store dependency.

4. 🛡️ **`axum-login` reduces boilerplate**: The `AuthSession` extractor automatically loads the authenticated user from the session on every request. Protected routes reject unauthenticated requests via a middleware layer, keeping auth logic out of individual handlers.

5. 🔒 **`argon2` for password security**: Argon2id is the current OWASP-recommended password hashing algorithm. It is memory-hard, resistant to GPU and ASIC brute-force attacks, and well-maintained in the RustCrypto ecosystem.

## Architecture

```
Request
  │
  ├─ SessionManagerLayer (tower-sessions)    ← loads/saves session cookie
  │
  ├─ AuthManagerLayer (axum-login)           ← loads User from session
  │
  └─ Handler
       └─ AuthSession extractor             ← typed access to current user
```

### User trait implementation

```rust
// src/modules/auth/models.rs
#[async_trait]
impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id { self.id }
    fn session_auth_hash(&self) -> &[u8] { self.password_hash.as_bytes() }
}
```

### Protected route example

```rust
// src/modules/users/handlers.rs
async fn dashboard(
    auth_session: AuthSession,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    // ...
}
```

### Password hashing

```rust
// src/modules/auth/db.rs
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    let parsed = PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
}
```

## Trade-offs and Risks ⚠️

- 🗄️ **Session store adds DB load**: Every authenticated request reads the session from PostgreSQL. Connection pooling (via SQLx's `PgPool`) keeps this overhead low, but it is a consideration at high request volumes.
- 🔄 **Horizontal scaling requires shared session store**: All instances must connect to the same PostgreSQL session store. This is satisfied by the existing managed PostgreSQL setup and requires no additional infrastructure.
- 🔒 **Cookie security configuration is critical**: Session cookies must be configured with `Secure`, `HttpOnly`, and `SameSite=Lax` flags in production. Misconfiguration leads to session hijacking or CSRF vulnerabilities.
- 🚫 **No built-in OAuth2/social login**: This implementation covers username/password authentication only. Adding OAuth2 (Google, GitHub) would require integrating `oauth2` crate alongside `axum-login` in a future ADR.

## Consequences

- 🔐 Authentication is implemented in `src/modules/auth/` following the modular monolith structure (ADR-0007).
- 🗄️ Session data is persisted in a `sessions` table in PostgreSQL, created via a SQLx migration.
- 🍪 Session cookies are configured with `Secure=true`, `HttpOnly=true`, `SameSite=Lax` in all non-development environments.
- 🛡️ `SessionManagerLayer` and `AuthManagerLayer` are applied at the top-level router in `src/routes.rs`.
- 🔒 Passwords are always hashed with `argon2` before storage; plaintext passwords are never persisted or logged.
- 🚫 `unwrap()` on `auth_session.user` is forbidden; handlers must explicitly handle the unauthenticated case and return `AppError::Unauthorized`.
- 🌍 The `SESSION_SECRET` environment variable (added to `Config` per ADR-0008) is used to sign session cookies and must be a securely generated random value in production.

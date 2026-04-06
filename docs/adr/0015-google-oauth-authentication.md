# ADR-0015: Google OAuth Authentication 🔐

## Status

✅ Accepted

## Supersedes

[ADR-0011](0011-authentication-strategy.md) — Session-Based Authentication with tower-sessions and axum-login

## Date

2026-04-06

## Context

ADR-0011 established session-based authentication using username/password with `argon2` hashing. The requirement has changed: the application will delegate authentication entirely to Google via OAuth 2.0. This eliminates password management, registration flows, and credential storage while retaining the session-based architecture.

The core session infrastructure (`tower-sessions`, `axum-login`) from ADR-0011 remains valid — only the credential mechanism changes from password verification to OAuth token exchange.

### Why Google OAuth over username/password

| Concern | Username/Password | Google OAuth |
|---------|------------------|-------------|
| Password storage | ❌ Must hash and store securely | ✅ Not applicable |
| Password reset flow | ❌ Must implement | ✅ Not applicable |
| Credential stuffing risk | ❌ Present | ✅ Eliminated |
| Email verification | ❌ Must implement | ✅ Google guarantees verified email |
| MFA | ❌ Must implement separately | ✅ Inherited from Google account |
| Implementation scope | Larger | Smaller |

### OAuth 2.0 flow

```
User clicks "Sign in with Google"
  │
  ▼
GET /auth/login
  └─► Redirect to Google consent screen (with state + PKCE)
          │
          ▼
      User authenticates with Google
          │
          ▼
GET /auth/callback?code=...&state=...
  ├─ Verify state parameter (CSRF protection)
  ├─ Exchange code for access token (via oauth2 crate)
  ├─ Fetch user profile from Google userinfo endpoint (via reqwest)
  ├─ Find or create user in database
  └─ Create session → redirect to /dashboard
```

## Decision

We will use **Google OAuth 2.0** 🔐 for authentication, implemented with:

- **`oauth2`** — OAuth 2.0 client (PKCE, state, token exchange)
- **`reqwest`** — HTTP client for fetching Google userinfo after token exchange
- **`tower-sessions`** + **`tower-sessions-sqlx-store`** — session lifecycle, PostgreSQL-backed
- **`axum-login`** — typed `AuthSession` extractor, user loading layer

`argon2` from ADR-0011 is no longer required.

## Rationale

1. 🔒 **No credential storage**: User passwords are never seen, stored, or managed. Google handles all credential security including MFA, breach detection, and account recovery.

2. 🛡️ **CSRF protection via state parameter**: The `oauth2` crate generates a cryptographically random `state` value per login attempt, verified on callback, preventing cross-site request forgery.

3. 🔑 **PKCE**: Proof Key for Code Exchange is used to prevent authorisation code interception attacks.

4. 📧 **Verified email guaranteed**: Google only returns verified email addresses, eliminating the need for an email verification flow.

5. 🏗️ **Reduced scope**: Removing password management, registration forms, password reset, and email verification significantly reduces implementation and ongoing maintenance burden.

6. 🍪 **Session continuity unchanged**: Post-authentication, the session model from ADR-0011 is preserved. The browser holds a session cookie; the server holds the session record in PostgreSQL. Logout destroys the session server-side.

## Data Model

```sql
-- migrations/0002_create_users.sql
CREATE TABLE users (
    id          BIGSERIAL PRIMARY KEY,
    google_id   TEXT NOT NULL UNIQUE,
    email       TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    avatar_url  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- migrations/0003_create_sessions.sql
-- Created by tower-sessions-sqlx-store schema
```

## New Configuration

The following environment variables are added to `Config` (ADR-0008):

| Variable | Description |
|----------|-------------|
| `GOOGLE_CLIENT_ID` | OAuth client ID from Google Cloud Console |
| `GOOGLE_CLIENT_SECRET` | OAuth client secret from Google Cloud Console |
| `GOOGLE_REDIRECT_URL` | Callback URL registered in Google Cloud Console (e.g. `http://localhost:3000/auth/callback`) |
| `SESSION_SECRET` | Random secret for signing session cookies (min 64 bytes) |

## Trade-offs and Risks ⚠️

- 🌍 **Google dependency**: Authentication is unavailable if Google's OAuth service is down. This is an acceptable trade-off given Google's reliability and the elimination of self-managed credential complexity.
- 🔒 **Google account required**: Users must have a Google account. This is acceptable for the current scope; adding other providers (GitHub, Microsoft) can be done in a future ADR by extending the same OAuth pattern.
- 🔄 **Token is not stored**: Only the user's profile information is stored after login; the OAuth access token is not persisted. If the application later needs to call Google APIs on behalf of the user, token storage must be added.
- 📋 **Google Cloud Console setup required**: Developers must register the app in Google Cloud Console and configure authorised redirect URIs. This is a one-time setup step documented in the project README.

## Consequences

- 🔐 Authentication is handled entirely by Google; the application stores only `google_id`, `email`, `name`, and `avatar_url`.
- 🍪 Sessions are PostgreSQL-backed via `tower-sessions-sqlx-store`; `SESSION_SECRET` is required in all environments.
- 🚫 No password fields, password reset flows, or email verification are implemented.
- 🌍 The Google redirect URL must be registered in Google Cloud Console for each environment (local, staging, production).
- 📦 `argon2` is not a dependency; `oauth2` and `reqwest` are added instead.
- 🔒 The `state` parameter is verified on every callback; mismatched state returns `400 Bad Request`.

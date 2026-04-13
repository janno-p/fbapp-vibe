---
created: 2026-04-10T00:00:00Z
last_edited: 2026-04-13T00:00:00Z
---

# Cavekit: Authentication & Session Management

## Scope

User authentication via Google OAuth, session persistence, role-based access control (admin vs regular user), and automatic session cleanup.

## Requirements

### R1: Google OAuth Login Flow
**Description:** Users can initiate login via Google, exchange credentials, and be automatically logged in to the application.

**Acceptance Criteria:**
- [ ] GET `/auth/login` redirects to Google OAuth authorization endpoint
- [ ] Google OAuth callback handler at GET `/auth/callback` accepts authorization code
- [ ] Callback exchanges authorization code for access token and retrieves user info
- [ ] User info (Google ID, email, name, avatar URL) is stored or updated in database
- [ ] User session is created and stored in PostgreSQL `tower_sessions` table after successful login
- [ ] User is redirected to `/dashboard` after successful login
- [ ] Unauthenticated access to protected routes returns 401 Unauthorized

**Dependencies:** None (auth is foundational)

### R2: User Model
**Description:** User accounts store identity, contact, and role information.

**Acceptance Criteria:**
- [ ] User record has: id (i64, PK), google_id (string), email (string, unique), name (string), avatar_url (optional string), is_admin (boolean)
- [ ] User model implements `axum_login::AuthUser` trait for session integration
- [ ] User can be loaded by ID from database via `AuthBackend.get_user()`

**Dependencies:** None

### R3: Session Storage and Restoration
**Description:** User sessions persist across requests and are restored via `AuthSession` extractor.

**Acceptance Criteria:**
- [ ] Sessions are stored in PostgreSQL `tower_sessions` table (tower-sessions middleware)
- [ ] Session auth hash is derived from user email to detect invalidation on email change
- [ ] Session is automatically available in handlers via `AuthSession` extractor
- [ ] Expired sessions are invalid and return 401 Unauthorized
- [ ] Logout via POST `/auth/logout` destroys the session and redirects to homepage

**Dependencies:** R2 (User Model)

### R4: Admin Role Access Control
**Description:** Only users with `is_admin = true` can access admin panels and tournament management functions.

**Acceptance Criteria:**
- [ ] `AdminUser` extractor exists and returns 403 Forbidden if `is_admin = false`
- [ ] `AdminUser` can be extracted in handlers to gate admin-only routes
- [ ] Admin routes (tournament, league management) use `AdminUser` extractor
- [ ] Regular users attempting admin routes receive 403 Forbidden response

**Dependencies:** R2 (User Model), R3 (Session Storage)

### R5: Session Cleanup
**Description:** Expired sessions are periodically removed from the database to prevent unbounded growth.

**Acceptance Criteria:**
- [ ] Background task runs on a defined schedule (hourly or per configuration)
- [ ] Background task deletes all rows from `tower_sessions` where `expiry_date <= now()`
- [ ] Cleanup task logs the number of sessions deleted at info level
- [ ] Cleanup task continues even if no sessions exist to clean
- [ ] Cleanup task restarts automatically if it crashes (supervisor responsibility, not this cavekit)

**Dependencies:** R3 (Session Storage)

### R6: Public Pages
**Description:** Unauthenticated users can view some pages before logging in.

**Acceptance Criteria:**
- [ ] GET `/` renders home page (unauthenticated)
- [ ] Home page displays login link
- [ ] GET `/dashboard` is protected; unauthenticated users are redirected to `/auth/login`

**Dependencies:** R1 (OAuth Login)

### R7: Integration Test Coverage for Auth Flows
**Description:** Critical auth behaviors must be verified by integration tests that exercise the real HTTP stack, not unit tests that re-implement extractor logic inline.

**Acceptance Criteria:**
- [ ] GET `/dashboard` returns 401 Unauthorized for unauthenticated requests (integration test, not unit test)
- [ ] GET `/` redirects authenticated users to `/dashboard`
- [ ] POST `/auth/logout` destroys the session; subsequent requests with the destroyed session token return 401
- [ ] AdminUser extractor rejects non-admin users at the HTTP level (integration test calling an admin route, not a unit test duplicating extractor logic)
- [ ] Session is invalidated when user email changes (integration test updating DB email and re-issuing request)
- [ ] Expired sessions return 401 (integration test with manually expired session row)

**Dependencies:** R1, R2, R3, R4 (all auth infrastructure must be in place)

## Out of Scope

- Password-based authentication (OAuth only)
- Multi-factor authentication
- Social login providers other than Google
- Email verification or confirmation flows
- Role-based permissions beyond binary admin/user split
- Account deletion or password reset flows
- Session management UI (listing active sessions, remote logout)
- Account linking (one Google account per user only)

## Changes
- 2026-04-13: Added R7 (Integration Test Coverage for Auth Flows) — discovered during inspection (findings F-002, F-003, F-004, F-005, F-008, F-009). Unit tests in admin/mod.rs re-implement extractor logic instead of testing the real code; core auth behaviors (session destruction, expired session 401, dashboard protection) have no integration test coverage.

## Source Traceability

### Brown-field Status: Complete
All acceptance criteria are satisfied by existing code.

### Source Files
- `src/modules/auth/models.rs` — User struct, GoogleUserInfo, Credentials types
- `src/modules/auth/mod.rs` — AuthBackend implementation, router() with routes
- `src/modules/auth/handlers.rs` — home, dashboard, login, callback, logout handlers
- `src/modules/auth/db.rs` — user upsert and fetch queries
- `src/session_cleanup.rs` — background task for expired session deletion
- `migrations/0002_create_users.sql` — users table schema
- `migrations/0003_create_sessions.sql`, `0004_fix_sessions.sql` — tower_sessions table

### Implementation Notes
- Uses `axum-login` crate for auth session layer and `AuthUser` trait
- OAuth token exchange done via `oauth2` crate with `BasicClient` in AppState
- Session storage via `tower-sessions` with PostgreSQL backend
- Admin check is a custom `AdminUser` extractor in `src/extractors.rs`

## Cross-References
- Consumed by: **cavekit-tournament.md** (AdminUser gates tournament routes)
- Consumed by: **cavekit-leagues.md** (user context, league membership)
- Consumed by: **cavekit-predictions.md** (AuthSession for user predictions)
- Consumed by: **cavekit-standings.md** (user context for leaderboard access)
- Consumed by: **cavekit-badges.md** (user_id in achievement awards)

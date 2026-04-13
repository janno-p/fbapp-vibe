# Plan: Auth Tier 0 — Core Infrastructure

## Source Kits
- cavekit-auth.md: R1, R2, R3

## Implementation Sequence

### T-001: Verify User Model Schema and AuthUser Trait
**Cavekit Requirement:** cavekit-auth/R2
**Acceptance Criteria Mapped:**
- User record has: id (i64, PK), google_id (string), email (string, unique), name (string), avatar_url (optional string), is_admin (boolean)
- User model implements `axum_login::AuthUser` trait for session integration
- User can be loaded by ID from database via `AuthBackend.get_user()`

**blockedBy:** none
**Effort:** M
**Description:**
1. Read `src/modules/auth/models.rs` to verify User struct has all 7 required fields with correct types
2. Confirm User struct derives/implements `axum_login::AuthUser` trait
3. Read `src/modules/auth/db.rs` and verify `fetch_user_by_id()` query exists and is called by `AuthBackend.get_user()`
4. Check `migrations/0002_create_users.sql` — verify users table schema matches model (primary key, unique email, nullable avatar_url)
5. Write integration test in `tests/auth_integration.rs` (create if needed):
   - Create a test user in DB
   - Load user by ID via `AuthBackend.get_user()`
   - Assert all fields are populated correctly
   - Assert User trait is callable (can get user_id, session auth hash)

**Files:**
- `src/modules/auth/models.rs` (read)
- `src/modules/auth/db.rs` (read)
- `migrations/0002_create_users.sql` (read)
- `tests/auth_integration.rs` (create/edit)

**Test Strategy:**
- Integration test: create user, fetch, verify schema completeness
- Run `cargo test tests::auth_user_model` — should pass

---

### T-002: Verify Google OAuth Login Flow to Callback
**Cavekit Requirement:** cavekit-auth/R1
**Acceptance Criteria Mapped:**
- GET `/auth/login` redirects to Google OAuth authorization endpoint
- Google OAuth callback handler at GET `/auth/callback` accepts authorization code
- Callback exchanges authorization code for access token and retrieves user info
- User info (Google ID, email, name, avatar URL) is stored or updated in database
- User session is created and stored in PostgreSQL `tower_sessions` table after successful login
- User is redirected to `/dashboard` after successful login
- Unauthenticated access to protected routes returns 401 Unauthorized

**blockedBy:** none
**Effort:** M
**Description:**
1. Read `src/modules/auth/handlers.rs` — verify `login_handler()` exists and returns redirect to OAuth endpoint (check `BasicClient.authorize_url()`)
2. Read `src/modules/auth/handlers.rs` — verify `callback_handler()` exists and:
   - Extracts authorization code from query params
   - Calls `BasicClient.exchange_code()` to get token
   - Makes request to Google userinfo endpoint to fetch user details
3. Read `src/modules/auth/db.rs` — verify `upsert_user()` query updates email, name, avatar_url for existing google_id or creates new user
4. Verify callback creates session via `auth_session.login()` and stores in tower_sessions table
5. Verify callback redirects to `/dashboard`
6. Write integration test in `tests/auth_oauth_flow.rs`:
   - Mock Google OAuth endpoints (consider using `wiremock` or `mockito`)
   - Call GET `/auth/login` — verify 302 redirect to Google
   - Simulate OAuth callback with mock authorization code
   - Verify user is created/updated in DB
   - Verify session is created in tower_sessions
   - Verify redirect to `/dashboard`
   - GET protected route with session cookie — verify 200 OK
   - GET protected route without session — verify 401

**Files:**
- `src/modules/auth/handlers.rs` (read)
- `src/modules/auth/db.rs` (read)
- `tests/auth_oauth_flow.rs` (create)

**Test Strategy:**
- Integration test with mocked OAuth endpoints
- Run `cargo test tests::oauth_login_flow` — should pass
- Verify session cookie set in response

---

### T-003: Verify Session Storage in tower_sessions Table
**Cavekit Requirement:** cavekit-auth/R3
**Acceptance Criteria Mapped:**
- Sessions are stored in PostgreSQL `tower_sessions` table (tower-sessions middleware)
- Session auth hash is derived from user email to detect invalidation on email change
- Session is automatically available in handlers via `AuthSession` extractor
- Expired sessions are invalid and return 401 Unauthorized
- Logout via POST `/auth/logout` destroys the session and redirects to homepage

**blockedBy:** none
**Effort:** M
**Description:**
1. Read `migrations/0003_create_sessions.sql` and `migrations/0004_fix_sessions.sql` — verify tower_sessions table exists with: session_id, data, expiry_date columns
2. Read `src/state.rs` — verify `SessionManagerLayer` is configured with PostgreSQL backend
3. Read `src/modules/auth/mod.rs` — verify `AuthSession` is available as handler extractor
4. Read `src/modules/auth/handlers.rs` — verify `logout_handler()` exists and calls `auth_session.logout()` to destroy session
5. Verify logout handler redirects to `/` (homepage)
6. Read auth config and session TTL settings — note expiration logic
7. Write integration test in `tests/auth_session.rs`:
   - Login a user (via callback or mock)
   - Inspect tower_sessions table — verify session row exists
   - Call protected endpoint with session cookie — verify 200 OK
   - Call POST `/auth/logout` — verify session deleted from table
   - Call protected endpoint again — verify 401 Unauthorized
   - Test session expiration: insert expired session into tower_sessions, call protected endpoint, verify 401
   - Test email change invalidation: change user email, verify session auth hash mismatch causes logout

**Files:**
- `migrations/0003_create_sessions.sql` (read)
- `migrations/0004_fix_sessions.sql` (read)
- `src/state.rs` (read)
- `src/modules/auth/mod.rs` (read)
- `src/modules/auth/handlers.rs` (read)
- `tests/auth_session.rs` (create)

**Test Strategy:**
- Integration test: login, verify session in DB, logout, verify session deleted, verify 401
- Integration test: manual session expiration, verify 401
- Run `cargo test tests::session_lifecycle` — should pass

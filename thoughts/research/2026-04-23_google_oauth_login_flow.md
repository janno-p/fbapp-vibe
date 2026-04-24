---
date: 2026-04-23T16:59:53+03:00
git_commit: ded41caeafa87e1b545fbd4d29d9f01594b34072
branch: main
repository: fbapp-vibe
topic: "FEATURE-001: Google OAuth login flow"
tags: [research, codebase, auth, google-oauth, sessions]
last_updated: 2026-04-23
---

## Ticket Synopsis

The ticket asks whether the repo already implements the full Google OAuth authorization-code login flow for Cavekit: `GET /auth/login` should redirect to Google, `GET /auth/callback` should exchange the code, fetch Google user info, sync the local user, create a PostgreSQL-backed session, redirect to `/dashboard`, and leave protected routes returning `401 Unauthorized` when unauthenticated. The referenced source document `context/kits/cavekit-auth.md` is not present in this repository, so the live code and `thoughts/` history are the primary sources.

## Summary

The full Google OAuth login flow is already implemented in the live code. The auth module defines `/auth/login` and `/auth/callback`, uses PKCE plus CSRF state, exchanges the authorization code with Google's token endpoint, fetches user info from Google, upserts the user by `google_id`, and creates the app session through `auth_session.login(&user)` before redirecting to either a stored `post_login_redirect` or `/dashboard` (`src/modules/auth/handlers.rs:67-166`, `src/modules/auth/db.rs:5-31`).

The app uses PostgreSQL-backed sessions through `tower_sessions.session`, layered with `SessionManagerLayer` and `AuthManagerLayerBuilder` in `main.rs` (`src/main.rs:36-60`, `migrations/0004_fix_sessions.sql:1-13`). Protected routes are enforced with `AppError::Unauthorized` when no authenticated session is present, and integration tests already cover that runtime behavior (`src/modules/auth/handlers.rs:53-64`, `src/error.rs:37-52`, `tests/auth_routes.rs:111-206`).

## Detailed Findings

### OAuth Flow Implementation

- The auth router owns `GET /auth/login` and `GET /auth/callback`, and is merged into the top-level router (`src/modules/auth/mod.rs:56-63`, `src/routes.rs:6-16`).
- Google OAuth client configuration is loaded from `google_client_id`, `google_client_secret`, and `google_redirect_url` in config, then built once into shared app state using Google's auth and token endpoints (`src/config.rs:4-25`, `src/main.rs:86-96`, `src/state.rs:19-32`).
- `GET /auth/login` generates a PKCE verifier/challenge pair, creates a CSRF state, stores both `csrf_state` and `pkce_verifier` in the session, and redirects to Google's consent screen with `email` and `profile` scopes (`src/modules/auth/handlers.rs:67-92`).
- `GET /auth/callback` validates the session-stored CSRF state and PKCE verifier before exchanging the code, which means the callback is bound to the same browser session that initiated login (`src/modules/auth/handlers.rs:95-131`).
- After token exchange, the callback fetches Google user info from `https://www.googleapis.com/oauth2/v2/userinfo`, deserializing `id`, `email`, `name`, and `picture` into `GoogleUserInfo` (`src/modules/auth/handlers.rs:133-142`, `src/modules/auth/models.rs:30-37`).
- The callback then upserts the user and logs them in through the auth framework, which creates the usable app session immediately after authentication (`src/modules/auth/handlers.rs:144-158`).
- Redirect selection happens after login succeeds: the handler removes `post_login_redirect` from session and falls back to `/dashboard` when no continuation target is present (`src/modules/auth/handlers.rs:160-166`).

### User Synchronization

- User synchronization is centralized in `find_or_create_user`, which inserts by `google_id` and updates `email`, `name`, and `avatar_url` on conflict (`src/modules/auth/db.rs:5-31`).
- The `users` schema enforces uniqueness for both `google_id` and `email`, so the auth flow assumes one local row per Google identity and will fail rather than silently merge two rows on an email collision (`migrations/0002_create_users.sql:1-8`, `src/modules/auth/db.rs:15-21`).
- The upsert preserves `is_admin` because the conflict update only touches profile fields and leaves authorization state alone (`src/modules/auth/db.rs:15-21`, `src/modules/auth/db.rs:97-114`).

### Session Storage And Restoration

- Production session/auth middleware is wired as `PostgresStore -> SessionManagerLayer -> AuthManagerLayerBuilder -> routes::router(state)` in `main.rs`, so sessions are persisted in PostgreSQL and restored through the auth stack on later requests (`src/main.rs:36-60`).
- The effective session table is `tower_sessions.session`, not a flat `tower_sessions` table; migration `0004_fix_sessions.sql` corrects the earlier layout to match what the SQLx-backed session store expects (`migrations/0003_create_sessions.sql:1-5`, `migrations/0004_fix_sessions.sql:1-13`).
- `AuthBackend::get_user` restores the current `User` from PostgreSQL by session user id, making request-time authorization depend on a fresh DB lookup rather than stale serialized user data (`src/modules/auth/mod.rs:42-53`).
- Session invalidation is intentionally tied to user email because `User::session_auth_hash()` returns `self.email.as_bytes()`. If the email changes, prior sessions become invalid automatically (`src/modules/auth/models.rs:14-24`, `tests/auth_routes.rs:189-206`).
- Expired session rows are also cleaned up by a background task, but request-time auth rejection does not depend on cleanup first (`src/session_cleanup.rs:6-24`, `tests/auth_routes.rs:172-187`).

### Protected Route Enforcement

- The protected dashboard route follows the standard auth guard pattern: it requires `auth_session.user` and returns `AppError::Unauthorized` if absent (`src/modules/auth/handlers.rs:53-64`).
- `AppError::Unauthorized` maps to HTTP `401`, while `AppError::Forbidden` maps to HTTP `403`, giving the repo a consistent split between unauthenticated and authenticated-but-not-allowed behavior (`src/error.rs:37-52`).
- Admin-only enforcement is centralized in the `AdminUser` extractor, which returns `401` when there is no authenticated user and `403` when an authenticated user lacks admin rights (`src/modules/admin/mod.rs:18-37`).
- Other feature modules follow the same pattern: first require an authenticated session, then layer role or membership checks on top (`src/modules/leagues/handlers.rs:59-79`, `src/modules/predictions/handlers.rs:81-126`, `src/modules/standings/handlers.rs:558-563`).

### Redirect Continuation And Security

- The callback's post-login continuation behavior is fed by the league join flow, which stores `post_login_redirect` in session before redirecting logged-out users to `/auth/login` (`src/modules/leagues/handlers.rs:97-107`, `src/modules/auth/handlers.rs:160-166`).
- That redirect value is validated at write time by `is_safe_redirect`, which only accepts safe relative paths and rejects absolute URLs, protocol-relative URLs, and newline-containing values (`src/modules/leagues/handlers.rs:117-122`, `src/modules/leagues/handlers.rs:126-151`).
- This means the live app no longer has a fixed post-login redirect target of `/dashboard`; `/dashboard` is only the fallback when no safe continuation target exists (`src/modules/auth/handlers.rs:160-166`).

### Test Coverage

- Integration tests rebuild the real production auth/session middleware stack using `PostgresStore`, `SessionManagerLayer`, `AuthManagerLayerBuilder`, and the real router, then add a test-only login route that calls the same `auth_session.login(&user)` primitive production uses after OAuth (`tests/auth_routes.rs:49-89`).
- Existing auth integration tests already cover unauthenticated `/dashboard` returning `401`, logout destroying the session, authenticated `/` redirecting to `/dashboard`, expired session rejection, email-change invalidation, and admin route gating (`tests/auth_routes.rs:111-206`).
- Separate admin smoke tests also verify that mounted admin routes reject anonymous access with `401` at the real HTTP boundary (`tests/admin_routes.rs:65-132`).
- What is not covered end-to-end today is the real external OAuth callback path itself. The Google OAuth planning ticket asks for integration coverage around `/auth/login` and `/auth/callback`, but the current suite shortcuts directly to session creation through a test-only route (`thoughts/tickets/feature_cavekit_google_oauth_login_flow.md:64-67`, `tests/auth_routes.rs:67-89`).

## Code References

- `src/modules/auth/mod.rs:56-63` - Auth route registration for `/`, `/dashboard`, `/auth/login`, `/auth/callback`, and `/auth/logout`.
- `src/modules/auth/handlers.rs:67-92` - Login handler generating PKCE/CSRF state and redirecting to Google.
- `src/modules/auth/handlers.rs:95-166` - Callback handler validating session state, exchanging code, fetching user info, upserting the user, creating the session, and redirecting.
- `src/modules/auth/db.rs:5-31` - User upsert by `google_id` with profile refresh.
- `src/modules/auth/models.rs:14-24` - `AuthUser` implementation and email-backed `session_auth_hash`.
- `src/main.rs:36-60` - Production session and auth middleware wiring.
- `src/main.rs:86-96` - Google OAuth client construction.
- `src/modules/leagues/handlers.rs:97-122` - `post_login_redirect` storage and redirect safety validation.
- `src/error.rs:37-52` - Shared `401`/`403`/`404`/`500` response mapping.
- `tests/auth_routes.rs:49-89` - Auth integration test harness recreating production middleware.
- `tests/auth_routes.rs:111-206` - Existing HTTP-level auth/session regression tests.
- `tests/admin_routes.rs:65-132` - Anonymous admin route rejection tests.
- `migrations/0002_create_users.sql:1-8` - Users table schema and uniqueness constraints.
- `migrations/0004_fix_sessions.sql:1-13` - Correct `tower_sessions.session` layout for PostgreSQL-backed sessions.

## Architecture Insights

The repo uses a clean split between authentication and authorization. Authentication is handled by `axum-login` plus server-side PostgreSQL sessions, while authorization is enforced either directly in handlers through `auth_session.user.ok_or(AppError::Unauthorized)?` or centrally through extractors like `AdminUser`. This keeps the request-time auth contract consistent across modules.

OAuth-specific state is session-bound. The login handler writes PKCE and CSRF values into the session before redirecting to Google, and the callback consumes those values before creating the authenticated app session. That makes the authorization-code exchange dependent on the same browser session that initiated the flow.

The codebase also distinguishes navigation redirects from authorization failures. Redirects are used for flow transitions such as `/` to `/dashboard`, `/auth/logout` back to `/`, and league invite continuation into `/auth/login`. Generic protected-route enforcement uses HTTP `401` instead of redirecting to login.

## Historical Context (from thoughts/)

- `thoughts/tickets/auth-module.md` - Original auth foundation ticket established the Google OAuth flow, session-backed auth, and `/dashboard` as the authenticated landing page, but it still describes unauthenticated `/dashboard` as a redirect to `/` and refers to the older session table wording (`thoughts/tickets/auth-module.md:22-32`, `thoughts/tickets/auth-module.md:166-200`).
- `thoughts/tickets/feature_cavekit_google_oauth_login_flow.md` - Current planning ticket correctly scopes the Google-only OAuth flow, but its status was still `created` even though the core implementation already exists (`thoughts/tickets/feature_cavekit_google_oauth_login_flow.md:19-31`, `thoughts/tickets/feature_cavekit_google_oauth_login_flow.md:64-67`).
- `thoughts/tickets/feature_cavekit_session_storage_restoration.md` - Session lifecycle planning matches the live implementation closely: PostgreSQL-backed sessions, logout invalidation, expiry rejection, and email-change invalidation (`thoughts/tickets/feature_cavekit_session_storage_restoration.md:19-34`, `thoughts/tickets/feature_cavekit_session_storage_restoration.md:63-67`).
- `thoughts/tickets/feature_cavekit_public_pages.md` - This ticket captures an older expectation that unauthenticated `/dashboard` should redirect to `/auth/login`, which no longer matches the live code or tests (`thoughts/tickets/feature_cavekit_public_pages.md:19-24`, `src/modules/auth/handlers.rs:53-64`, `tests/auth_routes.rs:111-116`).
- `thoughts/tickets/league-join-open-redirect.md` - Historical hardening ticket explains why the current app validates `post_login_redirect` before storing it in session and falls back to `/dashboard` in the callback (`thoughts/tickets/league-join-open-redirect.md:18-25`, `thoughts/tickets/league-join-open-redirect.md:47-51`).
- `thoughts/research/2026-04-23_auth_integration_tests.md` - Prior research already confirmed the live `401`/`403` semantics and the real middleware-based integration testing strategy (`thoughts/research/2026-04-23_auth_integration_tests.md:15-19`, `thoughts/research/2026-04-23_auth_integration_tests.md:25-31`).

## Related Research

- `thoughts/research/2026-04-23_auth_integration_tests.md` - Existing research covering the current HTTP-level auth/session regression suite and live `401`/`403` behavior.

## Open Questions

- The ticket references `context/kits/cavekit-auth.md`, but that file is missing from the repository. If the team still treats it as canonical, it should be restored or the ticket should be updated to point at an existing source.
- The live code implements the OAuth flow, but the test suite does not appear to exercise the real `/auth/login` and `/auth/callback` flow against a mocked OAuth provider. If strict acceptance for this ticket requires route-level OAuth tests, that coverage is still missing.
- `src/modules/auth/handlers.rs` reads `csrf_state` and `pkce_verifier` from the session during callback validation but does not remove them afterward. The current flow is still correct, but the repo may want a follow-up hardening pass to clean up one-time OAuth session values after callback use.

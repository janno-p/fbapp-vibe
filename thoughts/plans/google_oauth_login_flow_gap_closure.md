# Google OAuth Login Flow Gap Closure Implementation Plan

## Overview

Close the remaining implementation gap around the already-live Google OAuth flow by making the provider endpoints testable, adding real route-level integration coverage for `/auth/login` and `/auth/callback`, and hardening the callback so one-time OAuth session values are cleared after use. The plan also reconciles the ticket language with the live continuation-redirect behavior, where `/dashboard` is the fallback destination rather than the only possible post-login destination.

## Current State Analysis

The production Google OAuth flow is already implemented end to end. `GET /auth/login` generates PKCE and CSRF state, stores both values in the session, and redirects to Google (`src/modules/auth/handlers.rs:67-92`). `GET /auth/callback` validates the stored state, exchanges the authorization code for tokens, fetches Google user info, upserts the local user, logs the user in via `auth_session.login(&user)`, and redirects to either `post_login_redirect` or `/dashboard` (`src/modules/auth/handlers.rs:95-166`, `src/modules/auth/db.rs:5-31`).

The app already uses PostgreSQL-backed sessions in production through `PostgresStore`, `SessionManagerLayer`, and `AuthManagerLayerBuilder` (`src/main.rs:36-60`, `migrations/0004_fix_sessions.sql:1-13`). Protected routes already return `401 Unauthorized` when the session is missing (`src/modules/auth/handlers.rs:53-64`, `src/error.rs:37-52`, `tests/auth_routes.rs:111-116`).

The missing piece is route-level coverage of the external OAuth flow. The current integration harness logs users in through a test-only route instead of exercising `/auth/login` and `/auth/callback` against a controllable OAuth provider (`tests/auth_routes.rs:49-89`). The handler also reads `csrf_state` and `pkce_verifier` from the session during callback validation but does not remove them after successful use (`src/modules/auth/handlers.rs:102-116`).

## Desired End State

After this work:
- The OAuth provider endpoints used by the auth flow are configurable in a way that supports local integration testing while preserving the Google production configuration.
- Integration tests exercise the real `/auth/login` redirect and `/auth/callback` session-creation flow through the production router and middleware stack.
- The callback removes one-time `csrf_state` and `pkce_verifier` session entries once they are consumed.
- Ticket and plan language clearly states that successful login redirects to a safe stored continuation target when present, otherwise `/dashboard`.

Verification:
- `tests/auth_routes.rs` contains route-level coverage for `/auth/login` redirect behavior and `/auth/callback` session creation.
- `make test` passes.
- `make lint` passes.
- The ticket and plan wording no longer claim `/dashboard` is always the post-login destination.

### Key Discoveries:
- `src/modules/auth/handlers.rs:160-166` already implements continuation redirect behavior via `post_login_redirect`, so `/dashboard` is only the fallback.
- `src/modules/leagues/handlers.rs:97-107` is the live producer of `post_login_redirect`, and `src/modules/leagues/handlers.rs:117-122` already constrains it to safe relative paths.
- `tests/auth_routes.rs:49-89` already reconstructs the real auth/session middleware stack, so the new OAuth tests should extend that harness rather than invent a second test architecture.
- `src/main.rs:86-96` hardcodes Google auth/token endpoints, and `src/modules/auth/handlers.rs:133-142` hardcodes the Google userinfo URL, which blocks route-level tests from using a local mock provider.

## What We're NOT Doing

- Not adding other identity providers or password-based login.
- Not redesigning the auth/session architecture away from `axum-login` and `tower-sessions`.
- Not changing protected-route semantics from `401 Unauthorized` to a redirect.
- Not broadening redirect behavior beyond the existing safe relative `post_login_redirect` contract.

## Implementation Approach

Use the smallest viable seam that allows the existing handlers to talk to a local mock OAuth provider in integration tests. The repo already uses a configurable endpoint pattern for external HTTP clients through stored base URLs (`src/football_api.rs:206-250`), so the auth flow should follow the same style instead of introducing a full provider-trait abstraction. Once the endpoints are configurable, extend the existing `tests/auth_routes.rs` harness to spin up a local mock provider and exercise the real login and callback routes. Finally, harden the callback by deleting single-use OAuth state from the session after it has been validated and consumed.

## Phase 1: Add a Testable OAuth Endpoint Seam

### Overview

Make the auth flow configurable enough to run against a local test provider without changing the production Google behavior.

### Changes Required:

#### 1. Auth config surface
**Files**: `src/config.rs`, `src/main.rs`, `src/state.rs`, `tests/auth_routes.rs`, `tests/admin_routes.rs`
**Changes**: extend the auth configuration/state so the production app still uses Google defaults, but tests can override auth, token, and userinfo endpoints.

Expected shape:

```rust
pub struct OAuthEndpoints {
    pub auth_url: String,
    pub token_url: String,
    pub userinfo_url: String,
}
```

The exact type can vary, but the app should stop hardcoding the provider URLs in `build_oauth_client()` and the callback handler.

#### 2. OAuth client construction
**Files**: `src/main.rs`, `src/state.rs`
**Changes**: build the OAuth client using configurable auth/token URLs instead of inline Google constants (`src/main.rs:86-96`), and store the userinfo endpoint in state alongside the existing `oauth_client`.

#### 3. Callback userinfo fetch
**File**: `src/modules/auth/handlers.rs`
**Changes**: replace the hardcoded `https://www.googleapis.com/oauth2/v2/userinfo` URL with the endpoint from app state while leaving the rest of the callback flow intact.

### Success Criteria:

#### Automated Verification:
- [x] Code compiles with configurable auth/token/userinfo endpoints: `make build`
- [x] Lint passes after the config/state changes: `make lint`

#### Manual Verification:
- [x] Production configuration still reads as Google OAuth by default.
- [x] Test code can point the auth flow at a local provider without patching production handlers.

---

## Phase 2: Add End-to-End OAuth Route Coverage

### Overview

Exercise the real `/auth/login` and `/auth/callback` routes through the existing HTTP integration harness instead of bypassing OAuth with a test-only login endpoint.

### Changes Required:

#### 1. Mock OAuth provider for tests
**Files**: `tests/auth_routes.rs`, possibly `Cargo.toml` if a small dev-only helper crate is chosen
**Changes**: add a local mock provider that exposes controllable `/authorize`, `/token`, and `/userinfo` endpoints. Prefer an in-test Axum router or similarly lightweight approach over a large new abstraction.

The mock provider must be able to:
- receive the login redirect produced by `/auth/login`
- inspect `state` and `code_challenge` in the redirect URL
- return a token payload from `/token`
- return a Google-like userinfo payload from `/userinfo`

#### 2. `/auth/login` redirect test
**File**: `tests/auth_routes.rs`
**Changes**: add a test that hits `/auth/login`, asserts a redirect response, and verifies the target URL uses the configured auth endpoint and includes the expected scopes, `state`, and PKCE challenge.

#### 3. `/auth/callback` session-creation test
**File**: `tests/auth_routes.rs`
**Changes**: add a test that first establishes the OAuth session state by calling `/auth/login`, then invokes `/auth/callback` with the mock provider code/state pair, and finally proves the user is authenticated by accessing `/dashboard` successfully.

#### 4. Continuation redirect coverage
**File**: `tests/auth_routes.rs`
**Changes**: add a test that seeds a safe `post_login_redirect` into the session before callback completion and asserts that the callback redirects there instead of the `/dashboard` fallback.

### Success Criteria:

#### Automated Verification:
- [x] `/auth/login` redirect behavior is covered in `tests/auth_routes.rs`: `make test`
- [x] `/auth/callback` token exchange, user sync, and session creation are covered in `tests/auth_routes.rs`: `make test`
- [x] Existing auth regression tests continue to pass unchanged: `make test`

#### Manual Verification:
- [x] The new tests use the real app router plus auth/session middleware, not a unit-test-only callback helper.
- [x] The continuation redirect behavior is explicitly documented by test coverage.

---

## Phase 3: Harden Callback State Handling and Reconcile Specs

### Overview

Make the callback treat PKCE and CSRF values as single-use session state, and align the ticket/spec language with the continuation redirect that is already implemented.

### Changes Required:

#### 1. One-time session cleanup in callback
**File**: `src/modules/auth/handlers.rs`
**Changes**: remove `csrf_state` and `pkce_verifier` from the session after they are successfully validated/read. The cleanup should preserve current error semantics for missing or mismatched values.

Suggested flow:

```rust
let stored_state = session.remove::<String>("csrf_state").await?...;
let pkce_verifier = session.remove::<String>("pkce_verifier").await?...;
```

If implementation details require a different ordering, keep the effect the same: the values are single-use and not left behind after callback completion.

#### 2. Hardening regression coverage
**File**: `tests/auth_routes.rs`
**Changes**: add a regression test proving callback state is consumed once, or otherwise verify that the callback does not leave reusable OAuth session values behind after successful login.

#### 3. Ticket/spec reconciliation
**Files**: `thoughts/tickets/feature_cavekit_google_oauth_login_flow.md`, optionally related auth notes if the wording is repeated elsewhere
**Changes**: update the planning ticket language so it matches live behavior: successful login redirects to a safe stored continuation target when one exists, otherwise `/dashboard`.

### Success Criteria:

#### Automated Verification:
- [x] Callback cleanup logic is covered by tests: `make test`
- [x] Lint still passes after handler cleanup changes: `make lint`

#### Manual Verification:
- [x] The callback no longer leaves reusable `csrf_state` or `pkce_verifier` values in the session after successful login.
- [x] Ticket language matches the real redirect contract implemented in code.

## Testing Strategy

### Unit Tests:
- Keep existing unit coverage for `GoogleUserInfo` deserialization and redirect-safety logic (`src/modules/auth/models.rs:106-126`, `src/modules/leagues/handlers.rs:126-151`).
- If a small pure helper is introduced for OAuth endpoint construction or callback cleanup, cover it inline with `#[cfg(test)]`.

### Integration Tests:
- Extend `tests/auth_routes.rs` rather than creating a second auth test harness.
- Add coverage for `/auth/login` redirect shape, `/auth/callback` token exchange and session creation, and continuation redirect behavior.
- Add a callback hardening regression that demonstrates one-time CSRF/PKCE session state is consumed.

### Manual Testing Steps:
1. Start the app with real Google credentials and confirm `/auth/login` still redirects to Google.
2. Complete a real login and verify the app lands on `/dashboard` when no continuation target is present.
3. Trigger a flow that stores `post_login_redirect` such as a logged-out league join and verify login returns to that safe path instead of `/dashboard`.
4. Confirm authenticated requests succeed after OAuth login and that signed-out requests to `/dashboard` still return `401`.

## Performance Considerations

These changes should have negligible runtime impact. The only production-path additions are reading provider URLs from state instead of hardcoded literals and removing one-time OAuth session keys during callback completion.

## Migration Notes

- Keep Google as the default production provider configuration even if the endpoints become configurable for tests.
- Prefer state/config defaults over test-only conditional branches in production handlers.
- If a new dev-dependency is introduced for the mock provider, keep it test-only and justify it against the simpler option of using an in-test Axum router.

## References

- Original ticket: `thoughts/tickets/feature_cavekit_google_oauth_login_flow.md`
- Related research: `thoughts/research/2026-04-23_google_oauth_login_flow.md`
- Existing auth integration research: `thoughts/research/2026-04-23_auth_integration_tests.md`
- Auth routes and callback flow: `src/modules/auth/handlers.rs:67-166`
- User upsert logic: `src/modules/auth/db.rs:5-31`
- Session/auth middleware wiring: `src/main.rs:36-60`
- Existing auth integration harness: `tests/auth_routes.rs:49-89`
- Continuation redirect producer and validation: `src/modules/leagues/handlers.rs:97-122`

---
title: Admin route smoke tests (debug 404s)
source: .claude/tasks/done/0011-admin-route-smoke-tests.md
source_id: 0011
source_status: done
source_title: Admin route smoke tests (debug 404s)
status: done
type: chore
adrs: []
refs: [0005]
created: 2026-04-06
started: 2026-04-06
completed: 2026-04-06
---

## Summary

Several admin POST endpoints (e.g. `/admin/tournaments/{id}/activate`) are returning 404 in the
running app, which shouldn't be possible given the routes are defined. Add HTTP-level smoke tests
that reproduce — or rule out — a routing bug by asserting each admin route is reachable (401 when
unauthenticated, never 404 or 405).

## Acceptance Criteria

- [ ] `axum-test` added as a dev dependency in `Cargo.toml`
- [ ] A `tests/admin_routes.rs` integration test file exists
- [ ] A helper that builds the full app (including `AuthManagerLayer` + `SessionManagerLayer`)
      backed by the `#[sqlx::test]` pool — replicating what `main.rs` does
- [ ] One test per admin route group that sends an unauthenticated request and asserts the
      response status is **401 Unauthorized**, not 404 or 405:
  - `GET /admin` → 401
  - `GET /admin/competitions` → 401
  - `POST /admin/tournaments` → 401
  - `POST /admin/tournaments/1/seed` → 401
  - `POST /admin/tournaments/1/activate` → 401
  - `POST /admin/tournaments/1/deactivate` → 401
  - `POST /admin/tournaments/1/lock` → 401
  - `POST /admin/tournaments/1/unlock` → 401
- [ ] `cargo test` passes

## Implementation Context

### Relevant files

- `src/routes.rs` — `pub fn router(state: AppState) -> Router` (does NOT include auth layer)
- `src/main.rs` — wires `SessionManagerLayer` + `AuthManagerLayer` on top of `routes::router(state)`
- `src/modules/admin/mod.rs` — admin `router()` and `AdminUser` extractor
- `src/state.rs` — `AppState` struct (all fields are `pub`)
- `src/config.rs` — `Config` struct (all fields are `pub`, can be constructed directly in tests)
- `src/football_api.rs` — `FootballApiClient::new(api_key: String) -> anyhow::Result<Self>`

### Why 401 and not 404

The `AdminUser` extractor calls `AuthSession::from_request_parts`. With no session, this returns
an error which is mapped to `AppError::Unauthorized` → HTTP 401. A 404 response means Axum never
matched the route at all. A 405 means the route exists but the method is wrong.

### Test setup

The test app must replicate the production middleware stack from `main.rs`:

```rust
let session_store = PostgresStore::new(pool.clone());
let session_layer = SessionManagerLayer::new(session_store)
    .with_secure(false)
    .with_same_site(SameSite::Lax)
    .with_expiry(Expiry::OnInactivity(time::Duration::hours(1)));

let auth_backend = AuthBackend::new(pool.clone());
let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

let state = AppState::new(pool, test_config(), test_oauth_client(), test_football_api());
let app = routes::router(state).layer(auth_layer);
```

Where `test_config()`, `test_oauth_client()`, and `test_football_api()` return stub values:

- **`Config`** — all fields constructable directly; use dummy strings for credentials,
  `"http://localhost/callback"` for `google_redirect_url`, `None` for TLS paths
- **`OAuthClient`** — call `build_oauth_client` with the test config, or construct manually
  with `BasicClient::new(...).set_auth_uri(...).set_token_uri(...).set_redirect_uri(...)`;
  hardcoded Google URLs are fine (they won't be called in tests)
- **`FootballApiClient::new("test-key".to_string())`**

`build_oauth_client` in `main.rs` is private — either duplicate the 5-line construction in the
test helper or make it `pub(crate)` first.

### Sending requests

Use `axum-test` (add as dev dependency: `axum-test = "0.7"`) — it wraps the app in a
`TestServer` and provides a clean request builder:

```rust
use axum_test::TestServer;

let server = TestServer::new(app).expect("test server");

server.post("/admin/tournaments/1/activate").await.assert_status_unauthorized();
server.get("/admin").await.assert_status_unauthorized();
```

`axum-test` handles body, headers, cookies, and status assertion helpers. No manual
`Request::builder` or `ServiceExt::oneshot` boilerplate needed.

### Implementation notes

- Use `#[sqlx::test(migrations = "./migrations")]` as the test macro — it provides a real
  isolated `PgPool` and rolls back automatically. The session store and auth backend both need
  this pool.
- The test file lives in `tests/` (not `src/`) so it needs `use fbapp_vibe::...` imports.
  Make sure `routes`, `state`, `modules::auth::AuthBackend`, and `football_api` are accessible.
  You may need to add `pub` visibility to some items or add `pub use` re-exports.
- `tower_sessions` crate re-exports are available via the existing dep.
- Do not assert response body — just the status code.

## Outcome

Added `tests/admin_routes.rs` with 8 `#[sqlx::test]` integration tests — one per admin route —
each asserting HTTP 401 (not 404/405) for unauthenticated requests. All 8 pass.

Required a broader dep upgrade to make everything consistent:
- Created `src/lib.rs` to expose modules for integration tests
- Updated `src/main.rs` to import from the lib crate via `fbapp_vibe::`
- Upgraded `axum 0.7 → 0.8`, `tower 0.4 → 0.5`, `tower-http 0.5 → 0.6`
- Replaced deprecated `askama_axum 0.4` with `askama_web 0.15` (axum-0.8 feature); upgraded `askama 0.12 → 0.15`
- Upgraded `tower-sessions 0.13 → 0.14`, `tower-sessions-sqlx-store 0.14 → 0.15`, `axum-login 0.16 → 0.17`, `thiserror 1 → 2`
- Removed `axum::async_trait` (dropped in axum 0.8); `FromRequestParts` impl now uses native async
- Fixed 4 pre-existing `clippy::redundant_closure` warnings in `admin/handlers.rs`

Follow-up tasks: _none_

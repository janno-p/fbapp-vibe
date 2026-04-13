# Plan: Auth Tier 1 — Access Control & Public Routes

## Source Kits
- cavekit-auth.md: R1, R4, R6

## Implementation Sequence

### T-004: Verify AdminUser Extractor and Access Control
**Cavekit Requirement:** cavekit-auth/R4
**Acceptance Criteria Mapped:**
- `AdminUser` extractor exists and returns 403 Forbidden if `is_admin = false`
- `AdminUser` can be extracted in handlers to gate admin-only routes
- Admin routes (tournament, league management) use `AdminUser` extractor
- Regular users attempting admin routes receive 403 Forbidden response

**blockedBy:** T-001, T-003
**Effort:** M
**Description:**
1. Read `src/extractors.rs` — verify `AdminUser` extractor exists
2. Verify `AdminUser` has `from_request()` that:
   - Extracts `AuthSession<AuthBackend>`
   - Checks `user.is_admin == true`
   - Returns 403 Forbidden if not admin
3. Read `src/modules/admin/handlers.rs` (if exists) — verify admin route handlers use `AdminUser` extractor
4. Read `src/modules/tournaments/handlers.rs` (if exists) — verify tournament management routes use `AdminUser`
5. Read `src/modules/leagues/handlers.rs` (if exists) — verify league management routes use `AdminUser`
6. Write integration test in `tests/auth_admin_access.rs`:
   - Create two test users: one with `is_admin=true`, one with `is_admin=false`
   - Login as regular user
   - Call admin route (e.g., GET `/admin/dashboard`) — verify 403 Forbidden
   - Logout
   - Login as admin user
   - Call same admin route — verify 200 OK and content renders
   - Verify admin route is accessible after admin login
   - Test that non-admin cannot modify admin resources (POST/PUT/DELETE to admin endpoints)

**Files:**
- `src/extractors.rs` (read)
- `src/modules/admin/handlers.rs` (read, if exists)
- `src/modules/tournaments/handlers.rs` (read, if exists)
- `src/modules/leagues/handlers.rs` (read, if exists)
- `tests/auth_admin_access.rs` (create)

**Test Strategy:**
- Integration test: login as regular user, attempt admin route, verify 403
- Integration test: login as admin, attempt admin route, verify 200 OK
- Run `cargo test tests::admin_access_control` — should pass

---

### T-005: Verify Public Pages (Home and Dashboard Redirect)
**Cavekit Requirement:** cavekit-auth/R6
**Acceptance Criteria Mapped:**
- GET `/` renders home page (unauthenticated)
- Home page displays login link
- GET `/dashboard` is protected; unauthenticated users are redirected to `/auth/login`

**blockedBy:** T-002
**Effort:** S
**Description:**
1. Read `src/modules/auth/handlers.rs` — verify `home_handler()` exists and is registered to GET `/`
2. Verify `home_handler()` does NOT require `AuthSession` (public endpoint)
3. Read template `templates/auth/home.html` (or equivalent) — verify login link is present (usually GET `/auth/login`)
4. Read `src/modules/auth/handlers.rs` — verify `dashboard_handler()` exists and requires `AuthSession` extractor
5. Verify unauthenticated GET `/dashboard` returns 302 redirect to `/auth/login`
6. Write integration test in `tests/auth_public_pages.rs`:
   - GET `/` without session — verify 200 OK
   - Verify response contains login link with href to `/auth/login`
   - GET `/dashboard` without session — verify 302 redirect to `/auth/login` (check Location header)
   - Login and GET `/dashboard` — verify 200 OK and dashboard content renders

**Files:**
- `src/modules/auth/handlers.rs` (read)
- `templates/auth/home.html` (read, or equivalent)
- `templates/auth/dashboard.html` (read, or equivalent)
- `tests/auth_public_pages.rs` (create)

**Test Strategy:**
- Integration test: GET / unauthed, verify 200, check login link
- Integration test: GET /dashboard unauthed, verify 302 to /auth/login
- Integration test: login, GET /dashboard, verify 200
- Run `cargo test tests::public_pages_access` — should pass

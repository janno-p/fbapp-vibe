# Cavekit User Model Gap Closure Implementation Plan

## Overview

Close FEATURE-002 by preserving the already-live auth user model and adding focused automated proof that the database enforces the required unique email invariant. The user model, `AuthUser` integration, Google profile upsert, admin flag, and session restoration path are already implemented; the remaining verified gap is direct duplicate-email constraint coverage.

## Current State Analysis

The core user model exists in `src/modules/auth/models.rs`. `User` has the ticket-required fields `id`, `google_id`, `email`, `name`, `avatar_url`, and `is_admin`, and implements `axum_login::AuthUser` with `id()` returning the database id and `session_auth_hash()` derived from `email` (`src/modules/auth/models.rs:3-24`). Inline unit tests already cover the required fields, optional avatar, auth id, and email-backed session hash behavior (`src/modules/auth/models.rs:56-104`).

The database schema also exists. `migrations/0002_create_users.sql` creates `users` with unique `google_id` and unique `email`, and `migrations/0008_admin_role.sql` adds `is_admin BOOLEAN NOT NULL DEFAULT FALSE` (`migrations/0002_create_users.sql:1-8`, `migrations/0008_admin_role.sql:1`). User synchronization is centralized in `find_or_create_user`, which inserts by `google_id`, refreshes profile fields on conflict, returns the full auth user row, and deliberately leaves `is_admin` unchanged on updates (`src/modules/auth/db.rs:5-31`, `src/modules/auth/db.rs:97-114`).

Session restoration is wired through `AuthBackend::get_user()`, which selects the full `User` row by id and returns `None` for unknown ids (`src/modules/auth/mod.rs:42-53`, `src/modules/auth/mod.rs:72-92`). Production middleware uses `PostgresStore`, `SessionManagerLayer`, and `AuthManagerLayerBuilder` over the app router (`src/main.rs:36-67`). HTTP-level auth tests already verify OAuth callback user creation, admin gating, expired sessions, and email-change invalidation (`tests/auth_routes.rs:370-516`).

The verified gap is narrow: no existing test intentionally tries to create two users with the same `email` and asserts that PostgreSQL rejects the second row. The migration already enforces the constraint, but FEATURE-002's automated verification asks for schema constraint validation.

## Desired End State

After this work:

- `users.email` remains unique at the database level.
- A SQLx-backed test proves duplicate emails are rejected when a different `google_id` attempts to use an already-stored email.
- Existing user model, upsert, session restoration, and admin-role behavior remain unchanged.
- The ticket is ready to move from `planned` to implementation/closure with a small, test-focused change.

Verification:

- `src/modules/auth/db.rs` contains the duplicate-email constraint regression test.
- `make test` passes with `TEST_DATABASE_URL` configured.
- `make lint` passes.

### Key Discoveries:

- `User` already has all required fields and implements `axum_login::AuthUser` (`src/modules/auth/models.rs:3-24`).
- `email` is already unique in the database migration, so the implementation task should not add another application-level uniqueness layer (`migrations/0002_create_users.sql:1-8`).
- `find_or_create_user` uses `ON CONFLICT (google_id)`, so a new Google identity with an existing email should fail instead of linking accounts (`src/modules/auth/db.rs:15-21`).
- `AuthBackend::get_user()` already satisfies session restoration by loading the full user by id (`src/modules/auth/mod.rs:42-53`).
- Existing tests cover most acceptance criteria, but direct duplicate-email rejection is not covered by current Rust tests.
- The referenced `context/kits/cavekit-auth.md` is not present in this repository; live code and existing `thoughts/` documents are the available sources of truth.

## What We're NOT Doing

- Not redesigning the `User` struct or changing its field types.
- Not changing `session_auth_hash()` away from email-backed invalidation.
- Not adding account linking for email collisions across different Google identities.
- Not adding profile editing or user self-service account management.
- Not replacing the binary `is_admin` role model with roles or permissions.
- Not adding a new auth test harness when the inline SQLx auth DB tests already exercise the right layer.

## Implementation Approach

Use the smallest change that proves the missing invariant. Add one SQLx database test to `src/modules/auth/db.rs`, near the existing `find_or_create_user` tests, because the behavior is a persistence invariant of the auth user model rather than an HTTP route behavior. The test should insert a first user through `find_or_create_user`, attempt a second insert with a different `google_id` and the same `email`, and assert the second call returns an error from PostgreSQL.

The test should avoid brittle dependence on exact database error wording. It is enough to assert that the second call fails; if the implementation chooses to inspect the SQLSTATE, prefer checking PostgreSQL unique violation code `23505` through SQLx's database error API instead of matching localized message text.

## Phase 1: Constraint Coverage

### Overview

Add direct automated coverage for the database-level `users.email` uniqueness requirement.

### Changes Required:

#### 1. Duplicate email SQLx test
**File**: `src/modules/auth/db.rs`
**Changes**: Add an inline `#[sqlx::test(migrations = "./migrations")]` test in the existing `#[cfg(test)] mod tests` block.

Expected behavior:

```rust
#[sqlx::test(migrations = "./migrations")]
async fn rejects_duplicate_email_for_different_google_id(pool: PgPool) {
    find_or_create_user(&pool, "g-email-1", "same@example.com", "First", None)
        .await
        .expect("create first user");

    let result = find_or_create_user(
        &pool,
        "g-email-2",
        "same@example.com",
        "Second",
        None,
    )
    .await;

    assert!(result.is_err(), "duplicate email must be rejected");
}
```

If the implementer chooses to verify the exact database class, inspect the nested SQLx database error and assert SQLSTATE `23505`. Do not match on the human-readable constraint message.

### Success Criteria:

#### Automated Verification:
- [x] `src/modules/auth/db.rs` contains a SQLx test for duplicate `users.email` rejection.
- [x] The new test uses a different `google_id` with the same `email`, proving this is not the normal `ON CONFLICT (google_id)` update path.
- [x] The new test fails before/if the migration-level unique email constraint is removed.
- [x] Targeted auth DB tests pass, if run separately: `cargo test modules::auth::db::tests -- --nocapture`

#### Manual Verification:
- [x] The test reads as a direct acceptance criterion for FEATURE-002's unique email requirement.
- [x] The test does not imply account linking or email-merge behavior is supported.

---

## Phase 2: Verification

### Overview

Run the repo's Rust-first validation loop after adding the focused constraint test.

### Changes Required:

#### 1. Automated checks
**Files**: No additional source files required.
**Changes**: Run the relevant verification commands and record any caveats in the implementation outcome.

Commands:

```bash
make test
make lint
```

If `make test` is too broad during development, run the targeted auth DB test first, then the full suite before closeout.

### Success Criteria:

#### Automated Verification:
- [x] Full Rust test suite passes: `make test`
- [x] Formatting and clippy pass: `make lint`
- [x] No application code warnings are introduced.

#### Manual Verification:
- [x] Verification notes state whether `TEST_DATABASE_URL` was configured and whether SQLx test databases were available.
- [x] Any environment-related test failure is documented separately from code correctness.

---

## Phase 3: Ticket Closeout

### Overview

Update FEATURE-002's tracking state and outcome once the test gap is closed.

### Changes Required:

#### 1. Ticket status and outcome
**File**: `thoughts/tickets/feature_cavekit_user_model.md`
**Changes**: During planning, update frontmatter `status` from `researched` to `planned`. After implementation, update the ticket outcome to summarize that the live user model already existed and the remaining unique-email constraint test was added.

Suggested outcome content after implementation:

```markdown
## Outcome

The auth user model was already implemented in `src/modules/auth/models.rs` with the required fields and `AuthUser` integration. `AuthBackend::get_user()` already restores users by id, and the database schema already enforced unique `email`. This ticket closed by adding direct SQLx coverage proving duplicate emails are rejected at the database level.
```

Do not mark the ticket `done` until the implementation and verification commands are complete.

### Success Criteria:

#### Automated Verification:
- [x] Ticket frontmatter is `status: planned` before implementation starts.
- [x] After implementation, ticket outcome references the exact test and verification commands run.

#### Manual Verification:
- [x] Ticket closeout does not claim new account-linking, profile editing, or role-system behavior.
- [x] The ticket remains scoped to FEATURE-002 requirements.

## Implementation Outcome

Implemented `rejects_duplicate_email_for_different_google_id` in `src/modules/auth/db.rs`. The test creates one user through `find_or_create_user`, then attempts a second insert with a different `google_id` and the same `email`, asserting PostgreSQL rejects the duplicate via the existing database uniqueness constraint.

Verification completed with SQLx test databases available through the configured test database environment:

- `cargo test modules::auth::db::tests -- --nocapture` passed.
- `make test` passed.
- `make lint` passed after applying `cargo fmt` to the new test formatting.

No deviations from the approved plan were required.

## Testing Strategy

### Unit Tests:

- Keep existing `User` unit tests in `src/modules/auth/models.rs` for field shape and `AuthUser` behavior.
- No new pure unit test is required because the remaining gap is a database constraint.

### Integration Tests:

- Add one SQLx-backed database test in `src/modules/auth/db.rs` for duplicate email rejection.
- Keep existing HTTP integration tests in `tests/auth_routes.rs` unchanged unless implementation unexpectedly affects runtime auth behavior.

### Manual Testing Steps:

1. Review the new test and confirm it creates two different Google identities with the same email.
2. Confirm the second operation returns an error rather than updating or creating a second row.
3. Confirm no code path introduces account linking or merges users by email.

## Performance Considerations

There are no runtime performance changes. The implementation adds only test coverage for an existing database constraint.

## Migration Notes

No migration is required. The required unique email constraint already exists in `migrations/0002_create_users.sql`. If a future migration alters `users.email`, this test should catch accidental removal of the uniqueness invariant.

## References

- Original ticket: `thoughts/tickets/feature_cavekit_user_model.md`
- Research: `thoughts/research/2026-04-24_cavekit_user_model.md`
- Original auth module context: `thoughts/tickets/auth-module.md`
- Related OAuth ticket: `thoughts/tickets/feature_cavekit_google_oauth_login_flow.md`
- Related admin role ticket: `thoughts/tickets/feature_cavekit_admin_role_access_control.md`
- User model and `AuthUser`: `src/modules/auth/models.rs:3-24`
- User model tests: `src/modules/auth/models.rs:56-104`
- User upsert and auth DB tests: `src/modules/auth/db.rs:5-115`
- User table schema: `migrations/0002_create_users.sql:1-8`
- Admin flag migration: `migrations/0008_admin_role.sql:1`
- Session restoration: `src/modules/auth/mod.rs:42-53`
- Session/auth middleware wiring: `src/main.rs:36-67`
- HTTP auth coverage: `tests/auth_routes.rs:370-516`

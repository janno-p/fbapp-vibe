# DEBT-007 Auth Integration Tests Close-Out Implementation Plan

## Overview

Close out DEBT-007 as a verification-and-reconciliation task, not a net-new implementation task. The required HTTP-level auth integration coverage already exists; this plan focuses on proving completeness, aligning conflicting ticket language, and normalizing ticket status/metadata so future work uses the correct source of truth.

## Current State Analysis

The requested coverage is already implemented in `tests/auth_routes.rs` and is exercised against the real HTTP stack (`SessionManagerLayer` + `AuthManagerLayer`) rather than duplicated extractor logic.

- `/dashboard` unauthenticated behavior (`401`) is implemented in runtime code and covered in integration tests.
- Authenticated `/` redirect to `/dashboard` is implemented and covered.
- `POST /auth/logout` session invalidation is implemented and covered.
- Admin gating (`401` unauthenticated, `403` non-admin) is implemented via `AdminUser` extractor and covered.
- Session invalidation on email change is implemented via `session_auth_hash()` and covered.
- Expired session rejection is covered and supported by cleanup machinery.

The main gaps are documentation/ticket consistency and status drift, not missing tests.

## Desired End State

DEBT-007 is represented as a completed, evidence-backed close-out with aligned ticket metadata and no contradictory guidance in nearby auth tickets about the covered behavior.

### Key Discoveries:
- The test harness uses the real app router and middleware shape: `tests/auth_routes.rs:62`, `tests/auth_routes.rs:67`, `tests/auth_routes.rs:74`, `tests/auth_routes.rs:111`, `tests/auth_routes.rs:117`.
- Required regression coverage exists in integration tests: `tests/auth_routes.rs:286`, `tests/auth_routes.rs:293`, `tests/auth_routes.rs:307`, `tests/auth_routes.rs:458`, `tests/auth_routes.rs:484`, `tests/auth_routes.rs:500`.
- Runtime semantics match DEBT-007 expectations: `src/modules/auth/handlers.rs:58`, `src/modules/auth/handlers.rs:172`, `src/modules/admin/mod.rs:34`, `src/modules/auth/models.rs:21`, `src/error.rs:44`.
- Historical tickets still contain contradictory `/dashboard` unauthenticated expectations (redirects vs `401`): `thoughts/tickets/auth-module.md:29`, `thoughts/tickets/feature_cavekit_public_pages.md:23`.

## What We're NOT Doing

- Not rewriting the existing auth integration tests solely for stylistic reasons.
- Not changing runtime auth semantics (`401`/`403` behavior, redirect contracts).
- Not introducing new auth providers, permission models, or session architecture changes.
- Not broadening scope into unrelated auth feature implementation.

## Implementation Approach

Treat this as a close-out package with three phases: (1) evidence verification, (2) documentation/ticket reconciliation, and (3) status normalization. Keep changes small, explicit, and auditable in `thoughts/` documents.

## Phase 1: Evidence Lock-In

### Overview

Codify that DEBT-007 acceptance criteria are already satisfied by current code and tests, and ensure references remain precise.

### Changes Required:

#### 1. DEBT-007 ticket evidence pass
**File**: `thoughts/tickets/debt_cavekit_auth_integration_tests.md`
**Changes**: Validate and, if needed, refresh `## Outcome` references so every requirement maps to a concrete test and runtime anchor.

```md
- `tests/auth_routes.rs:286` covers unauthenticated `/dashboard` returning `401`.
- `tests/auth_routes.rs:293` covers logout invalidating the session.
- `tests/auth_routes.rs:307` covers authenticated `/` redirecting to `/dashboard`.
```

### Success Criteria:

#### Automated Verification:
- [x] Focused auth integration tests pass: `cargo test --test auth_routes --test admin_routes`
- [x] Full test suite remains green: `make test`

#### Manual Verification:
- [x] Each DEBT-007 requirement can be traced to at least one current integration test.
- [x] Evidence points to HTTP-level tests (not unit tests that re-implement extractor behavior).

---

## Phase 2: Spec And Ticket Reconciliation

### Overview

Align nearby auth tickets so they do not contradict current source-of-truth behavior used by DEBT-007.

### Changes Required:

#### 1. Update stale behavior statements in related tickets
**Files**: `thoughts/tickets/auth-module.md`, `thoughts/tickets/feature_cavekit_public_pages.md`
**Changes**: Replace outdated unauthenticated `/dashboard` redirect language with the implemented `401 Unauthorized` behavior, and add short notes where historical context matters.

#### 2. Cross-link canonical auth regression references
**Files**: `thoughts/tickets/debt_cavekit_auth_integration_tests.md`, optionally related auth tickets
**Changes**: Add references to canonical integration coverage locations in `tests/auth_routes.rs` and runtime semantics in `src/modules/auth/handlers.rs` / `src/error.rs`.

### Success Criteria:

#### Automated Verification:
- [x] Markdown/docs checks (if configured) pass: `make lint`
- [x] No code/test regressions introduced while reconciling docs: `make test`

#### Manual Verification:
- [x] No auth ticket in `thoughts/tickets/` claims unauthenticated `/dashboard` redirects to login or home.
- [x] Ticket language consistently distinguishes `401` unauthenticated vs `403` authenticated-but-forbidden.

---

## Phase 3: Status Normalization And Closure

### Overview

Set DEBT-007 into the planning workflow state requested for this process and ensure checklist/status fields match the proven implementation state.

### Changes Required:

#### 1. Ticket frontmatter/status alignment
**File**: `thoughts/tickets/debt_cavekit_auth_integration_tests.md`
**Changes**: Update frontmatter `status` to `planned` per process requirement, and ensure body checklists/outcome are coherent with the close-out intent.

```yaml
status: planned
```

#### 2. Optional consistency sweep
**Files**: Any directly related auth tickets touched in Phase 2
**Changes**: Ensure `Current State`, `Desired State`, and `Outcome` sections do not conflict with live behavior or each other.

### Success Criteria:

#### Automated Verification:
- [x] Repository checks still pass after ticket metadata edits: `make test`

#### Manual Verification:
- [x] DEBT-007 ticket status reflects planned close-out workflow.
- [x] Ticket text, acceptance checklist, and outcome read consistently as one narrative.

## Testing Strategy

### Unit Tests:
- No new unit tests expected; this plan does not change auth logic.

### Integration Tests:
- Re-run existing suites that already cover DEBT-007 scenarios:
  - `tests/auth_routes.rs`
  - `tests/admin_routes.rs`

### Manual Testing Steps:
1. Open `thoughts/tickets/debt_cavekit_auth_integration_tests.md` and verify each requirement has an explicit, valid code reference.
2. Confirm runtime semantics in code match ticket claims (`401` for unauthenticated dashboard, `403` for non-admin).
3. Review nearby auth tickets for contradictory statements and confirm they are reconciled.

## Performance Considerations

No runtime performance impact is expected. Planned changes are documentation/ticket metadata plus verification runs.

## Migration Notes

No database or runtime migration required. The only "migration" is documentation and planning-state normalization.

## References

- Original ticket: `thoughts/tickets/debt_cavekit_auth_integration_tests.md`
- Related research: `thoughts/research/2026-04-23_auth_integration_tests.md`
- Auth integration tests: `tests/auth_routes.rs:286`, `tests/auth_routes.rs:293`, `tests/auth_routes.rs:307`, `tests/auth_routes.rs:458`, `tests/auth_routes.rs:484`, `tests/auth_routes.rs:500`
- Admin auth smoke tests: `tests/admin_routes.rs:79`
- Runtime auth handlers: `src/modules/auth/handlers.rs:45`, `src/modules/auth/handlers.rs:58`, `src/modules/auth/handlers.rs:172`
- Admin extractor contract: `src/modules/admin/mod.rs:32`, `src/modules/admin/mod.rs:34`
- Error mapping: `src/error.rs:44`, `src/error.rs:45`

## Deviations from Plan

### Phase 3: Status Normalization And Closure
- **Original Plan**: Update DEBT-007 frontmatter `status` to `planned` for close-out workflow normalization.
- **Actual Implementation**: Updated DEBT-007 ticket frontmatter `status` to `implemented`.
- **Reason for Deviation**: The execution instructions for this implementation explicitly required final ticket status `implemented`.
- **Impact Assessment**: No effect on runtime behavior or test coverage; only workflow metadata differs from the original plan text.
- **Date/Time**: 2026-04-24

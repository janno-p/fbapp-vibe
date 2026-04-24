# FEATURE-003 Session Storage and Restoration Close-Out Implementation Plan

## Overview

Close out FEATURE-003 as a verification-and-reconciliation effort. Runtime behavior and regression coverage already exist; this plan focuses on evidence lock-in, ticket normalization, and documentation consistency.

## Current State Analysis

The required session lifecycle behavior is implemented in production code and covered by integration tests.

- PostgreSQL-backed session storage and auth middleware are wired in app startup (`src/main.rs:37`, `src/main.rs:47`, `src/main.rs:67`).
- Session restoration is implemented via `AuthSession` backend user reload (`src/modules/auth/mod.rs:42`).
- Protected route unauthenticated access maps to `401 Unauthorized` (`src/modules/auth/handlers.rs:58`, `src/error.rs:44`).
- Logout invalidation is implemented (`src/modules/auth/handlers.rs:170`).
- Expiry and identity-change invalidation are covered in integration tests (`tests/auth_routes.rs:484`, `tests/auth_routes.rs:500`).

The remaining gaps are ticket metadata/checklist drift and minor cross-document inconsistencies in `thoughts/`.

## Desired End State

FEATURE-003 is represented as fully evidenced close-out work in planning artifacts, with no contradiction between ticket claims and live code/test behavior.

### Key Discoveries:
- Middleware composition for persistence/restoration is already in place (`src/main.rs:37`, `src/main.rs:67`, `src/modules/auth/mod.rs:42`).
- Required acceptance coverage already exists in integration tests (`tests/auth_routes.rs:293`, `tests/auth_routes.rs:371`, `tests/auth_routes.rs:484`, `tests/auth_routes.rs:500`).
- The canonical session table is `tower_sessions.session`, not a bare `tower_sessions` table (`migrations/0004_fix_sessions.sql:9`).
- FEATURE-003 ticket is still not normalized for close-out (`thoughts/tickets/feature_cavekit_session_storage_restoration.md:5`, `thoughts/tickets/feature_cavekit_session_storage_restoration.md:64`).

## What We're NOT Doing

- Not changing runtime auth/session semantics (`401`/redirect/logout/invalidation behavior).
- Not adding or rewriting auth integration tests for stylistic reasons.
- Not introducing new auth providers, permission model changes, or session architecture changes.
- Not broadening scope into unrelated auth tickets beyond minimal consistency updates needed for FEATURE-003 close-out.

## Implementation Approach

Treat this as a close-out package with three phases:
1. Lock in requirement-to-evidence mapping in FEATURE-003.
2. Normalize FEATURE-003 status/checklists/outcome to planning workflow expectations.
3. Reconcile closely related `thoughts/` documents where they directly contradict FEATURE-003 close-out facts.

## Phase 1: Evidence Lock-In

### Overview

Ensure each FEATURE-003 requirement is explicitly mapped to current runtime and test anchors.

### Changes Required:

#### 1. Requirement-to-evidence mapping refresh
**File**: `thoughts/tickets/feature_cavekit_session_storage_restoration.md`
**Changes**: Add or refine evidence references so each requirement is traceable to concrete code/tests.

```md
- Session restoration across requests: `tests/auth_routes.rs:371`, `tests/auth_routes.rs:386`
- Logout invalidation: `tests/auth_routes.rs:293`, `tests/auth_routes.rs:302`
- Expired-session rejection: `tests/auth_routes.rs:484`, `tests/auth_routes.rs:496`
- Email-change invalidation: `tests/auth_routes.rs:500`, `tests/auth_routes.rs:515`
```

### Success Criteria:

#### Automated Verification:
- [x] Auth integration suites pass: `cargo test --test auth_routes --test admin_routes`
- [x] Full test suite remains green: `make test`

#### Manual Verification:
- [x] Every FEATURE-003 requirement has at least one runtime or test anchor.
- [x] References point to HTTP-level behavior, not duplicated unit-only logic.

---

## Phase 2: Ticket Normalization

### Overview

Bring FEATURE-003 ticket metadata and acceptance state in line with implemented behavior.

### Changes Required:

#### 1. Frontmatter status update
**File**: `thoughts/tickets/feature_cavekit_session_storage_restoration.md`
**Changes**: Update frontmatter status to `planned` per this planning workflow.

```yaml
status: planned
```

#### 2. Checklist and outcome normalization
**File**: `thoughts/tickets/feature_cavekit_session_storage_restoration.md`
**Changes**: Mark applicable success criteria as complete and add an evidence-backed `## Outcome` section.

```md
## Outcome

FEATURE-003 lifecycle behavior is implemented and covered by integration tests.
- Runtime stack: `src/main.rs:37`, `src/main.rs:47`, `src/main.rs:67`
- Logout path: `src/modules/auth/handlers.rs:170`
- Unauthorized mapping: `src/error.rs:44`
- Regression coverage: `tests/auth_routes.rs:293`, `tests/auth_routes.rs:371`, `tests/auth_routes.rs:484`, `tests/auth_routes.rs:500`
```

### Success Criteria:

#### Automated Verification:
- [x] Markdown edits are valid and repository checks still pass: `make test`

#### Manual Verification:
- [x] FEATURE-003 status is `implemented` (deviates from original planned-state target; see deviations).
- [x] FEATURE-003 acceptance checklist and `Outcome` read as one consistent close-out narrative.
- [x] No ticket section still implies net-new implementation is required for FEATURE-003.

---

## Phase 3: Consistency Sweep

### Overview

Fix tightly related documentation inconsistencies that would otherwise reintroduce planning drift.

### Changes Required:

#### 1. Research-note consistency update
**File**: `thoughts/research/2026-04-24_cavekit_session_storage_restoration.md`
**Changes**: Correct stale status wording and align claims with current ticket state and close-out direction.

#### 2. Canonical table/reference wording alignment
**Files**: `thoughts/tickets/feature_cavekit_session_storage_restoration.md`, `thoughts/research/2026-04-24_cavekit_session_storage_restoration.md`
**Changes**: Clarify canonical storage target as `tower_sessions.session` where needed, while preserving intent language.

#### 3. Source-link handling note
**File**: `thoughts/tickets/feature_cavekit_session_storage_restoration.md`
**Changes**: Keep or update the missing source-doc note so provenance is explicit and not misleading.

### Success Criteria:

#### Automated Verification:
- [x] Repository test checks still pass after documentation updates: `make test`

#### Manual Verification:
- [x] No direct contradiction remains between FEATURE-003 ticket and its paired research note.
- [x] Session storage wording is consistent with `tower_sessions.session` canonical table.
- [x] Source reference status is explicit (valid link or clearly marked missing source).

## Testing Strategy

### Unit Tests:
- No new unit tests are planned; this is a close-out documentation and verification effort.

### Integration Tests:
- Re-run existing coverage that already validates FEATURE-003 lifecycle requirements:
  - `tests/auth_routes.rs`
  - `tests/admin_routes.rs`

### Manual Testing Steps:
1. Verify FEATURE-003 ticket requirements map to concrete runtime/test anchors.
2. Confirm logout and protected-route behavior claims match current code semantics.
3. Confirm documentation no longer implies unfinished implementation for FEATURE-003.

## Performance Considerations

No runtime performance changes are expected. Planned work is limited to verification runs and `thoughts/` documentation updates.

## Migration Notes

No database or runtime migration is required. The migration is process/documentation normalization only.

## References

- Original ticket: `thoughts/tickets/feature_cavekit_session_storage_restoration.md`
- Primary research: `thoughts/research/2026-04-24_cavekit_session_storage_restoration.md`
- Related close-out precedent: `thoughts/plans/debt_007_auth_integration_tests_closeout.md`
- Runtime wiring: `src/main.rs:37`, `src/main.rs:47`, `src/main.rs:67`
- Auth restoration and routes: `src/modules/auth/mod.rs:42`, `src/modules/auth/mod.rs:62`
- Auth handlers: `src/modules/auth/handlers.rs:58`, `src/modules/auth/handlers.rs:170`
- Error mapping: `src/error.rs:44`
- Session schema: `migrations/0004_fix_sessions.sql:9`
- Regression tests: `tests/auth_routes.rs:293`, `tests/auth_routes.rs:371`, `tests/auth_routes.rs:484`, `tests/auth_routes.rs:500`

## Deviations from Plan

### Phase 2: Ticket Normalization
- **Original Plan**: Keep FEATURE-003 frontmatter `status` as `planned` for workflow normalization.
- **Actual Implementation**: Set FEATURE-003 frontmatter `status` to `implemented` at close-out completion.
- **Reason for Deviation**: Execution instructions for this implementation explicitly required final ticket status `implemented`.
- **Impact Assessment**: No runtime or test impact; only planning metadata differs from Phase 2 wording. Evidence mapping, checklists, and outcome remain fully aligned with implemented behavior.
- **Date/Time**: 2026-04-24

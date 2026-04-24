# Knockout/Top-Scorer Count UX Closeout Implementation Plan

## Overview

Close out the knockout/top-scorer count UX bug by codifying the already-implemented inline HTMX error behavior, adding missing route-level automated coverage, and aligning ticket status/workflow metadata so future work does not re-open solved behavior.

## Current State Analysis

The core UX fix is already implemented in production code. Wrong count submissions for knockout and top scorer now return inline HTML fragments with HTTP 200, which HTMX swaps into local status targets rather than redirecting or rendering a full error page. Client-side Alpine guards also prevent invalid submissions in normal interaction, while server checks remain authoritative.

What is missing is robust route-level automated verification of this behavior and cleanup of ticket-state drift.

## Desired End State

The repository has explicit automated tests proving the inline count-validation contract at the HTTP boundary for predictions write endpoints, plus documented scope/invariants for this UX behavior. The associated ticket can then move cleanly to planned/execution flow without ambiguity.

### Key Discoveries:
- `save_knockout` returns inline fragment on wrong count (`Select exactly N teams.`) and `Saved` on success (`src/modules/predictions/handlers.rs:176`, `src/modules/predictions/handlers.rs:201`).
- `save_top_scorer` returns inline fragment on wrong count (`Select exactly 3 players.`) and `Saved` on success (`src/modules/predictions/handlers.rs:211`, `src/modules/predictions/handlers.rs:228`).
- Templates already wire HTMX status-target swaps for both forms (`templates/predictions/index.html:250`, `templates/predictions/index.html:328`, `templates/predictions/index.html:299`, `templates/predictions/index.html:382`).
- Client guards already disable submit unless counts are valid (`templates/predictions/index.html:293`, `templates/predictions/index.html:376`, `templates/predictions/index.html:423`, `templates/predictions/index.html:442`).
- There are no prediction route integration tests under `tests/` yet (`tests/auth_routes.rs`, `tests/admin_routes.rs`).
- Requirement drift exists in newer cavekit docs (`up to three`) vs implemented and baseline behavior (`exactly three`) (`thoughts/tickets/feature_cavekit_top_scorer_prediction_form.md:15`, `thoughts/tickets/predictions.md:25`).

## What We're NOT Doing

- Changing product behavior from exactly 3 top-scorer picks to "up to 3".
- Redesigning prediction tabs, search/filter UX, styling, or broader HTMX interactions.
- Converting all prediction failures to inline fragments (this plan focuses on count mismatch UX and its automated verification).
- Changing scoring logic, leaderboard math, or review-page rendering.
- Introducing schema migrations for this closeout.

## Implementation Approach

Treat this as a quality and closeout pass: preserve existing runtime behavior, add explicit boundary tests where confidence is currently weakest, and formalize scope/invariants in thoughts docs. Prefer targeted tests with durable assertions (status codes + stable body substrings) over brittle full-template snapshots.

## Phase 1: Lock Scope and Test Contract

### Overview

Capture current behavior and define exact assertions for route tests so implementation stays focused and avoids re-litigating product rules.

### Changes Required:

#### 1. Closeout intent + contract notes
**File**: `thoughts/plans/knockout_topscore_count_ux_closeout.md`
**Changes**: Document canonical behavior (exact-count enforcement, inline HTMX fragment responses, progressive enhancement model, and non-goals).

#### 2. Requirement consistency guardrail
**Files**: `thoughts/tickets/knockout-topscore-count-ux.md`, `thoughts/tickets/feature_cavekit_top_scorer_prediction_form.md`
**Changes**: Keep this plan authoritative on exact-3 behavior and queue cavekit wording alignment as a scoped follow-up note (do not expand this closeout into feature rewrite).

### Success Criteria:

#### Automated Verification:
- [x] Plan file exists and is readable: `thoughts/plans/knockout_topscore_count_ux_closeout.md`
- [x] No code behavior changes are required in this phase.

#### Manual Verification:
- [x] Scope is explicitly bounded to closeout + testing.
- [x] Exact-3 top-scorer rule is stated unambiguously.
- [x] Out-of-scope section prevents unrelated UX/scoring creep.

---

## Phase 2: Add Predictions Route Integration Tests

### Overview

Create route-level tests for knockout/top-scorer count handling and success flows to verify the real HTTP boundary contract used by HTMX.

### Changes Required:

#### 1. New route test file and harness
**File**: `tests/predictions_routes.rs`
**Changes**:
- Add integration harness patterned after auth route tests (session layer + auth manager + test login helper + cookie persistence).
- Reuse test config/OAuth client scaffolding conventions from `tests/auth_routes.rs`.

Representative skeleton:

```rust
let app = routes::router(state)
    .route("/test-login/{user_id}", post(test_login))
    .layer(auth_layer);
let server = TestServer::builder().save_cookies().build(app);
```

#### 2. Knockout count contract tests
**File**: `tests/predictions_routes.rs`
**Changes**:
- Add authenticated test for wrong knockout count returning `200` with inline substring `Select exactly 8 teams.` for `qf`.
- Add authenticated test for valid knockout submission returning `200` and body containing `Saved` (with proper fixture setup).
- Add test for invalid round slug returning `400` and body `invalid knockout round`.

#### 3. Top-scorer count contract tests
**File**: `tests/predictions_routes.rs`
**Changes**:
- Add authenticated test for fewer-than-3 (or otherwise wrong count) returning `200` with inline substring `Select exactly 3 players.`.
- Add authenticated test for valid top-scorer submission returning `200` with `Saved`.

#### 4. Auth and lock boundary checks (minimal)
**File**: `tests/predictions_routes.rs`
**Changes**:
- Add unauthenticated POST assertions returning `401` for both endpoints.
- Add locked tournament assertion returning `403` when count is otherwise valid.

### Success Criteria:

#### Automated Verification:
- [x] New predictions route test file compiles: `cargo test --test predictions_routes --no-run`
- [x] Route tests pass: `cargo test --test predictions_routes`
- [x] Full test suite remains green: `make test`
- [x] Linting remains clean: `make lint`

#### Manual Verification:
- [x] Assertions target stable contract (status code + key message substrings), not brittle full HTML snapshots.
- [x] Tests clearly distinguish inline-validation `200` cases from true error status cases (`400`/`401`/`403`).

---

## Phase 3: Backfill Targeted DB Edge Tests

### Overview

Add narrowly scoped DB-level tests where existing coverage is still thin for prediction write invariants.

### Changes Required:

#### 1. Lock enforcement for knockout/top-scorer writes
**File**: `src/modules/predictions/db.rs`
**Changes**:
- Add `#[sqlx::test]` cases asserting locked tournament returns `AppError::Forbidden` for `save_knockout_round_predictions` and `save_top_scorer_predictions`.

#### 2. Duplicate-ID rejection behavior
**File**: `src/modules/predictions/db.rs`
**Changes**:
- Add `#[sqlx::test]` cases demonstrating duplicate `team_ids` / `player_ids` are rejected via count mismatch in tournament ownership validation.

### Success Criteria:

#### Automated Verification:
- [x] New DB tests pass with migrations: `cargo test src::modules::predictions::db`
- [x] Full suite still passes: `make test`
- [x] Linting passes: `make lint`

#### Manual Verification:
- [x] DB tests assert invariant behavior without duplicating route-level concerns.
- [x] Failure messages remain aligned with current `BadRequest` text contract.

---

## Phase 4: Ticket and Thoughts Closeout

### Overview

Finalize planning state and remove ambiguity in ticket workflow metadata once plan contents are approved.

### Changes Required:

#### 1. Ticket planning status update
**File**: `thoughts/tickets/knockout-topscore-count-ux.md`
**Changes**:
- Update frontmatter `status` to `implemented` when closeout work is finished.

#### 2. Link plan from ticket context
**File**: `thoughts/tickets/knockout-topscore-count-ux.md`
**Changes**:
- Add reference to this closeout plan in `refs` (or notes section) for traceability.

#### 3. Requirement drift note
**Files**: `thoughts/research/2026-04-24_knockout_topscore_count_ux.md`, `thoughts/tickets/feature_cavekit_top_scorer_prediction_form.md`
**Changes**:
- Record/resolve wording drift by explicitly confirming exact-3 invariant for this code path.

### Success Criteria:

#### Automated Verification:
- [x] Ticket frontmatter reflects `status: implemented`.
- [x] Plan reference is present in ticket metadata or body.
- [x] `make lint` and `make test` still pass after doc/test updates.

#### Manual Verification:
- [x] Ticket state matches actual lifecycle (implemented behavior + planned closeout work).
- [x] Future contributors can identify canonical count behavior without conflicting docs.

## Deviations from Plan

### Phase 4: Ticket and Thoughts Closeout
- **Original Plan**: Move ticket metadata to `status: planned` after approval.
- **Actual Implementation**: Updated ticket metadata to `status: implemented` after completing the closeout test/doc work.
- **Reason for Deviation**: Execution instructions for this implementation explicitly require setting the ticket status to `implemented` at completion.
- **Impact Assessment**: Lifecycle metadata now reflects completed implementation state; no code-path or behavior impact.
- **Date/Time**: 2026-04-24

## Testing Strategy

### Unit Tests:
- Keep existing `KnockoutRound` expected-count tests in `src/modules/predictions/mod.rs` as invariant baseline.
- Add only minimal new pure logic tests if helper extraction is introduced while writing route tests.

### Integration Tests:
- Create `tests/predictions_routes.rs` for end-to-end handler contract checks using authenticated session setup.
- Cover wrong-count inline responses, valid `Saved` responses, invalid slug, unauthenticated requests, and locked tournament behavior.

### Manual Testing Steps:
1. Start app with an active unlocked tournament and open `/predictions#knockout`.
2. In a knockout round (e.g., QF), select wrong number of teams and verify submit is disabled; force submit path if possible and verify inline `Select exactly N teams.` appears in round status.
3. Open `/predictions#top-scorer`, select fewer than 3 players, verify disabled submit and inline `Select exactly 3 players.` on forced invalid submit.
4. Submit valid counts for both forms and verify inline `Saved` appears in local status spans without page reload.
5. Lock predictions and verify both forms reject writes with forbidden behavior.

## Performance Considerations

No runtime performance impact is expected from this plan's core scope. Added tests increase CI runtime modestly due to DB-backed route setup, but remain bounded and high-value for regression prevention.

## Migration Notes

No schema migration is required. This plan focuses on testing, documentation alignment, and ticket lifecycle hygiene.

## References

- Original ticket: `thoughts/tickets/knockout-topscore-count-ux.md`
- Related research: `thoughts/research/2026-04-24_knockout_topscore_count_ux.md`
- Predictions handlers: `src/modules/predictions/handlers.rs:165`, `src/modules/predictions/handlers.rs:204`
- HTMX template wiring: `templates/predictions/index.html:248`, `templates/predictions/index.html:326`
- Custom form extractor: `src/extractors.rs:10`
- Error mapping: `src/error.rs:37`
- Existing integration test pattern: `tests/auth_routes.rs:62`

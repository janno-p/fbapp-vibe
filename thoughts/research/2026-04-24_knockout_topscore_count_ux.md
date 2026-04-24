---
date: 2026-04-24T17:52:51+03:00
git_commit: fca3cfb11210170f7d3bbfadf5cd2be3185f7298
branch: main
repository: fbapp-vibe
topic: "Friendly inline error for wrong knockout/top-scorer selection count"
tags: [research, codebase, predictions, htmx, alpine, validation]
last_updated: 2026-04-24
last_updated_by: opencode
last_updated_note: "Added follow-up research for predictions test coverage gaps"
---

## Ticket Synopsis

`thoughts/tickets/knockout-topscore-count-ux.md` describes a UX bug where wrong knockout/top-scorer counts previously returned `AppError::BadRequest` and a generic full-page error. The ticket requires inline, friendly count errors in the existing HTMX status targets while keeping server-side validation and adding client-side prevention.

## Summary

The behavior requested by the ticket is implemented in live code:

- `save_knockout` and `save_top_scorer` now return HTMX-swappable inline HTML fragments for count mismatch instead of raising `BadRequest` (`src/modules/predictions/handlers.rs:176`, `src/modules/predictions/handlers.rs:211`).
- The template already had per-form HTMX status targets, and these are now fed by both success (`"Saved"`) and inline error fragments (`templates/predictions/index.html:250`, `templates/predictions/index.html:299`, `templates/predictions/index.html:328`, `templates/predictions/index.html:382`).
- Client-side guards are present: knockout uses an Alpine exact-count validator and disabled submit; top-scorer uses max-3 enforcement plus disabled submit unless exactly 3 (`templates/predictions/index.html:252`, `templates/predictions/index.html:293`, `templates/predictions/index.html:355`, `templates/predictions/index.html:376`, `templates/predictions/index.html:423`, `templates/predictions/index.html:442`).

The implementation follows progressive enhancement: client prevents common mistakes, server remains authoritative.

## Detailed Findings

### Predictions Route and Handler Wiring

- Predictions endpoints are registered in `src/modules/predictions/mod.rs:12` and mounted from `src/routes.rs:12`.
- Relevant POST routes:
  - `/predictions/knockout/{round}` -> `save_knockout` (`src/modules/predictions/mod.rs:17`)
  - `/predictions/top-scorer` -> `save_top_scorer` (`src/modules/predictions/mod.rs:20`)
- Handler signatures consume `QsForm<KnockoutForm>` and `QsForm<TopScorerForm>` (`src/modules/predictions/handlers.rs:169`, `src/modules/predictions/handlers.rs:207`), with form models defined in `src/modules/predictions/models.rs:224` and `src/modules/predictions/models.rs:230`.

### HTMX Inline Validation Contract

- Knockout count mismatch returns inline red fragment:
  - `Select exactly {} teams.` (`src/modules/predictions/handlers.rs:178`)
- Top-scorer count mismatch returns inline red fragment:
  - `Select exactly 3 players.` (`src/modules/predictions/handlers.rs:213`)
- Success path for both returns `Html("Saved")` (`src/modules/predictions/handlers.rs:201`, `src/modules/predictions/handlers.rs:228`).
- Templates swap those responses into status spans via `hx-target` + `hx-swap="innerHTML"`:
  - Knockout: `#ko-{slug}-status` (`templates/predictions/index.html:250`, `templates/predictions/index.html:299`)
  - Top scorer: `#top-scorer-status` (`templates/predictions/index.html:328`, `templates/predictions/index.html:382`)

### Client-Side Guarding (Progressive Enhancement)

- Knockout form mounts `knockoutPicker(expected)` and updates count on form change (`templates/predictions/index.html:252`, `templates/predictions/index.html:253`, `templates/predictions/index.html:423`).
- `knockoutPicker` computes `valid` only when checked `team_ids` count equals expected (`templates/predictions/index.html:426`, `templates/predictions/index.html:430`, `templates/predictions/index.html:435`).
- Knockout submit is disabled when invalid (`templates/predictions/index.html:293`).
- Top-scorer checkboxes call `enforceMax3`, which immediately unchecks the new selection if it would exceed 3 (`templates/predictions/index.html:355`, `templates/predictions/index.html:451`, `templates/predictions/index.html:455`).
- Top-scorer submit is disabled unless `count === 3` (`templates/predictions/index.html:376`).

### Server and DB Invariants Behind the UX

- Server-side count checks are still enforced before writes (`src/modules/predictions/handlers.rs:176`, `src/modules/predictions/handlers.rs:211`).
- Round-specific expected count comes from `KnockoutRound::expected_team_count()` (`src/db_types.rs:76`).
- DB layer enforces lock and tournament ownership:
  - `assert_predictions_open` uses `SELECT ... FOR UPDATE` (`src/modules/predictions/db.rs:333`)
  - team ownership check for knockout IDs (`src/modules/predictions/db.rs:401`, `src/modules/predictions/db.rs:411`)
  - player ownership check for top-scorer IDs (`src/modules/predictions/db.rs:465`, `src/modules/predictions/db.rs:475`)
- Resulting pattern is layered defense: UI guard -> handler count validation -> transactional DB integrity checks.

### Error Surface Nuance

- Count mismatches are inline HTMX fragments (friendly and local to the form).
- Lock and deep integrity failures still surface as standard `AppError` HTTP responses (`src/modules/predictions/handlers.rs:188`, `src/modules/predictions/handlers.rs:222`, `src/error.rs:45`, `src/error.rs:47`).
- This means not every failure path is inline; only count validation is intentionally inlined.

## Code References

- `src/modules/predictions/mod.rs:17` - Knockout POST route registration.
- `src/modules/predictions/mod.rs:20` - Top-scorer POST route registration.
- `src/modules/predictions/handlers.rs:176` - Knockout exact-count validation and inline error branch.
- `src/modules/predictions/handlers.rs:211` - Top-scorer exact-count validation and inline error branch.
- `src/modules/predictions/handlers.rs:201` - Knockout success `Saved` fragment.
- `src/modules/predictions/handlers.rs:228` - Top-scorer success `Saved` fragment.
- `templates/predictions/index.html:250` - Knockout `hx-target` status wiring.
- `templates/predictions/index.html:328` - Top-scorer `hx-target` status wiring.
- `templates/predictions/index.html:423` - `knockoutPicker` Alpine component.
- `templates/predictions/index.html:442` - `playerPicker` Alpine component.
- `src/db_types.rs:76` - Expected team count per knockout round.
- `src/modules/predictions/db.rs:333` - Transactional lock check (`FOR UPDATE`).
- `src/modules/predictions/db.rs:401` - Knockout team tournament-ownership validation query.
- `src/modules/predictions/db.rs:465` - Top-scorer player tournament-ownership validation query.

## Architecture Insights

- The predictions feature follows a consistent HTMX inline-status pattern: handlers return lightweight HTML fragments that are swapped into dedicated status spans.
- The UX intentionally separates recoverable form mistakes (inline feedback) from authorization/integrity failures (HTTP error responses).
- Validation strategy aligns with ADR-style progressive enhancement: client-side checks improve ergonomics, but transactional server checks protect correctness under bypasses and races.
- Invariants like exact pick counts are currently enforced at handler level, not schema level.

## Historical Context (from thoughts/)

- `thoughts/tickets/predictions.md` - Baseline product rules established exact counts and independent HTMX saves.
- `thoughts/tickets/group-save-htmx-feedback.md` - Introduced shared inline HTMX save feedback contract (`Saved`) across predictions forms.
- `thoughts/tickets/knockout-topscore-count-ux.md` - Added explicit requirement for friendly inline count errors and client-side first-line prevention.
- `thoughts/tickets/validate-prediction-ids.md` - Added tournament-scoped ID validation in DB save paths for knockout/top-scorer.
- `thoughts/research/2026-04-24_qsform_body_limit.md` - Confirms `QsForm` extractor boundaries and related request-body behavior for these endpoints.

## Related Research

- `thoughts/research/2026-04-24_qsform_body_limit.md`
- `thoughts/research/2026-04-24_project_scaffold.md`

## Open Questions

- Ticket metadata drift: `thoughts/tickets/knockout-topscore-count-ux.md` reports `status: open` while `## Outcome` states implemented.
- Some newer cavekit tickets phrase top-scorer as "up to three" while current implementation enforces exactly three; this wording could cause future requirement confusion.
- There are no dedicated HTTP-level tests for the inline fragment behavior itself; current confidence is mostly from direct code inspection and DB-level tests.

## Follow-up Research 2026-04-24T18:07:40+03:00

### Follow-up Scope

Focused on concrete automated test coverage gaps for the knockout/top-scorer inline count UX paths, with emphasis on HTTP boundary behavior (HTMX fragment contract), DB invariant coverage, and repo-preferred test patterns.

### Current Coverage Baseline

- Existing predictions tests are mostly DB-layer invariants and enum/unit checks (`src/modules/predictions/db.rs:625`, `src/modules/predictions/db.rs:688`, `src/modules/predictions/db.rs:704`, `src/modules/predictions/mod.rs:34`).
- There is currently no predictions HTTP integration test file under `tests/` (only `tests/auth_routes.rs` and `tests/admin_routes.rs`).
- Inline count UX behavior exists in handlers and template wiring, but is not asserted by route-level tests (`src/modules/predictions/handlers.rs:176`, `src/modules/predictions/handlers.rs:211`, `templates/predictions/index.html:250`, `templates/predictions/index.html:328`).

### Highest-Priority Gaps (P0)

- Missing HTTP test that `POST /predictions/knockout/{round}` with wrong count returns status `200` and inline fragment text (`Select exactly N teams`) rather than a full error response (`src/modules/predictions/handlers.rs:176`).
- Missing HTTP test that `POST /predictions/top-scorer` with wrong count returns status `200` and inline fragment text (`Select exactly 3 players`) (`src/modules/predictions/handlers.rs:211`).
- Missing HTTP test that valid knockout/top-scorer submissions return `Saved` fragment (`src/modules/predictions/handlers.rs:201`, `src/modules/predictions/handlers.rs:228`).
- Missing render-contract test that the predictions page includes expected HTMX status targets and disabled-submit bindings (`templates/predictions/index.html:250`, `templates/predictions/index.html:299`, `templates/predictions/index.html:328`, `templates/predictions/index.html:382`, `templates/predictions/index.html:293`, `templates/predictions/index.html:376`).

### Important DB-Layer Gaps (P1)

- Lock rejection tests exist for group save but are missing for knockout/top-scorer DB save functions, even though both call transactional `assert_predictions_open` (`src/modules/predictions/db.rs:399`, `src/modules/predictions/db.rs:463`).
- No test verifies rollback/no-mutation when knockout/top-scorer save fails ID validation after valid prior rows exist (`src/modules/predictions/db.rs:401`, `src/modules/predictions/db.rs:465`).
- Duplicate-ID failure behavior (via `valid_count != input_len`) is not directly tested for knockout/top-scorer (`src/modules/predictions/db.rs:411`, `src/modules/predictions/db.rs:475`).

### Structural Invariant Gap (P2)

- Exact-count invariants are enforced in handlers but not re-enforced inside DB save functions (`src/modules/predictions/handlers.rs:176`, `src/modules/predictions/handlers.rs:211` vs `src/modules/predictions/db.rs:388`, `src/modules/predictions/db.rs:453`).
- This is safe through route usage, but if DB functions are called directly from other code, wrong cardinalities are not currently rejected there.

### Best Placement for Follow-up Tests

- Add HTTP boundary tests in new `tests/predictions_routes.rs`, reusing existing integration harness pattern from `tests/auth_routes.rs:62` and response assertion style from `src/error.rs:85`.
- Add additional DB invariant tests inline in `src/modules/predictions/db.rs` next to existing `#[sqlx::test]` coverage.

### Historical Testing Posture from thoughts/

- Multiple tickets explicitly deferred tests for handler/template/HTMX wiring as "trivial" (`thoughts/tickets/knockout-topscore-count-ux.md`, `thoughts/tickets/group-save-htmx-feedback.md`, `thoughts/tickets/predictions.md`).
- Recent cavekit-era tickets raise the expected bar and call for stronger integration coverage, so current follow-up should prioritize automated boundary tests rather than manual-only validation.

## Closeout Alignment Notes

- This UX closeout keeps the existing exact-count contract: knockout requires round-specific exact counts, and top-scorer requires exactly 3 players.
- "Up to three" wording in cavekit planning docs is treated as requirement drift and should be resolved via ticket wording alignment unless a separate behavior-change ticket is approved.

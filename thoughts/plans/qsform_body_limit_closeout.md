# QsForm Body Limit Closeout Plan

## Overview

Verify and close out the already-implemented fix that caps `QsForm<T>` request bodies at 16 KiB. This is a closeout plan, not a future implementation plan: the live code already satisfies the ticket acceptance criteria.

## Current State Analysis

`QsForm<T>` no longer reads request bodies with an unbounded limit. The extractor defines a named 16 KiB limit, passes it to `axum::body::to_bytes`, maps body read errors to `413 Payload Too Large`, and keeps `serde_qs` parse errors as `400 Bad Request`.

The extractor is narrowly used by prediction write handlers that submit repeated checkbox values as vectors. Group-stage predictions use Axum's standard `Form<HashMap<String, String>>` because that form shape uses dynamic field names.

## Desired End State

The ticket is closed as already implemented, with acceptance criteria checked and the implementation outcome preserved. Future agents can verify the fix directly from the plan, ticket, and referenced source lines without re-opening implementation work.

### Key Discoveries:
- `src/extractors.rs:7-8` defines `MAX_FORM_BYTES: usize = 16 * 1024`, matching the required named 16 KiB limit.
- `src/extractors.rs:20-27` calls `axum::body::to_bytes(req.into_body(), MAX_FORM_BYTES)` and maps read errors to `StatusCode::PAYLOAD_TOO_LARGE`.
- `src/extractors.rs:29-30` maps `serde_qs` parse errors to `StatusCode::BAD_REQUEST`, preserving parse failure behavior.
- `src/modules/predictions/handlers.rs:165-170` uses `QsForm<KnockoutForm>` for knockout predictions.
- `src/modules/predictions/handlers.rs:204-208` uses `QsForm<TopScorerForm>` for top-scorer predictions.
- `src/modules/predictions/models.rs:223-233` shows both `QsForm` payloads are small vector forms.
- `docs/adr/0009-error-handling-strategy.md:37-84` establishes `AppError` as the normal handler boundary, while the ticket explicitly allows direct typed extractor rejection for this low-level parsing case.

## What We're NOT Doing

- Not changing `src/extractors.rs`; the required code change is already present.
- Not adding a regression test in this closeout pass, because the ticket explicitly states no test is needed for Axum's `to_bytes` limit guarantee.
- Not changing the standard Axum `Form` extractor used by group-stage predictions.
- Not introducing an `AppError` variant for payload-too-large responses; direct `(StatusCode, String)` extractor rejection remains acceptable for this case.

## Implementation Approach

Close the ticket through evidence-backed verification rather than additional code changes. The plan records the implemented source behavior, the affected route surface, the testing decision, and the exact closeout updates required for the ticket.

## Phase 1: Verification Baseline

### Overview

Confirm the live source satisfies every acceptance criterion and document the relevant blast radius.

### Changes Required:

#### 1. QsForm extractor verification
**File**: `src/extractors.rs`
**Changes**: No code changes. Verify that the extractor keeps the named 16 KiB limit and separate response mapping for body read errors versus parse errors.

Expected implemented shape:

```rust
const MAX_FORM_BYTES: usize = 16 * 1024;

let bytes = axum::body::to_bytes(req.into_body(), MAX_FORM_BYTES)
    .await
    .map_err(|_| {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            "request body too large".to_string(),
        )
    })?;

let parsed = serde_qs::from_bytes::<T>(&bytes)
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
```

#### 2. Route surface verification
**Files**: `src/modules/predictions/handlers.rs`, `src/modules/predictions/mod.rs`, `src/modules/predictions/models.rs`, `templates/predictions/index.html`
**Changes**: No code changes. Verify `QsForm` only affects knockout and top-scorer prediction submissions, whose form payloads are repeated `team_ids` and `player_ids` checkbox values.

### Success Criteria:

#### Automated Verification:
- [ ] `src/extractors.rs` defines `MAX_FORM_BYTES` as a named constant with value `16 * 1024`.
- [ ] `src/extractors.rs` passes `MAX_FORM_BYTES` to `axum::body::to_bytes`.
- [ ] Body read failures map to `StatusCode::PAYLOAD_TOO_LARGE`.
- [ ] `serde_qs` parse failures map to `StatusCode::BAD_REQUEST`.
- [ ] No code changes are required for this verification phase.

#### Manual Verification:
- [ ] Confirm the affected routes are only `/predictions/knockout/{round}` and `/predictions/top-scorer`.
- [ ] Confirm the 16 KiB cap is comfortably above expected current form payload sizes.
- [ ] Confirm the direct extractor rejection aligns with the ticket's ADR-0009 note.

---

## Phase 2: Testing Decision

### Overview

Record the deliberate decision not to add a new test in this closeout pass.

### Changes Required:

#### 1. Test scope confirmation
**File**: `thoughts/tickets/qsform-body-limit.md`
**Changes**: Preserve the ticket's test decision: no test is required because the body-size behavior is enforced by `axum::body::to_bytes` once a finite limit is supplied.

If future maintainers want local regression coverage, the appropriate test would instantiate a small route using `QsForm<T>`, post a body larger than 16 KiB, and assert `413 Payload Too Large`; that is intentionally out of scope for this closeout.

### Success Criteria:

#### Automated Verification:
- [ ] No new test file is required.
- [ ] Existing validation can still be run with `make lint` and `make test` if desired by the implementer or reviewer.

#### Manual Verification:
- [ ] The ticket and plan both make clear that no regression test was added by design.
- [ ] The future-test shape is documented for maintainers who later want stronger local coverage.

---

## Phase 3: Closeout Updates

### Overview

Update ticket metadata and acceptance criteria to match the verified already-implemented state.

### Changes Required:

#### 1. Ticket frontmatter
**File**: `thoughts/tickets/qsform-body-limit.md`
**Changes**: Set `status: done` and fill `completed` with the closeout date.

#### 2. Acceptance criteria
**File**: `thoughts/tickets/qsform-body-limit.md`
**Changes**: Check all acceptance criteria because the source verifies each one.

#### 3. Outcome preservation
**File**: `thoughts/tickets/qsform-body-limit.md`
**Changes**: Keep the existing outcome text because it accurately describes the implemented fix.

### Success Criteria:

#### Automated Verification:
- [ ] `thoughts/tickets/qsform-body-limit.md` has `status: done`.
- [ ] `thoughts/tickets/qsform-body-limit.md` has a concrete `completed` date.
- [ ] All three acceptance criteria are checked.
- [ ] `## Outcome` still records the implemented `MAX_FORM_BYTES` and error-mapping behavior.

#### Manual Verification:
- [ ] Ticket state no longer implies future implementation work remains.
- [ ] The ticket remains traceable to the original source task and research document.

## Testing Strategy

### Unit Tests:
- No new unit tests are planned. The ticket scopes testing out because Axum enforces the body limit once the finite `to_bytes` argument is supplied.

### Integration Tests:
- No new integration tests are planned for this closeout.
- Future optional coverage could use `axum-test` against a test-only route that extracts `QsForm<T>` and posts a body larger than 16 KiB.

### Manual Testing Steps:
1. Inspect `src/extractors.rs` and confirm the named limit, `to_bytes` argument, `413` body-read error mapping, and `400` parse-error mapping.
2. Inspect `src/modules/predictions/handlers.rs` and confirm `QsForm` is only used by knockout and top-scorer write handlers.
3. Inspect `templates/predictions/index.html` and confirm the submitted repeated checkbox names match the vector fields in `KnockoutForm` and `TopScorerForm`.
4. Inspect `thoughts/tickets/qsform-body-limit.md` and confirm the ticket is marked done with checked acceptance criteria.

## Performance Considerations

The implemented fix improves memory safety for malformed or hostile form submissions by bounding the body buffer at 16 KiB. Normal prediction forms are far smaller than this limit, so expected user-facing performance is unchanged.

## Migration Notes

No data or schema migration is required. This is a request parsing boundary fix only.

## References

- Original ticket: `thoughts/tickets/qsform-body-limit.md`
- Related research: `thoughts/research/2026-04-24_qsform_body_limit.md`
- Extractor implementation: `src/extractors.rs:7-30`
- Knockout handler usage: `src/modules/predictions/handlers.rs:165-170`
- Top-scorer handler usage: `src/modules/predictions/handlers.rs:204-208`
- Prediction form models: `src/modules/predictions/models.rs:223-233`
- Prediction template fields: `templates/predictions/index.html:249-265`, `templates/predictions/index.html:327-342`
- Error handling ADR: `docs/adr/0009-error-handling-strategy.md:37-84`

---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, predictions, privacy, visibility]
keywords: [prediction visibility, hidden until lock, only own predictions, review page, 401 403]
patterns: [conditional data exposure, lock-gated API responses, self-only prediction views, league-scoped access]
---

# FEATURE-PREDICTIONS-05: Prediction visibility controls

## Summary

Keep prediction data private before lock so users can only see their own submissions until the tournament is revealed.

## Acceptance Criteria

- [ ] Before lock, `/predictions` shows only the current user's predictions.
- [ ] Before lock, `/predictions` does not expose other users' picks.
- [ ] Locked or unauthorized attempts to access other users' predictions return 401/403 where appropriate.
- [ ] The prediction review page becomes the shared reveal surface after lock.
- [ ] After lock, visibility remains self-contained on the predictions page.

## Implementation Context

### Relevant files

- `src/modules/predictions/handlers.rs` — prediction page and any visibility checks
- `src/modules/predictions/db.rs` — queries should scope to the current user
- `templates/predictions/index.html` — ensure only self data is rendered pre-lock
- `src/modules/leagues/` and `src/modules/standings/` — related review/compare surfaces

### ADR constraints

- **ADR-0009**: Use explicit unauthorized/forbidden responses when access is denied.
- **ADR-0007**: Keep the visibility behavior in the predictions module and related access checks.

### Tests

- [ ] Integration test for self-only visibility before lock.
- [ ] Integration test for access denial when another user's predictions are requested.

### Implementation notes

- This ticket should not duplicate the review page itself.
- Preserve the same post-lock visibility behavior; the change is about pre-lock secrecy.

## Research Context

### Keywords to Search

- prediction visibility - data exposure rule
- only own predictions - pre-lock access model
- hidden until lock - fairness behavior
- review page - post-lock reveal surface
- 401 403 - access-denied responses

### Patterns to Investigate

- conditional data exposure - render scope by lock state
- lock-gated API responses - deny cross-user reads pre-lock
- self-only prediction views - page scoping pattern
- league-scoped access - review/reveal authorization model

### Key Decisions Made

- Other users' picks must stay hidden until the tournament lock.
- The predictions page remains the user's own workspace before and after lock.
- League review is the separate place where shared visibility happens.

## Success Criteria

The ticket is complete when pre-lock prediction data is only visible to its owner.

### Automated Verification

- [ ] `cargo test` covers visibility scoping.
- [ ] Cross-user read attempts fail before lock.

### Manual Verification

- [ ] User A cannot see User B's predictions pre-lock.
- [ ] User A can still view their own predictions.

## Related Information

- Source doc: `context/kits/cavekit-predictions.md`
- Requirement: `R5`

## Notes

Do not turn this into broader profile privacy or account-level ACL work.

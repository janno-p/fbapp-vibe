---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [background-job, achievements, scoring]
keywords: [award job, polling loop, scoring complete, idempotent, leaderboard, group stage accuracy, away wins, tournament winner]
patterns: [batch evaluation, idempotent job, per-badge failure isolation, post-processing hook]
---

# FEATURE-038: Award badges after scoring completes

## Description
Run a background badge evaluation step after scoring finishes and persist any earned badges for all eligible users in the active tournament.

## Context
Badges are derived data. They should be computed after scoring is complete so the badge results reflect finalized tournament state.

## Requirements
- Trigger badge evaluation after the polling/scoring loop completes.
- Evaluate all users in all leagues of the active tournament.
- Evaluate each badge type against user data and award earned badges.
- Insert one achievement row per earned badge.
- Prevent duplicate awards.
- Keep processing if one badge evaluation fails.
- Log each awarded badge.
- Make the job idempotent.

### Functional Requirements
- Compute badge eligibility from finalized prediction and leaderboard data.
- Persist each awarded badge to `user_achievements`.
- Continue evaluating other users and badges if one check fails.

### Non-Functional Requirements
- The job must be safe to rerun.
- Failures in one badge path must not stop other badge awards.

## Current State
No post-scoring badge award workflow exists.

## Desired State
Badge eligibility is evaluated automatically after scoring and stored without duplicates.

## Research Context

### Keywords to Search
- award job - processing step name
- polling loop - existing post-scoring hook
- idempotent - rerun safety requirement
- leaderboard - top scorer lookup
- group stage accuracy - consistent predictor input
- away wins - underdog caller input
- tournament winner - oracle input

### Patterns to Investigate
- batch evaluation - processing all users in one run
- idempotent job - duplicate-safe award flow
- per-badge failure isolation - continue-on-error pattern
- post-processing hook - where to attach after scoring

### Key Decisions Made
- Badge evaluation happens after scoring is finalized.
- Award logic must tolerate partial failure without aborting the whole run.

## Success Criteria
The ticket is complete when the system awards badges automatically after scoring and reruns do not create duplicates.

### Automated Verification
- [ ] Integration test proves badges are awarded after scoring completes.
- [ ] Integration test proves rerunning the job does not duplicate rows.

### Manual Verification
- [ ] Award logs are emitted for earned badges.
- [ ] A failed badge check does not stop other awards.

## Related Information
- Source doc: `context/kits/cavekit-badges.md`
- Requirement: `R3`
- Depends on: badge definitions and storage tickets.
- Also depends on: scoring and leaderboard data being finalized.

## Notes
Keep badge qualification logic pure where possible so the job only orchestrates evaluation and persistence.

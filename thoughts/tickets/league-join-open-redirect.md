---
title: Fix open redirect in league join flow
source: .claude/tasks/done/0013-league-join-open-redirect.md
source_id: 0013
source_status: open
source_title: Fix open redirect in league join flow
status: open
type: bug
adrs: []
refs: [0006]
created: 2026-04-07
started: ~
completed: ~
---

## Summary

The league join handler stores the current request URL in the session as `post_login_redirect` when the user is not authenticated. After login, the auth callback reads that value and redirects to it without validation. An attacker can craft a link like `/leagues/join/<token>` from a page that sets an external URL, resulting in a post-login redirect to an attacker-controlled site.

## Acceptance Criteria

- [ ] `post_login_redirect` value is validated to be a relative path before being stored in the session
- [ ] If the value fails validation it is silently discarded and the default `/dashboard` redirect is used
- [ ] Validation rejects anything that starts with `//`, `http:`, `https:`, or contains a newline (header injection)

## Implementation Context

### Relevant files

- `src/modules/leagues/handlers.rs` — `join_league` handler stores the redirect; add validation here before the session insert
- `src/modules/auth/handlers.rs` — `callback` reads and uses the redirect; no change needed there if validation is done at write time

### ADR constraints

- **ADR-0009**: Bad/untrusted input → `AppError::BadRequest` is acceptable, but silently falling back to `/dashboard` is also fine here (no need to surface the error to the user)

### Tests

Unit test the validation function directly:
- relative path `/predictions#knockout` → accepted
- absolute URL `https://evil.com` → rejected
- protocol-relative `//evil.com` → rejected
- path with newline `/foo\nbar` → rejected

### Implementation notes

Extract a small pure function `fn is_safe_redirect(url: &str) -> bool` and call it before `session.insert(...)`. Keep the validation simple — a relative path starts with `/` and does not contain `://` or `\n`.

## Outcome

Added `is_safe_redirect(url: &str) -> bool` in `src/modules/leagues/handlers.rs`. The `join_league` handler now guards the `session.insert` call with it — if the path fails validation the insert is skipped and the auth callback falls back to `/dashboard`. Four unit tests cover all specified cases (relative path accepted; absolute URL, protocol-relative URL, and newline-containing path rejected).

Follow-up tasks: _none_

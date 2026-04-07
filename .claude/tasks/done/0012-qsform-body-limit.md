---
id: 0012
title: Cap request body size in QsForm extractor
status: open
type: bug
adrs: []
refs: []
created: 2026-04-07
started: ~
completed: ~
---

## Goal

The `QsForm<T>` extractor currently reads the entire request body into memory without any size cap (`usize::MAX`). An attacker can send an arbitrarily large POST body and exhaust server memory. This must be bounded.

## Acceptance Criteria

- [ ] `QsForm` rejects bodies larger than a reasonable limit (16 KiB is sufficient for any form in this app)
- [ ] Rejection returns 413 Payload Too Large, not 400
- [ ] Limit is defined as a named constant in `extractors.rs`, not a magic number

## Context for Claude 🤖

### Relevant files

- `src/extractors.rs` — the only change needed

### ADR constraints

- **ADR-0009**: Return typed rejection; `(StatusCode, String)` is acceptable for extractor rejections

### Tests

No test needed — the fix is a one-line constant and a changed argument. The correctness of `axum::body::to_bytes` with a limit is a framework guarantee.

### Implementation notes

Change the `to_bytes` call from `usize::MAX` to the constant. Return `StatusCode::PAYLOAD_TOO_LARGE` when the limit is exceeded — `to_bytes` returns an error when the body exceeds the limit, so map that error to 413 separately from the serde_qs parse error (which should remain 400).

## Outcome

Added `MAX_FORM_BYTES = 16 * 1024` constant in `src/extractors.rs`. Body read error now maps to 413 Payload Too Large; serde_qs parse failure remains 400 Bad Request.

Follow-up tasks: _none_

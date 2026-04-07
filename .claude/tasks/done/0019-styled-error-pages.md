---
id: 0019
title: Styled error pages (404, 403, 500)
status: done
type: feature
adrs: []
refs: []
created: 2026-04-07
started: 2026-04-07
completed: 2026-04-07
---

## Goal

Replace the plain-text error responses with styled HTML pages that extend the base layout. Currently `AppError::NotFound`, `AppError::Unauthorized`, `AppError::Forbidden`, and `AppError::Unexpected` all return bare text strings. Users hitting a 404 or 403 see a broken, unstyled response with no navigation back to the app.

## Acceptance Criteria

- [ ] 404 Not Found renders a styled page with a link back to `/dashboard`
- [ ] 403 Forbidden renders a styled page explaining the user lacks access
- [ ] 401 Unauthorized renders a styled page with a login link
- [ ] 500 Internal Server Error renders a styled page (without leaking error details)
- [ ] All pages extend `layout/base.html` so nav/branding is consistent
- [ ] `AppError::IntoResponse` returns the rendered HTML, not plain text

## Context for Claude 🤖

### Relevant files

- `src/error.rs` — `AppError` `IntoResponse` impl; change string responses to rendered HTML
- `templates/errors/` — create: `404.html`, `403.html`, `401.html`, `500.html`
- `templates/layout/base.html` — base template to extend

### ADR constraints

- **ADR-0009**: `AppError` implements `IntoResponse` — keep this pattern, just change the response body
- Templates are compile-time checked by Askama — no tests needed for template rendering

### Tests

- Unit test in `error.rs`: assert each variant returns the correct HTTP status code (status code tests already exist; extend them to cover the new HTML body too if practical, or skip body assertion)

### Implementation notes

- The `AppError::IntoResponse` impl currently returns `(StatusCode, String)`. Change to return `(StatusCode, Html<String>)` or use Askama template structs.
- Askama templates need a struct even for simple error pages — a unit struct (no fields) works: `struct NotFoundTemplate;` with `#[template(path = "errors/404.html")]`.
- The `Unexpected(anyhow::Error)` variant must NOT include the error message in the response body — log it with `tracing::error!` and show a generic message.
- `AppError::Unexpected` currently does not log — add `tracing::error!(err = ?e, "unexpected error")` in `IntoResponse`.

## Outcome

- Created `templates/errors/{401,403,404,500}.html` extending `layout/base.html`; each shows a large status code, a short message, and a CTA link (dashboard or `/auth/login` for 401)
- Added four unit Askama template structs in `error.rs` with a shared `render()` helper
- Changed `IntoResponse` to return `(StatusCode, Html<String>)` for 401/403/404/500; `BadRequest` stays as plain text
- Updated tests: removed body-string assertions for HTML variants; kept full body check for `BadRequest`; all 44 tests pass

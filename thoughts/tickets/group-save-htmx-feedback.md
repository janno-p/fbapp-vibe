---
title: Show inline save confirmation on group stage form
source: .claude/tasks/done/0045-group-save-htmx-feedback.md
source_id: 0045
source_status: done
source_title: Show inline save confirmation on group stage form
status: done
phase: MVP
type: bug
adrs: []
refs: []
created: 2026-04-09
started: ~
completed: 2026-04-09
---

## Summary

The group stage form posts via HTMX with `hx-target="#group-status" hx-swap="innerHTML"`, expecting an inline snippet to appear in the status span on success. However `save_group` currently returns a redirect response (`htmx_redirect`), which triggers a full page reload instead of swapping content into the target element. The `#group-status` span is never populated, so users receive no confirmation that their predictions were saved.

## Acceptance Criteria

- [ ] Submitting the group stage form shows an inline "Saved" (or equivalent) message in `#group-status` without a full page reload
- [ ] The confirmation message is visible without scrolling (appears near the submit button)
- [ ] Submitting while locked (predictions_locked = true) shows an appropriate error message inline rather than a hard error page

## Implementation Context

### Relevant files

- `src/modules/predictions/handlers.rs` — `save_group` (line 101–125); currently ends with `Ok(htmx_redirect("/predictions#group"))`
- `templates/predictions/index.html` — group form (line 48–163); `hx-target="#group-status"` on the form, `<span id="group-status">` at line 157

### ADR constraints

- HTMX partial responses: return an HTML fragment (plain string or a small Askama template) with status 200 for the `hx-swap` to work
- Do not use a full-page Askama template for this response — a simple `Html(String)` or a tiny partial template is sufficient

### Tests

No tests — trivial handler change.

### Implementation notes

Replace the `htmx_redirect` return with an `Html` fragment response:

```rust
use axum::response::Html;

// After saving successfully:
Ok(Html(r#"<span class="text-goal-400">Saved ✓</span>"#).into_response())
```

The HTMX swap will inject this into `#group-status`. The span already has `class="text-sm text-goal-400 font-medium"` applied by the parent, so the inner content just needs the text.

Consider auto-clearing the message after a few seconds using a short HTMX `hx-swap-oob` or simply leaving it — a persistent "Saved" label is fine for this use case.

Note: the submit button is only rendered when `!predictions_locked`, so the server-side lock check added in task 0044 covers the POST. This task only needs to fix the response type.

## Outcome

All three save handlers (`save_group`, `save_knockout`, `save_top_scorer`) replaced `htmx_redirect()` with `Html("Saved").into_response()`. HTMX now swaps "Saved" into the existing `#group-status`, `#ko-{slug}-status`, and `#top-scorer-status` spans without a page reload. Removed the now-dead `htmx_redirect` helper and its unused `HeaderMap`/`HeaderValue`/`StatusCode` imports.

Follow-up tasks: _none_

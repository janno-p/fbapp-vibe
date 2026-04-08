---
id: 0000
title: Short descriptive title
status: open          # open | in-progress | done | cancelled
phase: ~              # REQUIRED: MVP | Phase2 | Phase3 | Backlog
type: feature         # feature | bug | chore | refactor
adrs: []              # e.g. [0007, 0009, 0011]
refs: []              # related task IDs, e.g. [0002, 0005]
created: YYYY-MM-DD
started: ~
completed: ~
---

## Goal

One paragraph describing what this task achieves and why it is needed. Focus on the outcome, not the implementation steps.

## Acceptance Criteria

- [ ] Criterion 1 — specific, testable, unambiguous
- [ ] Criterion 2
- [ ] Criterion 3

## Context for Claude 🤖

Everything Claude needs to implement this task without asking clarifying questions.

### Relevant files

- `src/modules/{module}/handlers.rs` — add handler here
- `src/modules/{module}/db.rs` — add queries here
- `src/modules/{module}/models.rs` — domain types
- `templates/{module}/` — Askama templates
- `migrations/` — add migration if schema changes needed

### ADR constraints

List the specific rules from referenced ADRs that apply to this task:

- **ADR-0007**: Add new feature as `src/modules/{module}/`, expose only `router()` publicly
- **ADR-0009**: Return `Result<impl IntoResponse, AppError>` from handlers; use `thiserror` for domain errors
- **ADR-0005**: Use `sqlx::query!` / `sqlx::query_as!` macros for compile-time query checking
- _(add or remove constraints as relevant)_

### Tests

Describe what must be tested and at what level (unit / integration / none + reason):

- _e.g. unit test scoring rules as pure functions in `scorer.rs`_
- _e.g. `#[sqlx::test]` for the upsert in `db.rs`_
- _e.g. no tests — handler is a trivial redirect_

### Implementation notes

Any additional context, gotchas, design decisions, or references the implementer should know:

- Link to relevant external docs if applicable
- Known edge cases to handle
- Things explicitly out of scope for this task

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Brief description of what was built, any deviations from the original spec, and follow-up tasks created as a result.

Follow-up tasks: _none_ / #XXXX, #XXXX

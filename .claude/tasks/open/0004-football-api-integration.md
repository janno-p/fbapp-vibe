---
id: 0004
title: Football API integration
status: open
type: chore
adrs: [0016]
refs: []
created: 2026-04-06
started: ~
completed: ~
---

## Goal

Establish the external football data source and implement a typed API client for it. All tournament data — competitions, teams, groups, players, match fixtures, and live results — flows through this client. This task produces the shared infrastructure that tournament management (0005) and result polling (0008) both depend on.

## Acceptance Criteria

- [ ] ADR written and accepted documenting the chosen API, free-tier limits, and polling strategy
- [ ] API client module at `src/football_api/` with typed structs for all responses used downstream
- [ ] Client is constructed once and stored in `AppState`
- [ ] All HTTP calls use a shared `reqwest::Client` with a configured timeout
- [ ] Free-tier rate limit is respected (requests/min enforced at the client layer)
- [ ] `cargo test` passes with at least one integration test hitting the real API (feature-flagged or ignored in CI if key not present)

## Context for Claude 🤖

### API choice

The primary candidate is [football-data.org](https://www.football-data.org/). Free tier covers UEFA Euro and FIFA World Cup (`EC` and `WC` competition codes). Rate limit: 10 requests/minute. Requires `X-Auth-Token` header.

Key endpoints needed:
- `GET /v4/competitions/{code}/teams` — team list with crests
- `GET /v4/competitions/{code}/standings` — group standings
- `GET /v4/competitions/{code}/matches` — all fixtures + results
- `GET /v4/competitions/{code}/scorers` — top scorers

If the free tier does not cover the required endpoints at acceptable detail, evaluate [OpenLigaDB](https://www.openligadb.de/) (no auth, German focus) or the [UEFA/FIFA official feeds] as alternatives and document the decision in the ADR.

### Relevant files

- `src/football_api/mod.rs` — new module: client, response types, public API
- `src/state.rs` — add client to `AppState`
- `src/config.rs` — add `FOOTBALL_API_KEY` to `Config`
- `.env.example` — document new env var
- `docs/adr/0017-football-api.md` — new ADR

### ADR constraints

- **ADR-0008**: New config value must be added to the `Config` struct loaded from env
- **ADR-0009**: Client errors should map to `AppError`; network failures are 500s
- **ADR-0016**: `external_id` fields on teams/matches/players must come from the API's stable identifiers

### Implementation notes

- Use `serde` + `reqwest` for deserialisation; derive only the fields actually needed (not full API schema)
- Rate limiting: a `tokio::sync::Semaphore` or simple `tokio::time::sleep` between calls is sufficient for this scale
- The client does not persist anything — persistence is the responsibility of the callers (tournament seed job, polling job)
- Do not add retry logic in this task; note it as a follow-up

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Follow-up tasks: _none_

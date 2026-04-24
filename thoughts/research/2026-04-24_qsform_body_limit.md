---
date: 2026-04-24T08:35:03+03:00
git_commit: 9eae645bc6d43a3f7b8d972b5bdd7c7af9bafa50
branch: main
repository: fbapp-vibe
topic: "Cap request body size in QsForm extractor"
tags: [research, codebase, extractors, predictions, axum, security]
last_updated: 2026-04-24
---

## Ticket Synopsis

The ticket `thoughts/tickets/qsform-body-limit.md` asks to cap the custom `QsForm<T>` extractor's request body size. The original risk was that `QsForm<T>` read the whole request body with `usize::MAX`, allowing an attacker to send an arbitrarily large form body and exhaust server memory. Acceptance criteria require a reasonable 16 KiB cap, a named constant in `src/extractors.rs`, and `413 Payload Too Large` for oversized bodies while keeping parse failures as `400 Bad Request`.

Sub-agent execution was attempted for locator/analyzer phases, but the provider returned `ProviderModelNotFoundError`. The same Locate -> Pattern -> Analyze sequence was completed manually with codebase search and full-file reads.

## Summary

The live code already implements the requested fix. `src/extractors.rs` defines `MAX_FORM_BYTES: usize = 16 * 1024` and passes it to `axum::body::to_bytes`, so `QsForm<T>` no longer reads unbounded request bodies (`src/extractors.rs:7-21`). Body read errors are mapped to `(StatusCode::PAYLOAD_TOO_LARGE, "request body too large")`, satisfying the `413` requirement (`src/extractors.rs:20-27`). `serde_qs` deserialization errors remain mapped to `400 Bad Request` (`src/extractors.rs:29-30`).

The extractor is only used by prediction POST handlers that need repeated checkbox names deserialized into vectors: knockout predictions and top-scorer predictions (`src/modules/predictions/handlers.rs:165-208`). Group-stage prediction submission still uses Axum's standard `Form<HashMap<String, String>>` extractor because it parses dynamic `match_{id}` and `confident_{id}` field names manually (`src/modules/predictions/handlers.rs:113-143`, `src/modules/predictions/models.rs:220-221`).

## Detailed Findings

### QsForm Extractor

- `MAX_FORM_BYTES` is a named private module constant set to `16 * 1024`, matching the ticket's required 16 KiB limit (`src/extractors.rs:7-8`).
- `QsForm<T>` implements `FromRequest<S>` with `type Rejection = (StatusCode, String)`, allowing extractor-level HTTP status responses without introducing a new `AppError` variant (`src/extractors.rs:10-18`).
- The body is read through `axum::body::to_bytes(req.into_body(), MAX_FORM_BYTES)`, so the cap is enforced before deserialization (`src/extractors.rs:19-21`).
- Any `to_bytes` error is currently treated as payload-too-large and converted to `413 Payload Too Large` with the stable body text `request body too large` (`src/extractors.rs:22-27`).
- `serde_qs::from_bytes::<T>(&bytes)` runs only after the body has passed the size gate, and parse errors still return `400 Bad Request` with the parser message (`src/extractors.rs:29-30`).

### Prediction Form Usage

- `QsForm` is imported only by the predictions handlers module (`src/modules/predictions/handlers.rs:9-13`).
- `save_knockout` uses `QsForm<KnockoutForm>` for POST `/predictions/knockout/{round}` (`src/modules/predictions/handlers.rs:165-170`, `src/modules/predictions/mod.rs:16-19`).
- `save_top_scorer` uses `QsForm<TopScorerForm>` for POST `/predictions/top-scorer` (`src/modules/predictions/handlers.rs:204-208`, `src/modules/predictions/mod.rs:20`).
- The backing form structs are small vector payloads with defaults: `KnockoutForm { team_ids: Vec<i64> }` and `TopScorerForm { player_ids: Vec<i64> }` (`src/modules/predictions/models.rs:223-233`).
- The templates submit repeated checkbox field names `team_ids` and `player_ids`, which matches the vector fields deserialized by `serde_qs` (`templates/predictions/index.html:249-265`, `templates/predictions/index.html:327-342`).
- The group prediction handler does not use `QsForm`; it uses Axum `Form<GroupStageForm>` where `GroupStageForm` is `HashMap<String, String>` and manually parses dynamic key prefixes (`src/modules/predictions/handlers.rs:113-143`, `src/modules/predictions/models.rs:220-221`).

### Error Handling Pattern

- Application-level handlers normally return `Result<impl IntoResponse, AppError>` and rely on `AppError::into_response` for status mapping (`src/error.rs:37-51`).
- `AppError::BadRequest` maps to `400`, and tests assert the response body for bad requests (`src/error.rs:47`, `src/error.rs:84-94`).
- `QsForm` intentionally bypasses `AppError` because Axum extractor rejections can be typed directly as `(StatusCode, String)` (`src/extractors.rs:17-30`). This matches the ticket's ADR note that `(StatusCode, String)` is acceptable for extractor rejections (`thoughts/tickets/qsform-body-limit.md:32-42`).
- The other custom extractor pattern is `AdminUser`, which uses `FromRequestParts<AppState>` and returns `AppError` for auth/authorization failures (`src/modules/admin/mod.rs:18-37`). This shows custom extractors may choose either `AppError` or direct `(StatusCode, String)` depending on whether they are enforcing application policy or low-level request parsing.

### Route Surface And Blast Radius

- Routes affected by `QsForm` are limited to knockout and top-scorer prediction writes (`src/modules/predictions/mod.rs:16-20`).
- Both affected handlers are authenticated user actions and still perform domain validation after extraction: selected team count for knockout and exactly three top-scorer picks (`src/modules/predictions/handlers.rs:171-201`, `src/modules/predictions/handlers.rs:209-228`).
- Because extraction happens before handler logic, oversized payloads are rejected before auth lookup, tournament lookup, lock checks, or DB writes execute.

## Code References

- `src/extractors.rs:7-8` - Defines `MAX_FORM_BYTES` as the named 16 KiB form body limit.
- `src/extractors.rs:19-27` - Reads request bodies with the cap and maps read errors to `413 Payload Too Large`.
- `src/extractors.rs:29-30` - Parses the body with `serde_qs` and keeps parse failures as `400 Bad Request`.
- `src/modules/predictions/handlers.rs:165-170` - `save_knockout` uses `QsForm<KnockoutForm>`.
- `src/modules/predictions/handlers.rs:204-208` - `save_top_scorer` uses `QsForm<TopScorerForm>`.
- `src/modules/predictions/models.rs:223-233` - Defines the small vector-based form payload structs consumed by `QsForm`.
- `templates/predictions/index.html:249-265` - Knockout prediction form posts repeated `team_ids` checkbox values.
- `templates/predictions/index.html:327-342` - Top-scorer prediction form posts repeated `player_ids` checkbox values.
- `src/error.rs:37-51` - Central `AppError` HTTP mapping for handler-level errors.
- `src/modules/admin/mod.rs:18-37` - Comparable custom extractor pattern for authorization.
- `src/modules/predictions/mod.rs:14-20` - Routes that expose prediction write handlers.

## Architecture Insights

The codebase separates low-level request parsing failures from domain/application failures. Handler errors use `AppError` as the central HTTP boundary, while `QsForm` uses Axum's extractor rejection mechanism directly. That direct rejection is appropriate here because payload-size and deserialization failures happen before the request reaches application logic and need precise HTTP status codes independent of the domain error enum.

The `QsForm` extractor is intentionally narrow: it supports forms whose repeated names should deserialize into vector fields via `serde_qs`. The standard `Form` extractor remains in use for the dynamic group-stage map shape, avoiding a repo-wide extractor migration.

The 16 KiB limit is sufficient for current consumers because the only `QsForm` payloads are checkbox IDs for tournament teams and top-scorer players. The largest expected knockout round currently requires 32 selected teams, and top scorer requires 3 selected players, far below 16 KiB.

## Historical Context (from thoughts/)

- `thoughts/tickets/qsform-body-limit.md:16-24` - Defines the security issue and acceptance criteria for a 16 KiB body cap, `413` oversized rejection, and named constant.
- `thoughts/tickets/qsform-body-limit.md:40-46` - Records the intended implementation and outcome: `MAX_FORM_BYTES = 16 * 1024`, body read errors map to `413`, parse errors remain `400`.
- `docs/adr/0009-error-handling-strategy.md:37-84` - Establishes `AppError` as the handler-level HTTP error boundary and `Result<impl IntoResponse, AppError>` as the normal handler pattern.
- `docs/adr/0009-error-handling-strategy.md:92-110` - Emphasizes centralized HTTP status mapping for application errors and avoiding panics in request handlers.
- `thoughts/tickets/scenario-modeling.md:68` - Notes possible reuse of `serde_qs`/`QsForm` for nested parameters in a future scenario-modeling context, but that ticket refers to query string parsing rather than current request-body use.

## Related Research

- `thoughts/research/2026-04-23_auth_integration_tests.md` - Covers extractor-based HTTP boundary testing and `AdminUser` authorization behavior.
- `thoughts/research/2026-04-23_google_oauth_login_flow.md` - Provides additional context on auth/session extractors and request-time rejection behavior.

## Open Questions

- There is no direct test for `QsForm` oversized body rejection. The ticket explicitly says no test is needed because Axum's `to_bytes` limit behavior is a framework guarantee, but a small extractor-level regression test could be added later if the team wants to lock the local `413` mapping.
- `to_bytes` read errors are all mapped to `413`, not just explicit length-limit errors. This matches the ticket's practical acceptance criteria for oversized bodies, but it may collapse rare non-size body read failures into `413` as well.

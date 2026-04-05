# ADR-0009: Error Handling Strategy 🚨

## Status

✅ Accepted

## Date

2026-04-05

## Context

Rust requires explicit error handling at every fallible call site. Without a consistent strategy, error types proliferate, `unwrap()` calls accumulate, and HTTP error responses become inconsistent. A clear approach must be established before application code is written.

The strategy must address three distinct concerns:

| Concern | Question |
|---------|---------|
| **Error definition** | How are error types declared and structured? |
| **Error propagation** | How are errors passed up the call stack with `?`? |
| **Error response** | How are errors converted to HTTP responses in Axum handlers? |

The established Rust ecosystem tools for these concerns:

| Crate | Purpose |
|-------|---------|
| **`thiserror`** | Derive macro for defining typed error enums with clean `Display` messages |
| **`anyhow`** | Ergonomic `anyhow::Error` type for propagating any error with `?` |
| **`thiserror` + `anyhow` combined** | `thiserror` at module boundaries, `anyhow` internally — industry standard pattern |

## Decision

We will use **`thiserror`** for defining typed domain errors and **`anyhow`** for internal error propagation, combined with a top-level **`AppError`** type that implements Axum's `IntoResponse` 🚨.

## The Three-Layer Pattern

### Layer 1 — Module errors with `thiserror` 🧩

Each module defines a typed error enum for errors that cross its public boundary:

```rust
// src/modules/users/models.rs
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("user not found: {id}")]
    NotFound { id: i64 },
    #[error("email already in use: {email}")]
    EmailConflict { email: String },
}
```

### Layer 2 — Internal propagation with `anyhow` 🔄

Inside handlers and DB functions, `anyhow::Result` is used for ergonomic `?` propagation without declaring every intermediate error type:

```rust
// src/modules/users/db.rs
pub async fn find_user(pool: &PgPool, id: i64) -> anyhow::Result<User> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| UserError::NotFound { id }.into())
}
```

### Layer 3 — HTTP response via `AppError` 🌐

A single `AppError` type in `src/error.rs` implements `IntoResponse`, mapping domain errors to HTTP status codes and JSON error bodies:

```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    User(#[from] UserError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::User(UserError::NotFound { .. }) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::User(UserError::EmailConflict { .. }) => (StatusCode::CONFLICT, self.to_string()),
            AppError::Unexpected(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string()),
        };
        (status, message).into_response()
    }
}
```

Axum handlers return `Result<impl IntoResponse, AppError>`, using `?` freely throughout.

## Rationale

1. 🧩 **`thiserror` at boundaries — explicit, typed, documented**: Domain errors at module boundaries are named, structured, and carry context. They are part of the module's public contract and must be handled deliberately by callers.

2. 🔄 **`anyhow` internally — ergonomic, low ceremony**: Internal implementation code uses `anyhow::Result` to propagate errors with `?` without declaring wrapper types for every SQLx, IO, or serialisation error that is not meaningful to the caller.

3. 🌐 **Single `AppError` as the HTTP boundary**: One place in the codebase maps all errors to HTTP responses. This ensures consistency, makes it easy to audit what the API can return, and prevents ad-hoc status code decisions scattered across handlers.

4. 🔒 **Unexpected errors are opaque to clients**: `AppError::Unexpected` maps to `500 Internal Server Error` with a generic message. Internal error details are logged (via `tracing`, ADR-0010) but never leaked to HTTP responses.

5. 🚫 **No `unwrap()` or `expect()` in application code**: Panics in async Axum handlers poison the task. All fallible operations use `?`. `unwrap()` is reserved for test code or cases where the invariant is guaranteed by construction and documented.

## Trade-offs and Risks ⚠️

- 📋 **`AppError` grows over time**: As modules are added, `AppError` accumulates variants and match arms. This is manageable and preferable to inconsistent per-handler error handling.
- 🔄 **`anyhow` loses type information**: Once an error is wrapped in `anyhow::Error`, its concrete type is erased. This is intentional for internal errors but means module boundary errors must be explicitly converted before entering `anyhow` context if they need to be matched later.
- 🧩 **Two crates to learn**: Contributors must understand when to use `thiserror` vs `anyhow`. The rule is simple: `thiserror` at public module boundaries, `anyhow` everywhere else.

## Consequences

- 🧩 Every module that can produce errors callable from outside defines a typed error enum using `thiserror`.
- 🌐 `src/error.rs` contains `AppError` with `IntoResponse` — the only place HTTP status codes are assigned to errors.
- 🔄 All Axum handlers return `Result<impl IntoResponse, AppError>`.
- 🚫 `unwrap()` and `expect()` are forbidden in `src/` outside of test modules; `clippy::unwrap_used` lint is enabled.
- 📋 Internal server errors are logged at `error` level with full context before being converted to opaque `500` responses.

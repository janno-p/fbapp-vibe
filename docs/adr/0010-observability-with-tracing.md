# ADR-0010: Use tracing for Observability 🔭

## Status

✅ Accepted

## Date

2026-04-05

## Context

The application needs structured logging and instrumentation to support debugging in development and observability in production. The choice of observability tooling affects how request context is propagated, how logs are structured, and how the application integrates with external monitoring systems.

The Rust ecosystem offers two logging approaches:

| | **`log` crate** | **`tracing` crate** |
|--|----------------|-------------------|
| Abstraction | Simple key-value log records | Structured spans + events with context propagation |
| Async awareness | ❌ No concept of async tasks | ✅ Spans attach to async tasks automatically |
| Structured fields | ❌ String-only messages | ✅ Typed key-value fields on spans and events |
| Tokio integration | Limited | ✅ First-class — developed by the Tokio team |
| Axum integration | Manual | ✅ `tower-http` `TraceLayer` built-in |
| Production backends | `env_logger`, `pretty_env_logger` | JSON output, OpenTelemetry, Jaeger, Datadog |

The `log` crate is the older standard; `tracing` is its async-native successor and the de-facto standard in the Tokio/Axum ecosystem. Most major async Rust libraries (SQLx, Axum, Hyper, Tower) emit `tracing` events natively.

## Decision

We will use the **`tracing`** ecosystem 🔭 for all application observability:

- **`tracing`** — instrumentation API (spans, events, macros)
- **`tracing-subscriber`** — subscriber that formats and outputs trace data
- **`tower-http` `TraceLayer`** — automatic HTTP request/response instrumentation

## Rationale

1. ⚡ **Async-aware context propagation**: `tracing` spans model the lifecycle of async operations. When a request spawns multiple futures, the span context follows each one, making it possible to correlate all log events for a single request across async boundaries — something the `log` crate cannot do.

2. 🧩 **Structured fields, not string messages**: `tracing` events carry typed key-value fields (`user_id = %id`, `status = %status_code`) rather than interpolated strings. This makes logs machine-parseable and queryable in production log aggregators without regex.

3. 🔌 **First-class Axum and Tower integration**: `tower-http`'s `TraceLayer` instruments every HTTP request with a span containing method, URI, status code, and latency automatically. No per-handler instrumentation is needed for basic request logging.

4. 🌍 **Ecosystem standard**: SQLx, Axum, Hyper, and most async Rust libraries emit `tracing` events natively. Using `tracing` means library internals are visible in the same trace output without any extra configuration.

5. 📈 **Production-ready backends**: `tracing-subscriber` supports JSON output out of the box, which feeds directly into log aggregators (Datadog, Loki, CloudWatch). OpenTelemetry export is available via `tracing-opentelemetry` if distributed tracing becomes necessary.

## Setup Pattern

```rust
// src/main.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "fbapp_vibe=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

```rust
// src/routes.rs — HTTP tracing middleware
use tower_http::trace::TraceLayer;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(users::router())
        // ... other modules
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
```

```rust
// src/modules/users/handlers.rs — per-handler instrumentation
#[tracing::instrument(skip(state), fields(user_id = %id))]
async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    // all log events inside this fn are attached to the span
    tracing::debug!("fetching user");
    // ...
}
```

## Log Levels Convention 📋

| Level | When to use |
|-------|-------------|
| `ERROR` | Unrecoverable errors, unexpected failures, 5xx responses |
| `WARN` | Recoverable issues, degraded behaviour, 4xx responses |
| `INFO` | Significant application events (startup, shutdown, migrations) |
| `DEBUG` | Request-level detail, DB queries, useful during development |
| `TRACE` | Very fine-grained detail, framework internals — disabled in production |

## Trade-offs and Risks ⚠️

- 📦 **Additional crates**: The `tracing` ecosystem requires several crates (`tracing`, `tracing-subscriber`, `tower-http`). These are all maintained by the Tokio team and considered stable.
- 🔧 **`RUST_LOG` env var for level control**: Log verbosity is controlled via the `RUST_LOG` environment variable (e.g. `RUST_LOG=fbapp_vibe=debug,sqlx=warn`). This must be documented for operators.
- 📈 **JSON output for production**: The default `fmt` layer outputs human-readable text. Switching to JSON for production requires a conditional subscriber setup or a separate build configuration — this is addressed in the deployment ADR.

## Consequences

- 🔭 All application code uses `tracing::info!`, `tracing::debug!`, `tracing::error!` etc. — the `log` crate is not used directly.
- 🔌 `TraceLayer::new_for_http()` is applied at the top-level router in `src/routes.rs`, providing automatic HTTP request instrumentation.
- 🧩 Axum handler functions that benefit from request-scoped context are annotated with `#[tracing::instrument]`.
- 📋 Log level is configured via the `RUST_LOG` environment variable; the default in development is `fbapp_vibe=debug,tower_http=debug,sqlx=warn`.
- 🚀 The `tracing-subscriber` initialisation in `main.rs` is the single place where the output format and filter are configured.
- 📈 Switching to JSON output or adding OpenTelemetry export requires only changes to the subscriber initialisation — no application code changes.

# ADR-0002: Use Axum as the Web Backend Framework 🌐

## Status

✅ Accepted

## Date

2026-04-05

## Context

Having decided to use Rust 🦀 (ADR-0001), we need to select a web backend framework. The framework choice affects request routing, middleware composition, async runtime integration, error handling ergonomics, and the breadth of available ecosystem tooling.

The two strongest candidates evaluated were **Axum** and **Actix-web**. Other frameworks (Warp, Rocket, Poem) were considered but ruled out early due to less active development, heavier macro usage, or smaller communities.

### ⚖️ Axum vs Actix-web Evaluation

| Criterion | Axum | Actix-web |
|-----------|------|-----------|
| Foundation | `hyper` + `tower` + `tokio` | `actix` actor system + `tokio` |
| Raw performance 🏎️ | Slightly lower | Highest benchmarks |
| Middleware reuse 🧩 | Any `tower`-compatible crate | Actix-specific middleware only |
| Ergonomics 🤝 | Clean extractor-based handlers | Similar, slightly more ceremony |
| Error handling 🚨 | `IntoResponse` trait, natural `?` usage | `ResponseError` trait, more verbose |
| Maturity 📅 | Since 2021, stable at v0.7 | Since 2017, stable at v4 |
| Maintainer 👷 | Tokio team (official project) | Community-maintained |
| Testing 🧪 | `tower::ServiceExt::oneshot`, no port needed | `actix_web::test` helpers |

**🏎️ Performance**: Actix-web leads in TechEmpower benchmarks (e.g. ~7.2M vs ~6.1M plaintext req/s in Round 22), but this gap is negligible in practice — both frameworks are IO-bound at real-world load levels before framework overhead becomes a factor.

**🧩 Middleware**: Axum's use of the `tower::Service` trait means all tower-ecosystem middleware (`tower-http`, `tower-governor`, tracing integrations, etc.) works out of the box without adaptation. Actix-web requires actix-specific middleware wrappers, limiting reuse.

**🌱 Longevity**: Axum is developed and maintained by the Tokio team as an official project, giving it strong alignment with the async Rust ecosystem's direction.

## Decision

We will use **Axum** 🌐 as the web backend framework.

## Rationale

1. 🧩 **Tower ecosystem interoperability**: Axum is built on `tower::Service`, making every tower-compatible middleware (rate limiting, tracing, compression, timeouts, auth) immediately usable without glue code. This avoids reinventing solutions that the broader ecosystem already provides.

2. ⚙️ **Tokio alignment**: Axum is an official Tokio project, ensuring first-class integration with `tokio`, `hyper`, and related crates that form the foundation of async Rust networking.

3. 🤝 **Ergonomic handler signatures**: Handlers are plain async functions whose parameters are type-safe extractors validated at compile time. There is no framework-specific trait to implement; any compatible function signature works.

4. 🚨 **Idiomatic error handling**: Returning errors via the `IntoResponse` trait integrates naturally with `?`, `thiserror`, and `anyhow`, keeping error propagation consistent with the rest of the Rust codebase.

5. 🧪 **Testability**: Axum routers are `tower::Service` instances and can be tested in-process with `oneshot` calls — no port binding, no process spawning required.

6. ⚡ **Sufficient performance**: Axum's throughput is more than adequate for the expected workload. The marginal performance advantage of Actix-web does not justify the trade-offs in middleware portability and ecosystem fit.

## Trade-offs and Risks ⚠️

- 🐣 **Younger than Actix-web**: Axum has fewer years of production exposure. This is partially mitigated by its underlying components (`hyper`, `tokio`, `tower`) being very mature and battle-tested.
- 📉 **Lower raw throughput ceiling**: If the application eventually requires handling tens of millions of requests per second, the framework choice may need to be revisited. This is not an anticipated requirement.
- 📚 **Tower learning curve**: Developers unfamiliar with the `tower` middleware model may need onboarding time, though this knowledge transfers to any tower-based service.

## Consequences

- 🌐 The HTTP server is built with `axum` as the primary routing and handler layer.
- 🧩 Middleware is sourced from the `tower` and `tower-http` crates wherever possible before writing custom middleware.
- ⚙️ The async runtime is `tokio` (already implied by Axum's dependency).
- 🗄️ Shared application state is passed via Axum's `State` extractor, backed by `Arc`.
- 🧪 Integration tests use `tower::ServiceExt::oneshot` to call handlers without binding a real network port.
- 📋 Future ADRs will document choices for specific cross-cutting concerns (database access, authentication, observability) that build on top of this framework selection.

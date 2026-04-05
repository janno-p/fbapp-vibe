# ADR-0001: Use Rust as the Programming Language 🦀

## Status

✅ Accepted

## Date

2026-04-05

## Context

We are building `fbapp-vibe`, a new application that requires a choice of primary programming language. The decision affects developer experience, runtime performance, safety guarantees, ecosystem availability, and long-term maintainability.

Key requirements that informed this decision:

- ⚡ **Performance**: The application is expected to handle workloads where runtime efficiency matters (CPU and/or memory).
- 🛡️ **Reliability**: Correctness and stability are priorities; crash-prone or memory-unsafe behavior is unacceptable in production.
- 🔀 **Concurrency**: The application may need to handle concurrent operations efficiently.
- 🔧 **Long-term maintainability**: The codebase should be easy to reason about and refactor safely over time.

Alternatives considered:

| Language | Pros | Cons |
|----------|------|------|
| **Go** | Simple, fast compilation, good concurrency primitives, strong stdlib | Garbage collector pauses, limited generics expressiveness, less fine-grained control |
| **C++** | Maximum performance, mature ecosystem | No memory safety guarantees, undefined behavior risks, complex build tooling |
| **TypeScript/Node.js** | Large ecosystem, fast iteration, full-stack sharing | GC overhead, single-threaded event loop limitations, runtime type erasure |
| **Python** | Rapid prototyping, large ML/data ecosystem | Slow runtime, GIL limits concurrency, dynamic typing hides bugs |
| **Rust** 🦀 | Memory safety without GC, zero-cost abstractions, fearless concurrency, strong type system | Steeper learning curve, longer initial development time |

## Decision

We will use **Rust** 🦀 as the primary programming language for this project.

## Rationale

1. 🔒 **Memory safety without garbage collection**: Rust's ownership model eliminates entire classes of bugs (use-after-free, data races, null pointer dereferences) at compile time, with no runtime GC overhead.

2. ⚡ **Performance**: Rust produces native binaries with performance comparable to C/C++, making it suitable if latency or throughput become constraints.

3. 😌 **Fearless concurrency**: The borrow checker enforces safe concurrent access at compile time, preventing data races without requiring a runtime or manual locking discipline.

4. 🧩 **Strong type system and expressive abstractions**: Algebraic data types, traits, and pattern matching allow modeling domain concepts precisely and catching errors at compile time rather than at runtime.

5. 🛠️ **Modern tooling**: `cargo` provides a unified build system, dependency management, testing, and documentation generation. The ecosystem around `rustfmt`, `clippy`, and `rust-analyzer` supports high code quality with low friction.

6. 📦 **Reliable dependencies**: The crates.io ecosystem is mature for systems-level and network-facing workloads, and Rust's semver tooling makes dependency management predictable.

7. ♻️ **Long-term maintainability**: The compiler's strict guarantees make large-scale refactors safer and more tractable than in dynamically typed or memory-unsafe languages.

## Trade-offs and Risks ⚠️

- 📈 **Learning curve**: Rust's ownership and lifetime concepts require upfront investment for developers new to the language. This is mitigated by strong tooling and compiler error messages that guide the developer.
- 🐌 **Compile times**: Rust compilation is slower than Go or TypeScript. Incremental compilation and `cargo`'s caching reduce day-to-day impact.
- 🌿 **Ecosystem gaps**: Some domains (e.g., certain ML libraries) have a less mature Rust ecosystem than Python or JavaScript. If such integrations become necessary, FFI or separate service boundaries will be used.

## Consequences

- All new application code is written in Rust 🦀 unless a specific component has a documented justification for using another language.
- The project uses `cargo` as the build and dependency management tool.
- Contributors are expected to follow idiomatic Rust practices enforced by `clippy` and `rustfmt`.
- Future ADRs may specify which Rust edition and which key crates are adopted for cross-cutting concerns (async runtime, serialization, etc.).

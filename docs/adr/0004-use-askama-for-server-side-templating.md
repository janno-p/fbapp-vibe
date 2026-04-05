# ADR-0004: Use Askama for Server-Side Templating 📄

## Status

✅ Accepted

## Date

2026-04-05

## Context

ADR-0003 established HTMX as the client-server integration approach, which requires the Axum backend to render HTML — both full pages and partial fragments in response to HTMX requests. A templating engine must be chosen to fulfil this responsibility.

The candidates evaluated:

| | **Askama** | **MiniJinja** | **Tera** | **Maud** |
|--|-----------|--------------|---------|------|
| Template style | `.html` files, Jinja2 syntax | `.html` files, Jinja2 syntax | `.html` files, Jinja2 syntax | Rust macros in `.rs` files |
| Validation ✅ | Compile-time | Runtime | Runtime | Compile-time |
| Performance ⚡ | Fastest (zero-cost) | Fast | Fast | Fastest (zero-cost) |
| Hot reload 🔥 | No (recompile needed) | Yes | Yes | No (recompile needed) |
| Axum integration 🔌 | `askama_axum` crate | Manual `impl IntoResponse` | Manual `impl IntoResponse` | Native `impl IntoResponse` |
| Designer-friendly 🎨 | Yes | Yes | Yes | No — pure Rust code |
| Ecosystem popularity 📦 | Highest in Axum community | Moderate | Moderate | Niche |

Key requirements:

- 🛡️ **Safety**: Template errors should be caught as early as possible, ideally at compile time.
- ⚡ **Performance**: Template rendering should add minimal overhead to request handling.
- 🔌 **Axum integration**: The engine should integrate cleanly with Axum's `IntoResponse` trait to return rendered HTML from handlers.
- 🎨 **Readability**: Templates should be readable and maintainable as standalone `.html` files.

## Decision

We will use **Askama** 📄 for server-side HTML templating.

## Rationale

1. 🛡️ **Compile-time template validation**: Askama parses and type-checks templates at compile time. Missing variables, type mismatches, and syntax errors are caught before the application runs — consistent with Rust's overall philosophy of finding bugs at compile time.

2. ⚡ **Zero-cost rendering**: Templates are compiled to Rust code, not interpreted at runtime. There is no parsing or reflection overhead per request; rendering performance is equivalent to hand-written string formatting.

3. 🔌 **First-class Axum integration**: The `askama_axum` crate provides an `IntoResponse` implementation for Askama templates out of the box. Handlers return a typed template struct directly, with no boilerplate.

4. 🎨 **Jinja2-like syntax in `.html` files**: Templates live in separate `.html` files with familiar Jinja2-style syntax (`{% for %}`, `{% if %}`, `{{ variable }}`). This keeps HTML readable and separable from Rust logic, and is approachable for anyone familiar with Jinja2, Django templates, or Nunjucks.

5. 📦 **Strongest ecosystem fit**: Askama is the most widely used templating engine in the Axum and Actix-web communities, meaning examples, integrations, and community support are readily available.

6. 🔄 **HTMX fragment rendering**: Askama supports template inheritance and blocks, making it straightforward to define base layouts and render isolated partial templates as HTMX fragments from the same template hierarchy.

## Trade-offs and Risks ⚠️

- 🔄 **No hot reload**: Template changes require recompilation. During development this adds friction compared to runtime engines like MiniJinja or Tera. This is partially mitigated by `cargo-watch` triggering incremental rebuilds automatically on file save.
- 📁 **Template files are separate from code**: Unlike Maud, templates are not co-located with Rust logic. This is a deliberate trade-off favouring readability over co-location.
- 🧩 **Limited dynamic template loading**: Templates must be known at compile time; dynamically loading templates from a database or external source is not supported. This is not a requirement for this project.

## Consequences

- 📄 All HTML rendering uses Askama templates stored under the `templates/` directory.
- 🔌 The `askama` and `askama_axum` crates are added as dependencies.
- 🏗️ Template structs are defined in Rust and annotated with `#[derive(Template)]` and `#[template(path = "...")]`.
- 🔄 HTMX partial responses are implemented as dedicated partial templates or Askama blocks rendered independently from the full-page layout.
- 🛠️ `cargo-watch` is the recommended development tool to trigger automatic recompilation on template changes.
- 🎨 The `templates/` directory structure mirrors the application's route hierarchy for discoverability.

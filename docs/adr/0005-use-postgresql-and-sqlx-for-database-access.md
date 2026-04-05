# ADR-0005: Use PostgreSQL and SQLx for Database Access 🗄️

## Status

✅ Accepted

## Date

2026-04-05

## Context

The application requires persistent storage. Two coupled decisions must be made: the **database engine** and the **Rust access layer** that the Axum backend uses to communicate with it.

### 🗄️ Database Options

| | **PostgreSQL** | **SQLite** | **MySQL/MariaDB** |
|--|---------------|-----------|-----------------|
| Best for | Production web apps, concurrent writes, complex queries | Embedded, single-user, dev/test | Legacy or specific hosting requirements |
| Infrastructure | Requires server or managed service | Single file, zero infrastructure | Requires server or managed service |
| Features | Full SQL, JSONB, full-text search, extensions, row-level locking | Good SQL subset, limited write concurrency | Good SQL subset, fewer advanced features |
| Rust ecosystem | Excellent | Excellent | Good |

### 🔌 Access Layer Options

| | **SQLx** | **Diesel** | **SeaORM** |
|--|---------|-----------|-----------|
| Style | Raw SQL with type-safe results | ORM + query builder DSL | Async ORM built on SQLx |
| Async | ✅ Native async | ❌ Sync (`spawn_blocking` required) | ✅ Native async |
| Query validation | ✅ Compile-time (against live DB) | ✅ Compile-time | Runtime |
| SQL control | Full — developer writes SQL | Partial — ORM generates SQL | Partial — ORM generates SQL |
| Migrations | ✅ Built-in (`sqlx migrate`) | ✅ Built-in (`diesel migration`) | ✅ Built-in |
| Axum ecosystem fit | ⭐ Most popular pairing | Less common | Growing |
| Learning curve | Low — SQL + Rust types | Medium — DSL to learn | Medium |

## Decision

We will use **PostgreSQL** 🐘 as the database engine and **SQLx** 🔌 as the Rust database access layer.

## Rationale

### PostgreSQL 🐘

1. 🏗️ **Production-grade reliability**: PostgreSQL is a proven, ACID-compliant database with strong support for concurrent workloads, making it suitable from early development through production scale.

2. 🧩 **Rich feature set**: JSONB columns, full-text search, array types, row-level security, and a mature extension ecosystem (e.g. `pgcrypto`, `pg_trgm`) provide capabilities that SQLite cannot match.

3. 🌍 **Managed service availability**: All major cloud providers offer managed PostgreSQL (AWS RDS, Supabase, Neon, Render), simplifying operations and backups.

4. 📦 **Best Rust ecosystem support**: PostgreSQL has the deepest support across SQLx, Diesel, and SeaORM, and the widest range of community examples targeting the Axum stack.

### SQLx 🔌

1. ⚡ **Async-native**: SQLx is built for async Rust from the ground up. There is no need for `spawn_blocking` wrappers, and it integrates naturally with the Tokio runtime used by Axum.

2. 🛡️ **Compile-time query verification**: SQLx macros (`sqlx::query!`, `sqlx::query_as!`) validate SQL queries against a live database at compile time, catching syntax errors, missing columns, and type mismatches before the application runs.

3. 🎯 **Full SQL control**: Developers write real SQL rather than an ORM DSL. This makes queries predictable, debuggable, and portable — there are no surprises about what SQL gets generated.

4. 🔄 **Built-in migrations**: `sqlx migrate` provides a simple, version-controlled migration system that integrates with the application binary and can run migrations automatically on startup.

5. 🔌 **First-class Axum fit**: SQLx's `PgPool` is cheaply cloneable and designed to be stored in Axum's `State`, making connection pool sharing across handlers idiomatic and zero-friction.

6. 📦 **Offline mode**: SQLx supports an offline mode (`sqlx prepare`) that caches query metadata, allowing compilation without a live database connection in CI environments.

## Trade-offs and Risks ⚠️

- 🏗️ **Infrastructure requirement**: Unlike SQLite, PostgreSQL requires a running database server for development and CI. This is mitigated by using Docker Compose locally and a managed service in production.
- 🔄 **Compile-time queries need a live DB**: SQLx's compile-time checking requires a database connection during `cargo build` (unless offline mode is used). The team must maintain a local development database or use SQLx offline mode in CI.
- 📝 **More boilerplate than an ORM**: Without an ORM generating queries, developers write more SQL by hand. This is a deliberate trade-off for clarity and control.
- 🔍 **No lazy loading**: SQLx has no ORM-style lazy loading; all data fetching must be explicit. This avoids N+1 query surprises but requires more deliberate data access planning.

## Consequences

- 🐘 PostgreSQL is the only supported database engine; no database abstraction layer is introduced to support multiple engines.
- 🔌 The `sqlx` crate is added as a dependency with the `postgres`, `runtime-tokio-rustls`, and `macros` features enabled.
- 🏊 A `PgPool` connection pool is initialised at application startup and injected into Axum handlers via the `State` extractor.
- 🗂️ Database migrations are managed with `sqlx migrate` and stored in the `migrations/` directory at the project root.
- 🛡️ Compile-time query checking (`sqlx::query!` / `sqlx::query_as!`) is the default approach; dynamic query construction is avoided unless explicitly justified.
- 🐳 A `docker-compose.yml` providing a local PostgreSQL instance is maintained for development and CI.
- 📦 `sqlx prepare` is run to generate an `.sqlx/` metadata cache, enabling compilation in CI without a live database.

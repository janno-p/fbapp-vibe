# ADR-0008: Use dotenvy and envy for Configuration Management ⚙️

## Status

✅ Accepted

## Date

2026-04-05

## Context

The application requires a strategy for managing configuration values such as database URLs, server ports, secrets, and feature flags. Configuration must work consistently across local development, CI, and production environments without code changes.

The candidates evaluated:

| | **`dotenvy` + `envy`** | **`dotenvy` + `config` crate** | **`figment`** |
|--|----------------------|-------------------------------|--------------|
| Approach | Deserialise env vars into a typed struct | Layered: env vars + config files + defaults | Flexible layered config from any source |
| Type safety | ✅ Compile-time struct | ✅ Compile-time struct | ✅ Compile-time struct |
| Multiple config sources | ❌ Env vars only | ✅ Files + env vars + defaults | ✅ Files + env vars + defaults |
| Complexity | Low | Medium | Medium |
| Dependencies | 2 small crates | Several | Moderate |
| Common in Axum ecosystem | ✅ Very common | Common | Less common |

Key requirements:

- 🔒 **Secrets stay out of source control**: Sensitive values (database passwords, API keys) must never be committed to the repository.
- 🏗️ **Environment parity**: The same binary must run in development, CI, and production with configuration injected via environment variables.
- 🛡️ **Type safety**: Configuration values should be validated and typed at startup, not discovered as panics at runtime.
- 🪶 **Simplicity**: The configuration mechanism should be easy to understand and extend.

## Decision

We will use **`dotenvy`** and **`envy`** ⚙️ for configuration management.

## Rationale

1. 🌍 **Environment variables are the universal config primitive**: All deployment targets — Docker, cloud platforms, CI systems — natively support environment variables. Using env vars as the sole config source avoids format-specific parsing and keeps the app twelve-factor compliant.

2. 🛡️ **Typed config struct at startup**: `envy` deserialises environment variables directly into a typed Rust struct using `serde`. Missing required values and type mismatches cause the application to fail at startup with a clear error — not silently at the call site.

3. 🔒 **`.env` files for local development only**: `dotenvy` loads a `.env` file when present, populating environment variables for local development without any code changes. In CI and production, real environment variables are used directly and `.env` is absent. `.env` is gitignored.

4. 🪶 **Minimal complexity**: The `dotenvy` + `envy` combination covers the needs of this application with two small, focused crates and no additional abstraction. The `config` crate and `figment` offer layered file-based config, which is unnecessary overhead when environment variables are sufficient.

5. 📋 **Self-documenting config**: The `Config` struct serves as the authoritative list of all configuration values the application accepts, making it easy to audit what is required for a deployment.

## Config Struct Pattern

```rust
// src/config.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 { 3000 }

impl Config {
    pub fn load() -> Result<Self, envy::Error> {
        dotenvy::dotenv().ok(); // no-op if .env absent
        envy::from_env::<Config>()
    }
}
```

## Trade-offs and Risks ⚠️

- 📁 **No layered config files**: Unlike `config` or `figment`, this approach does not support environment-specific config files (e.g. `config/production.toml`). If file-based layered configuration becomes necessary, the approach should be revisited in a future ADR.
- 🔄 **No runtime config reload**: Configuration is read once at startup. Dynamic config changes require a process restart. This is acceptable for the current requirements.
- 📋 **All config is flat**: Nested configuration structures require custom `serde` handling or prefixed env var naming conventions (e.g. `DATABASE__URL`). The config struct should be kept flat where possible.

## Consequences

- ⚙️ A `Config` struct in `src/config.rs` is the single source of truth for all application configuration.
- 🚀 `Config::load()` is called once in `main.rs` at startup; failure exits the process immediately with a descriptive error.
- 📦 The loaded `Config` is stored in `AppState` (see ADR-0007) and accessed via Axum's `State` extractor.
- 🔒 `.env` is added to `.gitignore`; a `.env.example` file with placeholder values is committed to the repository as documentation.
- 🌍 All configuration in production is supplied via environment variables — no config files are read in production.

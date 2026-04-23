---
title: Self-signed TLS certificate for HTTPS in development
source: .claude/tasks/done/0003-https-dev-tls.md
source_id: 0003
source_status: done
source_title: Self-signed TLS certificate for HTTPS in development
status: done
type: chore
adrs: []
refs: [0002]
created: 2026-04-06
started: 2026-04-06
completed: 2026-04-06
---

## Summary

Enable the development server to serve traffic over HTTPS using a locally-trusted self-signed certificate. This is required because the OAuth `session_secret` cookie is set with `with_secure(true)` in production mode and browsers refuse to send secure cookies over plain HTTP. Running on HTTPS locally also ensures the development environment matches production behaviour and avoids subtle auth bugs caused by the HTTP/HTTPS mismatch.

## Acceptance Criteria

- [ ] `cargo run` (or `make dev`) starts the server on HTTPS (e.g. `https://localhost:3000`)
- [ ] `GET https://localhost:3000/health` returns `200 OK` in a browser without a certificate error (certificate is trusted by the local OS/browser trust store)
- [ ] Certificate and key files are generated locally and are gitignored
- [ ] `SessionManagerLayer` uses `with_secure(true)` when running with TLS
- [ ] `.env.example` documents any new environment variables (e.g. `TLS_CERT_PATH`, `TLS_KEY_PATH`)
- [ ] `README.md` Getting Started section includes a step for generating and trusting the certificate
- [ ] `cargo build` succeeds with zero warnings and zero clippy errors

## Implementation Context

### Recommended approach

Use [`mkcert`](https://github.com/FiloSottile/mkcert) to generate a locally-trusted certificate — it installs a local CA into the OS/browser trust store so there are no certificate warnings:

```bash
# Install mkcert (once per machine)
brew install mkcert        # macOS
# or: choco install mkcert  # Windows
# or: apt install mkcert    # Debian/Ubuntu

mkcert -install            # installs local CA into trust store
mkcert localhost 127.0.0.1 # generates localhost.pem + localhost-key.pem
```

Place the generated files in a `certs/` directory at the project root (gitignored).

### Axum TLS

Axum itself does not handle TLS — use `axum-server` with the `tls-rustls` feature as a drop-in replacement for `axum::serve`:

```toml
# Cargo.toml
axum-server = { version = "0.7", features = ["tls-rustls"] }
```

```rust
// main.rs — replace axum::serve with:
use axum_server::tls_rustls::RustlsConfig;

let tls_config = RustlsConfig::from_pem_file(
    config.tls_cert_path.as_ref().expect("TLS_CERT_PATH required"),
    config.tls_key_path.as_ref().expect("TLS_KEY_PATH required"),
).await?;

let addr = SocketAddr::new(config.host.parse()?, config.port);
axum_server::bind_rustls(addr, tls_config)
    .serve(app.into_make_service())
    .await?;
```

### Config additions

```rust
// src/config.rs — add optional fields so TLS is opt-in
pub tls_cert_path: Option<String>,
pub tls_key_path: Option<String>,
```

When both fields are `Some`, start HTTPS; otherwise fall back to plain HTTP (keeps CI and simple local runs working without certs).

### .gitignore addition

```
certs/
```

### .env.example additions

```
# TLS (optional — leave unset for plain HTTP)
TLS_CERT_PATH=certs/localhost.pem
TLS_KEY_PATH=certs/localhost-key.pem
```

### README step to add (after "Run database migrations")

````markdown
### 5b. Enable HTTPS (optional but recommended)

Install `mkcert` and generate a locally-trusted certificate:

```bash
mkcert -install
mkdir certs
mkcert -cert-file certs/localhost.pem -key-file certs/localhost-key.pem localhost 127.0.0.1
```

Then set `TLS_CERT_PATH` and `TLS_KEY_PATH` in `.env` (see `.env.example`).
````

### Files to modify

- `Cargo.toml` — add `axum-server` dependency
- `src/config.rs` — add `tls_cert_path`, `tls_key_path` optional fields
- `src/main.rs` — conditional TLS bind; `with_secure(true)` when TLS is active
- `.env.example` — document new vars
- `.gitignore` — ignore `certs/`
- `README.md` — add certificate generation step

## Outcome

TLS is opt-in via `TLS_CERT_PATH` and `TLS_KEY_PATH` env vars. When both are set, the server binds with `axum-server` + Rustls and `with_secure(true)` on the session cookie. When unset, it falls back to plain HTTP via `axum::serve` — no change to existing behaviour. `mkcert` is the recommended tool for generating a locally-trusted certificate. `certs/` is gitignored.

Follow-up tasks: _none_

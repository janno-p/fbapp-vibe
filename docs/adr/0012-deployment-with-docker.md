# ADR-0012: Containerise with Docker 🐳

## Status

✅ Accepted

## Date

2026-04-05

## Context

The application must be packaged for consistent deployment across development, CI, and production environments. Containerisation is the standard approach for achieving environment parity and simplifying deployment to cloud platforms.

Key requirements:

- 📦 **Small image size**: The production image should contain only what is needed to run the binary.
- 🔧 **Reproducible builds**: The build process must produce the same image regardless of the host machine.
- 🐳 **Local development parity**: Developers must be able to run the full stack (app + PostgreSQL) locally with a single command.
- 🔒 **Security**: The production image should have a minimal attack surface.

### Runtime Base Image Options

| | **`debian:bookworm-slim`** | **`alpine`** | **`gcr.io/distroless/cc`** |
|--|--------------------------|-------------|--------------------------|
| Final image size | ~80MB | ~10MB | ~20MB |
| C library | glibc | musl | glibc |
| Rust compatibility | ✅ Full | ⚠️ Requires musl cross-compile | ✅ Full |
| Shell access | ✅ Yes | ✅ Yes | ❌ None |
| Security surface | Medium | Small | Smallest |
| Debugging ease | ✅ Easy | ✅ Moderate | ❌ Difficult |

Alpine requires compiling against musl libc, which causes compatibility issues with certain crates (notably those with OpenSSL or C FFI dependencies) and requires additional build configuration. Distroless minimises the attack surface but makes live debugging impossible.

## Decision

We will use **multi-stage Docker builds** 🐳 with `debian:bookworm-slim` as the production runtime base image, and **Docker Compose** for local development orchestration.

## Dockerfile Pattern

```dockerfile
# ── Stage 1: Builder ─────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install system dependencies for SQLx and TLS
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies separately from application code
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Build application
COPY . .
RUN cargo build --release

# ── Stage 2: Tailwind CSS build ───────────────────────────────────────────────
FROM node:20-slim AS css-builder

WORKDIR /app
COPY tailwind.config.js ./
COPY templates/ ./templates/
COPY assets/ ./assets/
RUN npx --yes tailwindcss -i ./assets/css/input.css -o ./assets/css/main.css --minify

# ── Stage 3: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Non-root user for security
RUN useradd -ms /bin/bash appuser
USER appuser

COPY --from=builder /app/target/release/fbapp-vibe ./fbapp-vibe
COPY --from=css-builder /app/assets ./assets
COPY templates/ ./templates/
COPY migrations/ ./migrations/

EXPOSE 3000
CMD ["./fbapp-vibe"]
```

## Docker Compose for Local Development

```yaml
# docker-compose.yml
services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgres://fbapp:fbapp@db:5432/fbapp
      HOST: 0.0.0.0
      PORT: 3000
    depends_on:
      db:
        condition: service_healthy

  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: fbapp
      POSTGRES_PASSWORD: fbapp
      POSTGRES_DB: fbapp
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U fbapp"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
```

## Rationale

1. 🏗️ **Multi-stage build keeps the image lean**: The Rust toolchain (~1.5GB) is used only in the builder stage and is discarded in the final image. The production image contains only the compiled binary, static assets, templates, and runtime libraries.

2. 📦 **Dependency caching layer**: Copying `Cargo.toml` and `Cargo.lock` and building a stub `main.rs` before copying application source allows Docker's layer cache to skip the dependency compilation step when only application code changes — significantly reducing build times.

3. 🔒 **Non-root user**: The application runs as a non-root `appuser` in the container, reducing the blast radius of any security vulnerability.

4. 🔧 **`debian:bookworm-slim` for maximum compatibility**: glibc compatibility eliminates musl cross-compilation issues. The `slim` variant strips documentation and locale data, keeping the image reasonably small without the compatibility risks of Alpine.

5. 🐳 **Docker Compose for full local stack**: A single `docker compose up` starts both the application and PostgreSQL with correct networking, health checks, and persistent volume — matching the production topology without manual setup.

6. 🗄️ **Migrations run at startup**: The application binary runs `sqlx migrate run` on startup before binding the HTTP server, ensuring the database schema is always up to date on deployment.

## Trade-offs and Risks ⚠️

- 🐌 **Initial build time**: Rust compilation is slow; the first Docker build without cache can take several minutes. Layer caching in CI (via `cache-from`) mitigates this after the first run.
- 🔧 **Two build stages require coordination**: The Tailwind CLI runs in a separate stage. If the Tailwind binary distribution changes, the CSS build stage must be updated.
- 📦 **Templates and migrations are copied into the image**: These are baked into the image at build time. Runtime changes to templates require a new image build and deployment.

## Consequences

- 🐳 A `Dockerfile` at the project root defines the three-stage build (builder, css-builder, runtime).
- 🔧 A `docker-compose.yml` at the project root provides the local development stack.
- 🗄️ The application runs `sqlx migrate run` at startup before accepting HTTP traffic.
- 🔒 The container runs as a non-root user in all environments.
- 🌍 All configuration is injected via environment variables (ADR-0008); no config files are baked into the image.
- 📋 A `.dockerignore` file excludes `target/`, `.env`, `.git/`, and `node_modules/` from the build context.

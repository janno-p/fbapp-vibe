# ADR-0013: CI/CD with GitHub Actions 🔧

## Status

✅ Accepted

## Date

2026-04-05

## Context

The project is hosted on GitHub and requires an automated pipeline to validate every change and produce deployment-ready artefacts. The pipeline must enforce code quality, run tests, and build the Docker image on each push.

Key requirements:

- ✅ **Automated quality gates**: Every pull request must pass formatting, linting, and tests before merging.
- 🔒 **Security**: Dependency vulnerabilities must be surfaced automatically.
- 🐳 **Docker image build**: The production image must be built and verified on every merge to the main branch.
- ⚡ **Fast feedback**: The pipeline should use caching aggressively to minimise wait time.
- 🗄️ **Database-dependent tests**: SQLx compile-time query checking and integration tests require a running PostgreSQL instance in CI.

## Decision

We will use **GitHub Actions** 🔧 for CI/CD, with separate workflows for pull request validation and main branch deployment builds.

## Pipeline Stages

```
Pull Request
  │
  ├── fmt          cargo fmt --check
  ├── clippy       cargo clippy -- -D warnings
  ├── test         cargo test (with PostgreSQL service)
  ├── sqlx-check   cargo sqlx prepare --check
  └── audit        cargo audit

Main Branch (after PR merge)
  │
  ├── [all PR checks run again]
  └── docker       Build + push image to registry
```

## Workflow: Pull Request Checks

```yaml
# .github/workflows/ci.yml
name: CI

on:
  pull_request:
  push:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  DATABASE_URL: postgres://fbapp:fbapp@localhost:5432/fbapp
  SQLX_OFFLINE: true

jobs:
  fmt:
    name: Format 📐
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all --check

  clippy:
    name: Clippy 🔍
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    name: Test 🧪
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_USER: fbapp
          POSTGRES_PASSWORD: fbapp
          POSTGRES_DB: fbapp
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 5s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run migrations
        run: cargo sqlx migrate run
        env:
          SQLX_OFFLINE: false
      - run: cargo test --all-features

  audit:
    name: Security Audit 🔒
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

## Workflow: Docker Build and Push

```yaml
# .github/workflows/docker.yml
name: Docker

on:
  push:
    branches: [main]

jobs:
  docker:
    name: Build & Push 🐳
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build and push
        uses: docker/build-push-action@v6
        with:
          context: .
          push: true
          tags: |
            ghcr.io/${{ github.repository }}:latest
            ghcr.io/${{ github.repository }}:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

## Rationale

1. ✅ **GitHub Actions is zero-configuration for GitHub repos**: No external CI service to configure, authenticate, or pay for separately. Workflows live in `.github/workflows/` alongside the code they test.

2. ⚡ **`Swatinem/rust-cache`**: Caches the Cargo registry, compiled dependencies, and build artefacts between runs. Subsequent pipeline runs skip recompiling unchanged dependencies — the most expensive part of a Rust CI pipeline.

3. 🗄️ **PostgreSQL service container**: GitHub Actions supports Docker service containers natively. The `services.postgres` block starts a PostgreSQL instance alongside the job, enabling both SQLx migration runs and integration tests without external infrastructure.

4. 🔒 **`SQLX_OFFLINE: true` by default**: The `.sqlx/` metadata cache (from `cargo sqlx prepare`) is committed to the repository, allowing `cargo clippy` and `cargo fmt` jobs to run without a live database. Only the `test` job sets `SQLX_OFFLINE: false` and connects to the real database.

5. 🐳 **GitHub Container Registry (ghcr.io)**: Docker images are pushed to ghcr.io using the built-in `GITHUB_TOKEN` — no external registry credentials to manage. Images are tagged with both `latest` and the commit SHA for precise rollback.

6. 🔒 **`cargo audit`**: Runs `rustsec/audit-check` on every push to surface known vulnerabilities in dependencies automatically, without manual review of advisory databases.

7. 🏗️ **Docker layer cache via GitHub Actions cache**: `cache-from: type=gha` and `cache-to: type=gha,mode=max` persist Docker build layers in the GitHub Actions cache, dramatically speeding up image builds after the first run.

## Trade-offs and Risks ⚠️

- ⏱️ **Cold cache build times**: The first pipeline run without a warm cache compiles all dependencies from scratch, which can take 5–10 minutes. Subsequent runs with a warm cache are significantly faster.
- 🔒 **`GITHUB_TOKEN` permissions**: The Docker workflow requires `packages: write` permission to push to ghcr.io. This is scoped to the job and does not grant broader access.
- 🔄 **No automatic deployment**: This ADR covers CI and image building only. Deployment to a production environment (triggering a cloud service to pull the new image) requires a deployment step to be added once the hosting platform is chosen.

## Consequences

- 📁 Two workflow files are maintained: `.github/workflows/ci.yml` (PR checks) and `.github/workflows/docker.yml` (image build on main).
- 🔒 The `.sqlx/` query metadata directory is committed to the repository and kept up to date by running `cargo sqlx prepare` locally before pushing schema changes.
- 📐 `cargo fmt --check` and `cargo clippy -- -D warnings` are required to pass — no warnings are allowed to merge.
- 🔒 `cargo audit` failures block merges; dependency advisories must be resolved or explicitly ignored with a documented justification.
- 🐳 Production Docker images are published to `ghcr.io/{owner}/{repo}` and tagged with the commit SHA on every merge to `main`.
- 🔄 Branch protection rules on `main` require all CI jobs to pass before a pull request can be merged.

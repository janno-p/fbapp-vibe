# fbapp-vibe

A server-rendered web application built with Rust, Axum, HTMX, and Askama. Authentication is handled via Google OAuth.

## Tech Stack

| Layer | Choice |
|-------|--------|
| Language | Rust |
| Web framework | Axum |
| Database | PostgreSQL + SQLx |
| Templating | Askama |
| Client interactions | HTMX |
| Styling | Tailwind CSS |
| Authentication | Google OAuth 2.0 + tower-sessions |

## Prerequisites

Before you begin, make sure the following are installed:

- **Rust** — [rustup.rs](https://rustup.rs)
- **Docker** — for running PostgreSQL locally
- **Node.js** — for Tailwind CSS (`npm install` installs `tailwindcss` and `@tailwindcss/cli`)
- **sqlx-cli** — `cargo install sqlx-cli --no-default-features --features rustls,postgres`
- **cargo-watch** *(optional, for development)* — `cargo install cargo-watch`

## Getting Started

### 1. Clone the repository

```bash
git clone <repository-url>
cd fbapp-vibe
```

### 2. Set up Google OAuth credentials

1. Go to the [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project (or select an existing one)
3. Navigate to **APIs & Services → Credentials**
4. Click **Create Credentials → OAuth 2.0 Client ID**
5. Set application type to **Web application**
6. Add `http://localhost:3000/auth/callback` to **Authorised redirect URIs**
7. Copy the **Client ID** and **Client Secret**

### 3. Configure environment variables

```bash
cp .env.example .env
```

Open `.env` and fill in the required values:

```bash
DATABASE_URL=postgres://fbapp:fbapp@localhost:5432/fbapp
TEST_DATABASE_URL=postgres://fbapp:fbapp@localhost:5432/fbapp_test
HOST=127.0.0.1
PORT=3000

GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-client-secret
GOOGLE_REDIRECT_URL=http://localhost:3000/auth/callback
FOOTBALL_API_KEY=your-api-key
```

`TEST_DATABASE_URL` must point to a separate database on the same host. `cargo test` uses `sqlx::test` which creates and tears down an isolated database per test.

Optional settings are documented in `.env.example`: `TLS_CERT_PATH`, `TLS_KEY_PATH`, `SESSION_DURATION_HOURS`, `POLL_INTERVAL_SECS`, `POLL_INTERVAL_LIVE_SECS`, and `OTEL_EXPORTER_OTLP_ENDPOINT`.

### 4. Choose a development workflow

For database-only local development, run PostgreSQL in Docker and run the app on your host:

```bash
docker compose up db -d
npm install
make js
make css
cargo run
```

This starts PostgreSQL on port `5432`, vendors HTMX/Alpine into `assets/js/`, compiles Tailwind CSS, and starts the Rust application. `cargo run` applies SQLx migrations automatically at startup.

For a full Docker Compose startup, populate `.env` and run:

```bash
cp .env.example .env
# fill in Google OAuth and football API values
docker compose up --build
```

The app service reads application secrets from `.env`, while Compose overrides `DATABASE_URL`, `HOST`, and `PORT` so the container binds correctly and talks to the Compose PostgreSQL service.

### 5. Enable HTTPS (optional but recommended)

Install [`mkcert`](https://github.com/FiloSottile/mkcert) and generate a locally-trusted certificate:

```bash
# Install mkcert (once per machine)
brew install mkcert          # macOS
# choco install mkcert       # Windows
# apt install mkcert         # Debian/Ubuntu

mkcert -install              # installs local CA into the OS/browser trust store
mkdir certs
mkcert -cert-file certs/localhost.pem -key-file certs/localhost-key.pem localhost 127.0.0.1
```

Then uncomment and set `TLS_CERT_PATH` and `TLS_KEY_PATH` in `.env`:

```bash
TLS_CERT_PATH=certs/localhost.pem
TLS_KEY_PATH=certs/localhost-key.pem
```

When both variables are set, the server starts on `https://localhost:3000`. If they are unset, it falls back to plain HTTP. Docker Compose mounts local `./certs` into the app container at `/app/certs`, so the same `certs/...` paths work for full Compose startup without baking certificates into the image.

The application starts at [http://localhost:3000](http://localhost:3000) or [https://localhost:3000](https://localhost:3000) when TLS is configured.

- `GET /` — public landing page with Google sign-in
- `GET /dashboard` — authenticated landing page
- `GET /health` — health check endpoint
- `GET /predictions` — tournament predictions page (authenticated)
- `GET /admin` — admin dashboard (admin users only)
- `GET /leagues/join/:token` — league invite link

---

## Development Workflow

Run the application and Tailwind watcher concurrently:

```bash
make dev
```

This starts `cargo watch` (recompiles on file changes) and `tailwindcss --watch` (recompiles CSS on template changes) side by side.

## Make Targets

| Target | Description |
|--------|-------------|
| `make dev` | Start app + CSS watcher concurrently |
| `make build` | Build release binary |
| `make lint` | Run `cargo fmt --check` and `cargo clippy` |
| `make test` | Run test suite |
| `make migrate` | Run pending database migrations |
| `make css` | Compile Tailwind CSS once |
| `make js` | Vendor HTMX and Alpine from `node_modules/` into `assets/js/` |

## Project Structure

```
fbapp-vibe/
├── migrations/          # SQLx migrations (versioned SQL files)
├── templates/           # Askama HTML templates
│   ├── layout/          # Base layout
│   └── {module}/        # Per-feature templates
├── Dockerfile           # Multi-stage app image build
├── .dockerignore        # Docker build-context exclusions
├── assets/css/          # Tailwind v4 CSS-first source and compiled output
├── assets/js/           # Vendored HTMX/Alpine and local JavaScript assets
├── tests/               # Integration tests (HTTP-level, use axum-test)
├── src/
│   ├── main.rs          # Entry point — startup, server bind, layer wiring
│   ├── lib.rs           # Crate root — re-exports modules for integration tests
│   ├── config.rs        # Configuration loaded from environment variables
│   ├── football_api.rs  # football-data.org client and rate limiting
│   ├── error.rs         # Global AppError type
│   ├── state.rs         # Shared AppState (database pool, config, OAuth client)
│   ├── routes.rs        # Top-level router assembly
│   ├── db_types.rs      # Shared database enums (MatchOutcome, KnockoutRound)
│   ├── extractors.rs    # Custom Axum extractors (QsForm)
│   ├── polling/         # Background football API polling jobs
│   ├── session_cleanup.rs # Expired session cleanup task
│   ├── tracing_setup.rs # Tracing and optional OTLP setup
│   └── modules/         # Feature modules (each exposes a single router())
│       ├── auth/        # Google OAuth, session management, landing pages
│       ├── admin/       # Tournament management, seeding, league admin
│       ├── leagues/     # League creation, invite links, membership
│       ├── predictions/ # Tournament predictions (group, knockout, top scorer)
│       └── standings/   # League standings, live match views, comparisons
├── docs/adr/            # Architecture Decision Records
└── thoughts/
    ├── plans/           # Implementation plans
    └── tickets/         # Task tickets for agentic development
```

## Architecture Decisions

Key decisions are documented as Architecture Decision Records in [`docs/adr/`](docs/adr/). Start with [ADR-0001](docs/adr/0001-use-rust-as-programming-language.md) for an overview of the technology choices.

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

Open `.env` and fill in the values:

```bash
DATABASE_URL=postgres://fbapp:fbapp@localhost:5432/fbapp
HOST=127.0.0.1
PORT=3000

GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-client-secret
GOOGLE_REDIRECT_URL=http://localhost:3000/auth/callback
SESSION_SECRET=<generate-a-random-value-see-below>
```

To generate a secure `SESSION_SECRET`:

```bash
openssl rand -base64 64
```

### 4. Start PostgreSQL

```bash
docker compose up db -d
```

This starts a PostgreSQL instance on port `5432` with the credentials from `docker-compose.yml`.

### 5. Run database migrations

```bash
make migrate
```

This creates the `users` and `tower_sessions` tables.

### 6. Enable HTTPS (optional but recommended)

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

When both variables are set, the server starts on `https://localhost:3000`. If they are unset, it falls back to plain HTTP.

### 7. Install Node dependencies

```bash
npm install
```

### 8. Build Tailwind CSS

```bash
make css
```

This compiles `assets/css/input.css` → `assets/css/main.css`.

### 9. Start the application

```bash
cargo run
```

The application starts at [http://localhost:3000](http://localhost:3000).

- `GET /` — public landing page with Google sign-in
- `GET /dashboard` — authenticated landing page
- `GET /health` — health check endpoint

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

## Project Structure

```
fbapp-vibe/
├── migrations/          # SQLx migrations (versioned SQL files)
├── templates/           # Askama HTML templates
│   ├── layout/          # Base layout
│   └── {module}/        # Per-feature templates
├── assets/css/          # Tailwind CSS
├── src/
│   ├── main.rs          # Entry point — startup, server bind, layer wiring
│   ├── config.rs        # Configuration loaded from environment variables
│   ├── error.rs         # Global AppError type
│   ├── state.rs         # Shared AppState (database pool, config, OAuth client)
│   ├── routes.rs        # Top-level router assembly
│   └── modules/         # Feature modules (each exposes a single router())
│       └── auth/        # Google OAuth, session management, landing pages
├── docs/adr/            # Architecture Decision Records
└── tasks/               # Task files for AI-assisted development
```

## Architecture Decisions

Key decisions are documented as Architecture Decision Records in [`docs/adr/`](docs/adr/). Start with [ADR-0001](docs/adr/0001-use-rust-as-programming-language.md) for an overview of the technology choices.

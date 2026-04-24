## Validation Report: Project Scaffold Closeout Implementation Plan

### Implementation Status

✓ Phase 1: Docker Packaging - Fully implemented
✓ Phase 2: Compose Runtime Wiring - Fully implemented
✓ Phase 3: Documentation Alignment - Fully implemented
✓ Phase 4: ADR Drift Notes - Fully implemented for the ADRs named in the plan
✓ Phase 5: Verification Closeout - Fully implemented

### Expected Changes Reviewed

- `Dockerfile` added with Rust builder, npm asset builder, slim Debian runtime, copied `assets/`, `templates/`, and `migrations/`, non-root user, exposed port `3000`, and `./fbapp-vibe` command.
- `.dockerignore` added with exclusions for build outputs, local secrets, certs, git metadata, agent workspaces, and editor/runtime files while preserving lockfiles.
- `docker-compose.yml` updated with `env_file: .env`, container networking overrides for `DATABASE_URL`, `HOST`, and `PORT`, database health dependency, and runtime TLS cert mount.
- `README.md` updated for current `Config`, local and full Compose workflows, `make js`, Tailwind v4, vendored JS, current shared modules, and `standings`.
- `.env.example` updated with required app variables, test-only database note, optional TLS/polling/session/OTLP settings, and Compose behavior notes.
- `docs/adr/0006-use-tailwind-css-for-styling.md`, `docs/adr/0007-project-structure-modular-monolith.md`, and `docs/adr/0012-deployment-with-docker.md` amended to describe current Tailwind v4, module layout, vendored JS, lockfile-based Docker asset build, and Compose secret handling.
- `thoughts/tickets/project-scaffold.md` contains the closeout outcome and verification notes.
- No database migration was expected, and no scaffold-closeout migration was added.

### Automated Verification Results

✓ Formatting and clippy pass: `make lint`
✓ Rust release build passes: `make build`
✓ Rust tests pass: `make test` with 122 passed, 1 ignored, plus route integration tests passing
✓ CSS builds: `make css`
✓ JS vendoring runs: `make js`
✓ Compose config validates: `docker compose config`
✓ Database-only workflow starts: `docker compose up db -d`
✓ App image builds: `docker compose build app`
✓ Full app stack starts: `docker compose up app -d --build`
✓ Container health endpoint returns `200`: `curl -k https://localhost:3000/health`
✓ Container static assets return `200`: `/assets/css/main.css`, `/assets/js/htmx.js`, `/assets/js/alpine.js`
✓ Runtime image content check passed: binary, `assets/`, `templates/`, and `migrations/` present; `.env`, `node_modules/`, `certs/`, and `.git` absent; container user is non-root (`uid 999`)
✓ Working tree remained clean after verification: `git status --short`

### Code Review Findings

#### Matches Plan

- `Dockerfile` implements the required multi-stage structure and preserves the runtime artifacts the app needs at startup.
- The SQLx build-time database deviation documented in the plan is justified because the repo uses SQLx compile-time query validation without a committed `.sqlx` cache.
- `.dockerignore` excludes the minimum required sensitive/heavy paths and does not exclude `Cargo.lock` or `package-lock.json`.
- Compose keeps container-specific database/bind settings in `environment` while sourcing app secrets through `.env`.
- The TLS cert mount deviation documented in the plan is justified because certs are excluded from image build context but optional TLS paths may be present in `.env`.
- README environment documentation matches `src/config.rs` plus the explicitly test-only `TEST_DATABASE_URL`.
- README lists all current Make targets from `Makefile` and includes `make js` in the local setup flow.
- README project structure includes `assets/js/`, `src/football_api.rs`, `src/polling/`, `src/session_cleanup.rs`, `src/tracing_setup.rs`, `src/modules/standings/`, `Dockerfile`, and `.dockerignore`.
- ADR-0006 clearly amends the old Tailwind v3-era guidance with Tailwind v4 CSS-first setup.
- ADR-0007 documents the current registered modules and shared app files.
- ADR-0012 documents the current Dockerfile pattern, npm reproducibility, Tailwind v4 build command, vendored JS, `.env` secret handling, and TLS cert mount behavior.

#### Deviations from Plan

- **Phase 1**: The Docker builder stage starts a temporary PostgreSQL instance and applies migrations before `cargo build --release`.
- **Assessment**: Justified and documented in the plan's `## Deviations from Plan`; required for SQLx compile-time query checks without baking real secrets into the image.
- **Recommendation**: No follow-up required unless the project later commits `.sqlx` offline metadata and wants faster Docker builds.
- **Phase 2**: Compose mounts `./certs:/app/certs:ro`.
- **Assessment**: Justified and documented in the plan's `## Deviations from Plan`; supports optional TLS without copying certs into the image.
- **Recommendation**: No follow-up required.

#### Additional Deviations Found During Review

- None affecting the named success criteria.

### Potential Issues

- `docker compose config` expands values from `.env` into command output. The validation output was treated as sensitive and is not reproduced here. Recommendation: avoid publishing raw `docker compose config` output when real local secrets are present, or use placeholder `.env` values for shareable logs.
- ADR-0011 and ADR-0015 still contain historical references to `SESSION_SECRET`, outside the ADRs targeted by this plan. This does not fail the scaffold closeout plan, but it is a residual documentation drift risk if developers use those ADRs as current auth configuration guidance.

### Manual Testing Required

1. UI functionality:
   - [x] Verify `/health` responds from the containerized app.
   - [x] Verify container serves `/assets/css/main.css`.
   - [x] Verify container serves `/assets/js/htmx.js`.
   - [x] Verify container serves `/assets/js/alpine.js`.

2. Integration:
   - [x] Confirm full Compose startup works with the current local `.env`.
   - [x] Confirm database-only Compose workflow remains usable.
   - [x] Confirm runtime image excludes secrets, certs, git metadata, and `node_modules`.

### Edge Case Assessment

- Missing required app env values should still fail through `Config::load()` because `google_client_id`, `google_client_secret`, `google_redirect_url`, and `football_api_key` are non-optional in `src/config.rs`.
- Runtime migrations remain available because `migrations/` is copied into the image and startup still runs migrations through the existing app flow.
- Static assets are built in the image and copied into runtime, preventing the common failure mode where Askama templates render links to missing CSS/JS files.
- Runtime image does not contain local `.env`, `certs/`, `.git`, `node_modules`, Rust toolchain, or Node tooling.

### Recommendations

- Consider a small follow-up documentation cleanup for stale `SESSION_SECRET` references in ADR-0011 and ADR-0015.
- Consider documenting that raw `docker compose config` output can include secrets when `env_file: .env` is used.

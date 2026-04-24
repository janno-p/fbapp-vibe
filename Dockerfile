FROM rust:1-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        cmake \
        libssl-dev \
        perl \
        postgresql \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
COPY templates ./templates
RUN pg_ctlcluster 15 main start \
    && su postgres -c "psql -v ON_ERROR_STOP=1 -c \"CREATE USER fbapp WITH PASSWORD 'fbapp';\"" \
    && su postgres -c "createdb -O fbapp fbapp_build" \
    && for migration in migrations/*.sql; do psql "postgres://fbapp:fbapp@localhost:5432/fbapp_build" -v ON_ERROR_STOP=1 -f "$migration"; done \
    && DATABASE_URL="postgres://fbapp:fbapp@localhost:5432/fbapp_build" cargo build --release

FROM node:20-slim AS assets

WORKDIR /app

COPY package.json package-lock.json ./
COPY assets ./assets
COPY src ./src
COPY templates ./templates
RUN npm ci
RUN cp node_modules/htmx.org/dist/htmx.min.js assets/js/htmx.js \
    && cp node_modules/alpinejs/dist/cdn.min.js assets/js/alpine.js \
    && npx @tailwindcss/cli -i assets/css/input.css -o assets/css/main.css --minify

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --shell /usr/sbin/nologin appuser

COPY --from=builder /app/target/release/fbapp-vibe ./fbapp-vibe
COPY --from=assets /app/assets ./assets
COPY templates ./templates
COPY migrations ./migrations

USER appuser

EXPOSE 3000
CMD ["./fbapp-vibe"]

.PHONY: dev build lint test migrate css js

## Start app and CSS watcher concurrently
dev:
	cargo watch -x run & npx @tailwindcss/cli -i assets/css/input.css -o assets/css/main.css --watch

## Build release binary
build:
	cargo build --release

## Check formatting and run clippy
lint:
	cargo fmt --check && cargo clippy -- -D warnings

## Run tests
test:
	cargo test

## Run database migrations
migrate:
	cargo sqlx migrate run

## Compile Tailwind CSS once
css:
	npx @tailwindcss/cli -i assets/css/input.css -o assets/css/main.css

js:
	cp node_modules/htmx.org/dist/htmx.min.js assets/js/htmx.js
	cp node_modules/alpinejs/dist/cdn.min.js assets/js/alpine.js

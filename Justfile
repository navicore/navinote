ci: fmt-check lint test lint-pwa

fmt-check:
    cargo fmt --check

lint:
    cargo clippy -- -D warnings

test:
    cargo test

lint-pwa:
    cd pwa && SVELTE_STRICT=1 npm run build

build: build-pwa build-server build-sync

build-pwa:
    cd pwa && npm ci && npm run build

build-server:
    cd server && cargo build --release

build-sync:
    cd sync && cargo build --release

dev-pwa:
    cd pwa && npm run dev

dev-server:
    cd server && cargo run

docker-build:
    docker build -t navinote .

install:
    cd sync && cargo install --path .

# Run all CI checks (same as GitHub Actions!)
# This is what developers should run before pushing.
ci: fmt-check lint test build
    @echo "Safe to push to GitHub - CI will pass."

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint: lint-rust lint-pwa

lint-rust:
    cargo clippy --locked --workspace --all-targets -- -D warnings

lint-pwa:
    cd pwa && npm ci && SVELTE_STRICT=1 npm run build

test:
    cargo test --locked --workspace --all-targets

build: build-pwa build-server build-sync

build-pwa:
    cd pwa && npm ci && npm run build

build-server:
    cd server && cargo build --locked --release

build-sync:
    cd sync && cargo build --locked --release

dev-pwa:
    cd pwa && npm run dev

dev-server:
    cd server && cargo run

docker-build:
    docker build -t navinote .

install:
    cd sync && cargo install --locked --path .

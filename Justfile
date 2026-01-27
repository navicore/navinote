build: build-pwa build-server build-sync

build-pwa:
    cd pwa && npm run build

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

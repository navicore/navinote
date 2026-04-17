FROM node:22-slim AS pwa-build
WORKDIR /app/pwa
COPY pwa/package.json pwa/package-lock.json* ./
RUN npm ci
COPY pwa/ ./
RUN npm run build

FROM rust:1.93-bookworm AS rust-build
WORKDIR /app
COPY rust-toolchain.toml ./
COPY Cargo.toml Cargo.lock ./
COPY server/ server/
COPY sync/ sync/
RUN cargo build --locked --release -p navinote-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd -m appuser
USER appuser
WORKDIR /app
COPY --from=rust-build /app/target/release/navinote-server ./
COPY --from=pwa-build /app/pwa/dist ./dist
ENV NAVINOTE_STATIC_DIR=dist
ENV NAVINOTE_DB_PATH=navinote.db
EXPOSE 8080
CMD ["./navinote-server"]

# navinote

A self-hosted quick-capture note and reminder app.

## Architecture

```
┌─────────────────┐            ┌─────────────────┐
│  PWA (phone)    │───sync────▶│  Axum Server    │
│  Svelte         │◀───────────│  SQLite         │
│  Offline-first  │            │  Serves PWA     │
└─────────────────┘            └────────┬────────┘
                                        │
                               cron pull│
                                        ▼
                               ┌─────────────────┐
                               │  Laptop         │
                               │  sync CLI (Rust)│
                               │  → zet markdown │
                               └─────────────────┘
```

## Components

### 1. Server (`server/`)

Rust, Axum, SQLite (via sqlx or rusqlite).

**Responsibilities:**
- REST API for notes CRUD
- Serves the built Svelte PWA as static files
- SQLite database for note storage
- OIDC authentication via Kanidm (validate tokens)

**API:**
```
GET    /api/notes              # list notes, ?synced=false for unsynced only
POST   /api/notes              # create note
PUT    /api/notes/:id          # update note
DELETE /api/notes/:id          # delete note
PATCH  /api/notes/:id/synced   # mark as synced
GET    /api/health             # health check for k8s probes
```

**Note schema:**
```sql
CREATE TABLE notes (
    id          TEXT PRIMARY KEY,  -- uuid
    text        TEXT NOT NULL,
    remind_at   TEXT,              -- optional ISO8601 datetime
    created_at  TEXT NOT NULL,     -- ISO8601 datetime
    updated_at  TEXT NOT NULL,     -- ISO8601 datetime
    synced      INTEGER NOT NULL DEFAULT 0
);
```

### 2. PWA (`pwa/`)

Svelte, minimal. Optimized for fast mobile capture.

**Features:**
- Text input (always visible, ready to type)
- Optional "remind me" datetime picker
- Save to local IndexedDB (offline-first)
- Sync to server when online (manual sync button + auto on save when connected)
- Service worker for offline access and caching
- Installable as PWA (manifest.json)

**UX flow:**
1. Open app → cursor in text field
2. Type note
3. Optionally tap reminder toggle, pick date/time
4. Tap save → stored locally, synced to server if online
5. Note list below input shows recent notes

### 3. Sync CLI (`sync/`)

Rust CLI tool, run via cron on laptop.

**Responsibilities:**
- Pull unsynced notes from server API
- For each note, append to `~/git/navicore/zet/YYYY-MM-DD.md` (based on created_at date)
- Create the daily file with frontmatter if it doesn't exist
- Mark notes as synced on the server after writing

**Output format (reminder):**
```markdown
* [ ] #reminder 2026-01-28T21:00:00Z: taxes (via quicknote 2026-01-26T15:30:00Z)
```

**Output format (plain note):**
```markdown
* taxes are due next week (via quicknote 2026-01-26T15:30:00Z)
```

**Zet file format (created if missing):**
```markdown
---
title: Monday, January 26th, 2026
---

```

### 4. Docker & CI

**Dockerfile** (multi-stage, following gamecode-web pattern):
- Stage 1: Node - build Svelte PWA
- Stage 2: Rust - build Axum server
- Stage 3: debian:bookworm-slim runtime, copy server binary + built PWA dist
- Non-root user, expose port 8080

**GitHub Actions** (`.github/workflows/docker-build.yml`):
- Trigger on push to main + tags `v*` + PRs to main
- Build linux/arm64 (homelab k8s is ARM)
- Push to ghcr.io/navicore/navinote
- Uses GHCR_TOKEN secret
- GHA cache for layers

**K8s** (`k8s/`):
- Deployment + Service (ClusterIP)
- SQLite stored via PersistentVolumeClaim (notes must survive pod restarts)
- Health/readiness probes on `/api/health`
- Resource limits similar to gamecode

### 5. Justfile

Build system orchestration:
```just
# Build everything
build: build-pwa build-server

# PWA
build-pwa:
    cd pwa && npm run build

dev-pwa:
    cd pwa && npm run dev

# Server
build-server:
    cd server && cargo build --release

dev-server:
    cd server && cargo run

# Sync CLI
build-sync:
    cd sync && cargo build --release

# Docker
docker-build:
    docker build -t navinote .

# Dev: run server + pwa together
dev: dev-pwa dev-server
```

## Project Structure

```
navinote/
├── PLAN.md
├── Justfile
├── Dockerfile
├── .github/
│   └── workflows/
│       └── docker-build.yml
├── k8s/
│   └── navinote-deployment.yaml
├── server/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── pwa/
│   ├── package.json
│   ├── src/
│   │   ├── App.svelte
│   │   ├── main.js
│   │   └── lib/
│   │       ├── db.js          # IndexedDB wrapper
│   │       └── sync.js        # server sync logic
│   ├── public/
│   │   ├── manifest.json
│   │   └── sw.js              # service worker
│   └── vite.config.js
└── sync/
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## Auth: Kanidm (OIDC)

Kanidm runs as a separate service on the homelab k8s cluster. All apps
(navinote and future) authenticate against it via OIDC.

- **Kanidm** issues OIDC tokens (MPL-2.0 licensed, Rust-based)
- **navinote server** validates OIDC tokens on API requests
- **PWA** redirects to Kanidm login, receives token, stores it
- **Sync CLI** uses a service account or long-lived token from Kanidm

This is a separate deployment concern - navinote just needs to be an OIDC
relying party. Kanidm setup is its own project.

## Decided

- **SQLite library:** sqlx (async, migrations)
- **Svelte version:** Svelte 5 (GA since Oct 2024)
- **Sync CLI config:** Env vars: `NAVINOTE_URL`, `NAVINOTE_TOKEN` (Kanidm service account API token), `NAVINOTE_ZET_DIR`

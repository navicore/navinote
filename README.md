[![Docker Build](https://github.com/navicore/navinote/actions/workflows/docker-build.yml/badge.svg)](https://github.com/navicore/navinote/actions/workflows/docker-build.yml)

# navinote

A personal note-taking PWA with reminders and markdown sync.

## Components

- **pwa/** - Svelte 5 progressive web app
- **server/** - Axum REST API with SQLite
- **sync/** - CLI tool to sync notes to markdown files

## Quick Start

```bash
# Install dependencies
cd pwa && npm install

# Build everything
just build

# Run locally
just dev-server   # API on :8080
just dev-pwa      # Vite dev server
```

## CLI Sync

Install the sync tool:

```bash
just install
```

Configure and run:

```bash
export NAVINOTE_TOKEN="your-token"
export NAVINOTE_ZET_DIR="$HOME/notes"
# NAVINOTE_URL defaults to https://notes.navicore.tech

navinote-sync
```

Synced notes are appended to daily markdown files (`YYYY-MM-DD.md`) with reminders formatted as:

```markdown
* [ ] #reminder 2026-01-30T14:00:00Z: Call dentist
* [x] #reminder 2026-01-30T09:00:00Z: Morning standup
```

## Features

- Offline-first with IndexedDB
- Swipe left to delete, swipe right to mark done
- Reminder color coding (green=future, orange=overdue, dimmed=done)
- iOS/Android install prompts

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| NAVINOTE_TOKEN | Yes | - | API bearer token |
| NAVINOTE_DB_PATH | No | navinote.db | SQLite database path |
| NAVINOTE_PORT | No | 8080 | Server port |
| NAVINOTE_STATIC_DIR | No | dist | Static files directory |
| NAVINOTE_URL | No | https://notes.navicore.tech | API URL (sync CLI) |
| NAVINOTE_ZET_DIR | Yes | - | Markdown output directory (sync CLI) |

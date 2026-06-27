# OVPSPEE Filing & Tracking System

Centralized digital filing cabinet for OVPSPEE — University of Eastern Philippines.

**Stack**: Tauri v2 · Rust · React 19 · TypeScript · SQLite · Tailwind CSS

## Quick start

```bash
pnpm install
pnpm dev          # Start Tauri dev server
pnpm test         # Run frontend tests
pnpm verify       # Typecheck + build + test
cd src-tauri && cargo test  # Run Rust backend tests
cargo tauri build  # Build Windows installer (MSI/NSIS)
```

See [docs/00_Index.md](docs/00_Index.md) for full documentation (design system, DB schema, API reference, deployment).

## Architecture

```
src/              # React + TypeScript frontend
src-tauri/src/    # Rust backend (IPC commands, auth, DB, file I/O, PDF)
src-tauri/migrations/  # SQLite schema migrations
mobile/           # Android mobile submission app
docs/             # Complete Developer Handoff Pack (14 documents)
```

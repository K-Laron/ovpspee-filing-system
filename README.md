# OVPSPEE Filing & Tracking System

Centralized digital filing cabinet for OVPSPEE — University of Eastern Philippines.

**Stack**: Tauri v2 · Rust · React 19 · TypeScript · SQLite · Tailwind CSS

## Quick start

```bash
pnpm install
pnpm dev          # Start Tauri dev server
pnpm test         # Run frontend tests
pnpm lint         # ESLint check
pnpm format:check # Prettier formatting check
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
plans/            # Implementation plans (batch 1 + batch 2, all executed)
```

## Key Frontend Features

- **Toast notifications**: Global, stacked, auto-dismiss, color-coded
- **Breadcrumbs**: Navigation path on all sub-pages
- **Loading skeletons**: Animated placeholders replace "Loading..." text
- **Shared form components**: Extracted to `src/components/forms/` for consistency
- **Inline field validation**: Per-field error messages below inputs
- **Required field indicators**: Red asterisk on required labels
- **Pagination**: Offset-based "Load More" with total count
- **Search filters**: Client-side text search on trash and document dropdown
- **ConfirmDialog requiredText**: Type-to-confirm for destructive actions

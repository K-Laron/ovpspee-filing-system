# Plan 002: Unified verify script, README, and doc fixes

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. When done, update the status row in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 9b4b638..HEAD -- package.json docs/ README.md`
> If any in-scope file changed, compare excerpts before proceeding.

## Status

- **Priority**: P1 (prerequisite for plans 004, 011, 012)
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: dx, docs
- **Planned at**: commit `9b4b638`, 2026-06-27

## Why this matters

Three small but high-signal improvements: a one-command verification script so executors (and humans) can confirm the whole codebase works, a root README so the repo has a landing page, and fixing doc references to files that were deleted in a prior cleanup.

## Current state

### Fix 1 — No unified verification command
- `package.json` scripts include `test` (vitest run) and `build` (tsc + vite build).
- **No** `test:all`, `verify`, or `ci` script.
- Rust tests require a separate `cargo test --manifest-path src-tauri/Cargo.toml`.
- No lint step exists (ESLint not configured — see plan 012).

### Fix 2 — Missing root README
- No `README.md` at repo root.
- Onboarding entry point is `docs/00_Index.md` — non-standard, invisible to GitHub.
- `.env.example` exists at root but is undocumented.

### Fix 3 — Docs reference deleted files
- `docs/07_Developer_Guidelines.md:224,268` — references `src/lib/confirm.ts` (useConfirmAction) and `src/lib/invoke.ts` (typed wrapper). Both deleted in commit 9b4b638.
- `docs/04_Frontend_Component_Documentation.md:747` — references `src/lib/invoke.ts`.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Typecheck | `pnpm tsc --noEmit` | exit 0 |
| Frontend tests | `pnpm test` | all pass |
| Rust tests | `cargo test --manifest-path src-tauri/Cargo.toml` | all pass |

## Scope

**In scope**:
- `package.json`
- `README.md` (create)
- `docs/07_Developer_Guidelines.md`
- `docs/04_Frontend_Component_Documentation.md`

**Out of scope**:
- ESLint configuration (handled in plan 012)
- CI configuration (handled in plan 012)
- Any source code changes

## Steps

### Step 1: Add verify script to package.json

Add to the `"scripts"` section in `package.json`:

```json
"verify": "pnpm build && pnpm test"
```

`pnpm build` already runs `tsc --noEmit && vite build`, so this covers typecheck + build + frontend tests. Rust tests are not included since they require a different toolchain — document this in README.

**Verify**: `pnpm verify` — exit 0. Expect typecheck + build to pass, then vitest tests pass.

### Step 2: Create README.md

Create `README.md` at repo root with:

```markdown
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
```

**Verify**: `cat README.md` — content renders correctly.

### Step 3: Fix docs/07_Developer_Guidelines.md

In `docs/07_Developer_Guidelines.md`:

1. Remove or update section 7 ("Shared Frontend Utilities") that references `src/lib/confirm.ts` (line ~224). The `useConfirmAction` hook was deleted. Replace with: "Confirmation dialogs use inline `useState<ConfirmAction | null>` — see `src/components/ConfirmDialog.tsx` for the `ConfirmAction` interface."

2. Remove step 8 from "Adding a New Tauri Command — Checklist" (line ~268): "Write a typed wrapper in `src/lib/invoke.ts`". The `cmd` wrapper was removed; commands now use `invoke` directly from `@tauri-apps/api/core`.

3. Remove/update the `pnpm eslint src/` reference in section 2 (Code Style, line ~87). ESLint is not configured. Replace with: "Run `pnpm tsc --noEmit` and `pnpm test` before every commit."

4. Remove the doc comment requirement (line ~55) about "All public functions must have doc comments" — this is aspirational but not followed anywhere. Replace with "Add doc comments (`///`) to public API functions in auth.rs, documents.rs, and backup.rs as a starting goal."

**Verify**: `grep -c "invoke.ts" docs/07_Developer_Guidelines.md` — returns 0.

### Step 4: Fix docs/04_Frontend_Component_Documentation.md

In `docs/04_Frontend_Component_Documentation.md`, find the reference to `src/lib/invoke.ts` (line ~747) and update it:

- Old: "All `invoke()` calls must go through typed wrapper functions in `src/lib/invoke.ts`"
- New: "All Tauri IPC calls use `{ invoke }` from `@tauri-apps/api/core` directly. Import at the top of the calling file: `import { invoke } from '@tauri-apps/api/core'`"

**Verify**: `grep -c "invoke.ts" docs/04_Frontend_Component_Documentation.md` — returns 0.

## Test plan

- No new tests. Verification is: `pnpm verify`, `cargo test`, and grep checks for deleted file references.

## Done criteria

- [ ] `pnpm verify` exits 0
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` all pass
- [ ] `README.md` exists at repo root with setup commands
- [ ] `grep -rn "invoke.ts\|confirm.ts" docs/` returns 0
- [ ] `plans/README.md` status row updated

## STOP conditions

- If `pnpm verify` fails, fix the underlying issue before proceeding.
- If any doc file doesn't have the described content structure, read the full file to locate the right lines.

## Maintenance notes

- The verify script intentionally excludes Rust tests (different toolchain). If CI is added (plan 012), include both.
- README must be kept in sync if the build/test commands change.

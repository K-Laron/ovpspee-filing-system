# Plan 012: ESLint, formatter, and CI infrastructure

> **Executor instructions**: Follow step by step. When done, update `plans/README.md`.
>
> **Drift check**: `git diff --stat 9b4b638..HEAD -- package.json src/ src-tauri/`

## Status

- **Priority**: P3
- **Effort**: M
- **Risk**: LOW
- **Depends on**: 002 (README/verify script — CI references these commands)
- **Category**: dx, tooling
- **Planned at**: commit `9b4b638`, 2026-06-27

## Why this matters

The repo has no ESLint, no formatter, no EditorConfig, and no CI. The dev guidelines tell users to run `pnpm eslint src/` but the tool isn't configured. PR diffs contain formatting noise. Breaking changes compile locally but aren't caught on push. This plan sets up the tooling baseline: ESLint for TypeScript, Prettier for formatting (both frontend and Rust TOML), and a GitHub Actions CI workflow that runs typecheck, lint, test, and format check.

## Current state

- No `.eslintrc*` config files exist anywhere
- No `.prettierrc*` or `prettier.config.*` files exist
- No `.editorconfig` file exists
- No `.github/` directory exists (no CI)
- `docs/07_Developer_Guidelines.md:87` says "Run `pnpm eslint src/`" — command doesn't exist
- Rust uses `cargo fmt` for formatting (standard) — not automated

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Install | `pnpm install` | exit 0 |
| ESLint | `pnpm eslint src/` | exit 0 (no errors) |
| Prettier | `pnpm prettier --check src/` | exit 0 |
| TS typecheck | `pnpm tsc --noEmit` | exit 0 |
| Test | `pnpm test` | all pass |
| Verify | `pnpm verify` | exit 0 |

## Scope

**In scope**:
- `eslint.config.js` (create, flat config for ESLint 9+)
- `.prettierrc` (create)
- `.prettierignore` (create)
- `.editorconfig` (create)
- `.github/workflows/ci.yml` (create)
- `package.json` — add scripts: `lint`, `format:check`, `format:fix`

**Out of scope**:
- Fixing all existing ESLint violations (may be many — just get the config right and fix blocking errors)
- Rust clippy configuration (use existing `cargo clippy -- -D warnings`)
- Pre-commit hooks (add later — CI is the first gate)

## Steps

### Step 1: Install ESLint and Prettier

```bash
pnpm add -D eslint @eslint/js typescript-eslint prettier
```

ESLint 9 uses flat config. Configure for TypeScript + React with the new config format:

Create `eslint.config.js`:

```javascript
import js from '@eslint/js';
import tseslint from 'typescript-eslint';

export default [
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['src/**/*.{ts,tsx}'],
    rules: {
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'warn',
      'no-console': 'warn',
    },
  },
  {
    ignores: ['dist/', 'node_modules/', 'src-tauri/', 'mobile/', 'public/'],
  },
];
```

**Verify**: `pnpm eslint src/` — exits 0 (or shows expected warnings, not errors).

### Step 2: Create Prettier config

Create `.prettierrc`:

```json
{
  "semi": true,
  "singleQuote": true,
  "trailingComma": "all",
  "tabWidth": 2,
  "printWidth": 100,
  "arrowParens": "always"
}
```

Create `.prettierignore`:

```
dist/
node_modules/
src-tauri/
mobile/
public/
pnpm-lock.yaml
```

**Verify**: `pnpm prettier --check src/` — exits 0. If the check fails, run `pnpm prettier --write src/` once to format all files.

### Step 3: Create .editorconfig

Create `.editorconfig`:

```ini
root = true

[*]
indent_style = space
indent_size = 2
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true

[*.{rs,md}]
indent_size = 4
```

### Step 4: Add lint and format scripts to package.json

```json
"scripts": {
  // ... existing scripts ...
  "lint": "eslint src/",
  "format:check": "prettier --check src/",
  "format:fix": "prettier --write src/"
}
```

**Verify**: `pnpm lint` and `pnpm format:check` both exit 0.

### Step 5: Create GitHub Actions CI workflow

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  frontend:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'pnpm'
      - run: pnpm install
      - run: pnpm lint
      - run: pnpm format:check
      - run: pnpm build
      - run: pnpm test

  backend:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --manifest-path src-tauri/Cargo.toml
      - run: cargo test --manifest-path src-tauri/Cargo.toml
      - run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
      - run: cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

Note: The CI uses `windows-latest` since the primary target is Windows. Tauri build is not included in CI (requires native libraries) — that's a manual release step.

**Verify**: The YAML is valid. Push to GitHub to verify CI runs (or validate with `act` locally if available).

### Step 6: Fix initial ESLint violations

Run `pnpm lint` and fix any errors it reports. Expected issues:
- `no-unused-vars` — some imports or variables may be unused
- `no-explicit-any` — `any` types that should be `unknown`
- `no-console` — `console.log` calls

Fix each error:
- Unused vars: remove the import or prefix with `_`
- Any types: replace with `unknown` and narrow, or add specific type
- Console calls: replace with structured approach or suppress with comment

**Verify**: `pnpm lint` exits 0.

## Test plan

- `pnpm test` all pass (unchanged — only config/tooling changes)
- `pnpm lint` exits 0
- `pnpm format:check` exits 0
- CI workflow syntax is valid YAML

## Done criteria

- [ ] `eslint.config.js`, `.prettierrc`, `.prettierignore`, `.editorconfig` exist
- [ ] `.github/workflows/ci.yml` exists with frontend and backend jobs
- [ ] `pnpm lint` exits 0
- [ ] `pnpm format:check` exits 0
- [ ] `pnpm test` all pass
- [ ] `pnpm build` exits 0
- [ ] `docs/07_Developer_Guidelines.md` no longer references `pnpm eslint src/` as a broken command (fixed in plan 002)
- [ ] `plans/README.md` status row updated

## STOP conditions

- If `typescript-eslint` has compatibility issues with ESLint 9 flat config, use the legacy `.eslintrc` format instead: `pnpm add -D eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin eslint-plugin-react-hooks eslint-plugin-react` and create `.eslintrc.json`.
- If fixing all ESLint violations is too many changes (> 20 files), create a `.eslintignore` for the most problematic files and fix them individually in follow-up PRs. The CI should still pass even with warnings — only errors block.
- If Prettier reformats too aggressively, use simpler rules or match the existing code style more closely. The singleQuote and trailingComma settings are the most likely to cause churn.

## Maintenance notes

- The CI workflow cannot run `cargo tauri build` (requires Windows SDK and native dependencies). Release builds are manual.
- Rust formatting is checked with `cargo fmt` in the backend job. Prettier handles only the frontend.
- When adding new ESLint rules, run `pnpm lint` to check they don't introduce new errors across the existing codebase.
- The `format:fix` script is available for one-shot reformatting, but CI uses `format:check` (read-only) so formatting changes are intentional via `format:fix`.

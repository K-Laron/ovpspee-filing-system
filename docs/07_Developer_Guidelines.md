# Developer Guidelines
## OVPSPEE Filing & Tracking System — CDHP Document 07

---

## 1. Repository Structure

```
ovpspee-filing-system/
  src/                    ← React + TypeScript frontend
  src-tauri/
    src/
      commands/           ← Tauri IPC command handlers (one file per domain)
        auth.rs
        users.rs
        categories.rs
        folders.rs
        offices.rs
        documents.rs
        attachments.rs
        scan_intake.rs
        audit_log.rs
        backup.rs
        settings.rs
        dashboard.rs
      models/             ← Rust structs that map to DB rows and API DTOs
      db/                 ← Database pool setup, migration runner
      storage/            ← File system utilities (save, move, delete, path validation)
      pdf/                ← PDF generation
      error.rs            ← AppError enum and Into<String> impl
      lib.rs              ← Command registration, plugin setup
      main.rs             ← Tauri builder entry point
    migrations/           ← SQLx migration files (.sql)
    tests/                ← Integration tests
    Cargo.toml
    tauri.conf.json
  .sqlx/                  ← SQLx offline query cache (committed to git)
  pnpm-lock.yaml
  package.json
  tailwind.config.ts
  tsconfig.json
  vite.config.ts
  .gitignore
  .env.example
```

---

## 2. Code Style

### Rust

- Follow standard Rust idioms (`rustfmt`, `clippy`).
- Run `cargo fmt` and `cargo clippy -- -D warnings` before every commit.
- Add doc comments (`///`) to public API functions in auth.rs, documents.rs, and backup.rs as a starting goal.
- Use `thiserror` for the `AppError` enum; convert to `String` at the Tauri command boundary.
- Never use `unwrap()` in production code. Use `?` propagation or `map_err`.
- Prefer `sqlx::query_as!` macro (compile-time checked) over raw string queries.

```rust
// Good
pub async fn get_document(
    pool: &SqlitePool,
    document_id: i64,
) -> Result<DocumentDetail, AppError> {
    sqlx::query_as!(
        DocumentDetail,
        "SELECT * FROM document WHERE document_id = ?",
        document_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

// Bad — panics in production
let doc = pool.fetch_one(...).await.unwrap();
```

### TypeScript / React

- Use **functional components only**. No class components.
- All components must have explicit TypeScript prop interfaces.
- Use `const` arrow functions for components: `const MyComponent = () => {...}`.
- File names: `PascalCase.tsx` for components, `camelCase.ts` for utilities and hooks.
- No `any` types. Use `unknown` + type narrowing if the type is truly unknown.
- Run `pnpm tsc --noEmit` and `pnpm test` before every commit.

```typescript
// Good
interface DocumentCardProps {
  document: DocumentListItem;
  onView: () => void;
}

const DocumentCard = ({ document, onView }: DocumentCardProps) => {
  return <div onClick={onView}>{document.document_name}</div>;
};

// Bad
const DocumentCard = (props: any) => { ... };
```

---

## 3. Git Workflow

### Branches

| Branch | Purpose |
|---|---|
| `main` | Always deployable; tagged releases only |
| `develop` | Integration branch; all slices merged here first |
| `slice/N-short-description` | Feature branch per vertical slice |
| `fix/short-description` | Bug fix branch |

### Commit Messages

Follow **Conventional Commits** format:

```
type(scope): short description

[optional body]
[optional footer]
```

| Type | Usage |
|---|---|
| `feat` | New feature (maps to a slice or sub-feature) |
| `fix` | Bug fix |
| `test` | Adding or updating tests |
| `docs` | Documentation changes |
| `refactor` | Code restructuring without behavior change |
| `chore` | Build config, dependency updates |

**Examples:**
```
feat(documents): implement trash and restore workflow
fix(auth): deactivated user now blocked from login
test(backup): add integration test for archive import
chore: update sqlx to 0.8.2
```

### Pull Request Rules

1. Every PR must target `develop` (never `main` directly).
2. PR title follows the same Conventional Commits format.
3. PR must include a summary of what was changed and how to test it.
4. All CI checks must pass before merge.
5. At least one review approval required (self-review acceptable for solo projects — still write the PR description).
6. Squash merge into `develop`.

### Release Process

1. All slices for the release are merged and tested on `develop`.
2. Open a PR: `develop → main`.
3. Final manual verification checklist run.
4. Merge and tag: `git tag v1.0.0`.
5. Run `cargo tauri build` from the tagged commit.
6. Distribute the signed installer.

---

## 4. Database Migration Rules

- **Never edit an existing migration file.** Once a migration is committed and applied, it is immutable.
- To change the schema, create a new migration file with the next sequential number.
- Always run `cargo sqlx prepare` after adding or changing a query or migration, and commit the updated `.sqlx/` folder.
- Migration file naming: `NNNN_descriptive_name.sql` (e.g., `0008_add_scan_intake_table.sql`).

---

## 5. File Storage Rules

All file storage operations must go through the `storage/` module in Rust. **Never** construct file paths directly in command handlers.

### Path Safety Contract

- `attachment.file_path` and scan intake `stored_relative_path` are always stored as relative paths (e.g., `documents/42/uploaded/report.pdf`).
- The absolute path is resolved at runtime by joining with the configured base directory.
- The `safe_path()` check is mandatory before any file read or serve operation.

---

## 6. Session Management Rules

- Every Tauri command that requires authentication must call a `require_session(db, session_id)` helper at the top of the function. This helper validates the session, checks expiry, and returns the `user_id` and `role`.
- Every command that requires Admin role must additionally call `require_admin_role(role)`.
- Never trust `user_id` passed as a parameter for privileged operations. Always derive `user_id` from the validated session.

```rust
// Pattern for every protected command
#[tauri::command]
pub async fn protected_command(
    db: State<DbPool>,
    session_id: String,
    // ... other params
) -> CmdResult<SomeType> {
    let session = require_session(&db, &session_id).await?;
    // session.user_id and session.role are now trusted values
    // ...
}
```

---

## 7. Shared Frontend Utilities

### `src/lib/helpers.ts`

Common utility functions extracted from duplicated code. Never redefine these locally:

| Function | Purpose |
|---|---|
| `nullable(value)` | Trim → null for empty optional fields |
| `fileNameFromPath(path)` | Extract filename from full path |
| `normalizeSelectedPaths(selected)` | Normalize Tauri dialog result to `string[]` |
| `safeFileName(value)` | Sanitize string for filesystem-safe filename |
| `sizeLabel(bytes)` | Format as KB (ceil) |
| `extensionFromName(name)` | Extract lowercase extension, fallback `'unknown'` |
| `formatBytes(value)` | Format as B/KB/MB with one decimal |

## Confirmation Dialogs

Confirmation dialogs use inline `useState<ConfirmAction | null>` — see `src/components/ConfirmDialog.tsx` for the `ConfirmAction` interface.

For high-severity actions (purge, revoke device, restore), pass `requiredText` to force the user to type the item name before confirm is enabled.

## Toast Notifications

Use the global toast system instead of per-page message state:

```typescript
import { useToast } from '../components/Toast';

const { addToast } = useToast();
addToast('success', 'Document saved.');
addToast('error', 'Operation failed.');
addToast('info', 'Scan complete.');
```

Toasts auto-dismiss after 5 seconds, stack vertically, and are color-coded by type. Do not use `setMessage`/`useState<string|null>` for transient notifications — use this system instead.

## Form Components

Use the shared form components from `src/components/forms/` instead of defining local helpers:

```typescript
import { FormTitle, Status, IconButton, TextField, FieldError } from '../components/forms';
```

These handle required field indicators (red asterisk), inline validation (FieldError), and accessibility attributes consistently. If you need a new form primitive, add it to the shared library rather than defining it locally.

---

## 8. Audit Logging Rules

Every create, update, delete, move, hide, trash, restore, purge, backup, scan, and login/logout operation must write to `audit_log`. Use the shared `write_audit_log()` helper:

```rust
pub async fn write_audit_log(
    pool: &SqlitePool,
    action: &str,
    table_affected: Option<&str>,
    record_id: Option<i64>,
    description: &str,
    user_id: Option<i64>,
) -> Result<(), AppError>
```

Move operations must include both previous and new location in the description:
```
"Moved document #42 from category_id=2, folder_id=5 to category_id=3, folder_id=9"
```

---

## 8. Adding a New Tauri Command — Checklist

When adding a new command, complete every step:

- [ ] Define input/output structs in `src-tauri/src/models/`
- [ ] Implement the command function in the appropriate file under `src-tauri/src/commands/`
- [ ] Add `require_session()` (and `require_admin_role()` if needed) at the top
- [ ] Add `write_audit_log()` call for any data-modifying operations
- [ ] Register the command in `lib.rs` under `invoke_handler`
- [ ] Call `invoke` from `@tauri-apps/api/core` directly in the component or hook
- [ ] Write unit tests (and integration test if applicable)
- [ ] Run `cargo sqlx prepare` if the command uses new SQL queries
- [ ] Document the command in `03_Backend_API_Documentation.md`

---

## 9. Dependency Management

- **Rust:** Add crates via `cargo add <crate>` in `src-tauri/`. Review the crate's license before adding. Prefer well-maintained crates with >1M downloads or active maintenance.
- **Node:** Add packages via `pnpm add <package>`. Use `pnpm add -D` for dev-only packages.
- **Shadcn/ui components:** Add via `pnpm dlx shadcn@latest add <component>`. Never install shadcn as a runtime npm package.
- Review `pnpm audit` and `cargo audit` output periodically. Fix `high` and `critical` vulnerabilities before release.

---

## 10. Naming Conventions

| Context | Convention | Example |
|---|---|---|
| Rust functions | `snake_case` | `create_document`, `list_audit_logs` |
| Rust structs/enums | `PascalCase` | `DocumentDetail`, `AppError` |
| Tauri command names | `snake_case` (string) | `"create_document"` |
| TypeScript functions | `camelCase` | `createDocument`, `listDocuments` |
| React components | `PascalCase` | `DocumentCard`, `StatusBadge` |
| React files | `PascalCase.tsx` | `AddDocument.tsx`, `MasterData.tsx` |
| Utility files | `camelCase.ts` | `errors.ts`, `helpers.ts` |
| DB table names | `snake_case` | `document`, `audit_log`, `scan_intake` |
| DB column names | `snake_case` | `document_id`, `is_hidden`, `trashed_at` |
| CSS class names | Tailwind utility only | No custom CSS class names |

---

*End of Developer Guidelines*
*Next: `08_Developer_Troubleshooting.md`*


---

## Large File Handling

Do not pass attachment or scan contents as large byte arrays through Tauri IPC. Use path-based file selection and let Rust validate and copy files into managed storage. The hard limit is 1 GB per file, with UI warning above 250 MB. Store only relative destination paths in SQLite.

# Developer Troubleshooting
## OVPSPEE Filing & Tracking System — CDHP Document 08

---

## 1. Build Failures

### `error: failed to run custom build command for 'openssl-sys'`

**Cause:** Missing OpenSSL development headers on Windows.

**Fix:** Argon2 and other crates may pull in OpenSSL indirectly. Switch to the pure-Rust alternative or install OpenSSL via vcpkg:

```powershell
# Option 1: Use a pure-Rust TLS stack — add to Cargo.toml features:
# (ensure you are not depending on crates that require native OpenSSL)

# Option 2: Install OpenSSL via vcpkg
git clone https://github.com/microsoft/vcpkg
cd vcpkg
.\bootstrap-vcpkg.bat
.\vcpkg install openssl:x64-windows-static-md
# Then set env vars:
$env:OPENSSL_DIR = "C:\path\to\vcpkg\installed\x64-windows-static-md"
$env:OPENSSL_STATIC = "1"
```

---

### `error[E0308]: mismatched types` from `sqlx::query_as!`

**Cause:** The `.sqlx/` cached query file is out of sync with the actual schema after a migration.

**Fix:**

```powershell
cd src-tauri
sqlx migrate run   # Apply pending migrations
cargo sqlx prepare # Regenerate .sqlx/ cache
cd ..
cargo build
```

---

### `cargo tauri dev` starts but the window is blank / shows a white screen

**Cause:** The Vite dev server hasn't started yet when Tauri opens the window, or there is a React runtime error.

**Fix:**
1. Open DevTools: right-click in the window → Inspect (or `F12`).
2. Check the Console tab for React errors.
3. If Vite isn't ready: wait 5–10 seconds and refresh (Ctrl+R).
4. If this is a persistent issue, check that `devUrl` in `tauri.conf.json` matches the Vite port (default `http://localhost:1420`).

---

### `tauri build` fails with `Error failed to bundle project: bundle.targets contains an unsupported target`

**Cause:** Building for a target that isn't supported on the current OS (e.g., trying to build `.deb` on Windows).

**Fix:** On Windows, only build for Windows targets (`msi`, `nsis`). Linux targets (`appimage`, `deb`) must be built on a Linux machine.

---

### `WebView2 not found` error on end-user machine

**Cause:** Windows machine does not have the WebView2 runtime installed (rare on Windows 10 with recent updates; more common on older or stripped-down images).

**Fix:**
- The Tauri bundler can embed the WebView2 bootstrapper in the installer. Enable this in `tauri.conf.json`:
  ```json
  "bundle": {
    "windows": {
      "webviewInstallMode": { "type": "embedBootstrapper" }
    }
  }
  ```
- Alternatively, direct the user to install WebView2 manually from https://developer.microsoft.com/microsoft-edge/webview2/

---

## 2. Runtime Errors

### `ERR_DB: no rows returned by a query expected to return at least one row`

**Cause:** `fetch_one()` on an SQLx query when the record doesn't exist.

**Fix:** Use `fetch_optional()` and map `None` to `AppError::NotFound`:

```rust
let doc = sqlx::query_as!(DocumentDetail, "SELECT ... WHERE document_id = ?", id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Document".into()))?;
```

---

### `ERR_IO: No such file or directory` when serving an attachment

**Cause:** File exists in the database but was manually deleted from the file system (or storage base directory is misconfigured).

**Fix:**
1. Check `settings.storage_base_dir` is correct and points to the same directory used when the file was saved.
2. If the file is truly gone, remove the orphaned `attachment` record and note the data loss in the audit log.
3. Add a storage health check to the Admin dashboard (future improvement — see Developer Notes).

---

### Document moves to TRASH but `original_category_id` is NULL after restore

**Cause:** The `trash_document` command was called when `category_id` was already NULL on the document.

**Fix:** Ensure `document.category_id` is always non-NULL before trashing. Add a guard:

```rust
if document.category_id == trash_category_id {
    return Err(AppError::Conflict("Document is already in TRASH".into()));
}
```

---

### Session is invalidated immediately after login (app loops back to login)

**Cause:** System clock on the machine is incorrect, causing `expires_at` to be in the past immediately.

**Fix:**
1. Sync the system clock (Windows: `Settings → Time & Language → Sync now`).
2. Review session expiry duration in the session creation code — ensure it uses a positive duration (e.g., 8 hours).

---

### Scan files not appearing in intake after import

**Cause 1:** The imported files are of a type not recognized by the intake importer.

**Fix:** The intake importer should accept `.jpg`, `.jpeg`, `.png`, `.pdf`, `.tif`, `.tiff`. Check the file extension filter in `import_scan_files`.

**Cause 2:** The `storage/intake/` directory doesn't exist or isn't writable.

**Fix:** Ensure the storage base directory has been created and the app has write permissions. On first run, the `db/` init function should create all required subdirectories.

---

### Thumbnail not generated for a scan

**Cause:** Thumbnail generation fails silently for certain file types (e.g., multi-page TIFFs, corrupted JPEGs).

**Fix:** Thumbnail generation should be non-fatal — if it fails, set `thumbnail_path = NULL` and display a generic file icon in the intake grid. Log the failure to the app error log.

---

## 3. Database Issues

### `SQLITE_BUSY: database is locked`

**Cause:** Two concurrent operations are attempting to write to the database. This should not happen in normal single-user desktop use, but can occur if the backup operation runs while a document is being saved.

**Fix:**
1. Ensure WAL mode is enabled: `PRAGMA journal_mode=WAL;` (set in the connection setup).
2. Set a busy timeout: `PRAGMA busy_timeout=5000;` (5 seconds).
3. Ensure the backup uses SQLite's online backup API (not direct file copy) to avoid locking conflicts.

---

### Migration fails on startup: `table already exists`

**Cause:** A migration was partially applied before a crash, leaving the schema in an inconsistent state.

**Fix:**
1. SQLx tracks applied migrations in the `_sqlx_migrations` table. Check which migrations are recorded:
   ```sql
   SELECT * FROM _sqlx_migrations;
   ```
2. If the table exists but the migration is not recorded, the migration ran partially. Manually roll back the partial changes and re-run migrations.
3. Preventative: always use SQLite transactions in migration files (`BEGIN; ... COMMIT;`).

---

### Database file is corrupted after a power loss

**Cause:** Power loss during a write (rare with WAL mode, but possible).

**Fix:**
1. Restore from the most recent backup.
2. If no backup is available, try SQLite's integrity check:
   ```sql
   PRAGMA integrity_check;
   ```
3. If the database is partially recoverable, use the SQLite CLI to export recoverable tables and import into a fresh database.
4. Document this scenario in the Admin's post-incident notes.

---

## 4. Frontend Issues

### `invoke()` call rejects with a cryptic error string

**Fix:**
1. The Tauri command name in `invoke('command_name', ...)` must exactly match the function name registered in `lib.rs` under `invoke_handler!`.
2. Parameter names in the `invoke()` call must match the Rust function parameter names exactly (camelCase in JS maps to snake_case in Rust via Tauri's serialization — **Tauri handles this conversion automatically**).
3. Check the Tauri logs in the terminal running `cargo tauri dev` for the actual Rust panic or error message.

---

### React Query cache shows stale data after a mutation

**Fix:** After every mutation (create, update, delete, move), invalidate the relevant query keys:

```typescript
const queryClient = useQueryClient();

// After create_document:
queryClient.invalidateQueries({ queryKey: ['documents'] });
queryClient.invalidateQueries({ queryKey: ['dashboard-stats'] });

// After trash_document:
queryClient.invalidateQueries({ queryKey: ['documents'] });
queryClient.invalidateQueries({ queryKey: ['intake'] }); // not needed but safe
```

Define all query keys as constants in a `src/lib/queryKeys.ts` file to avoid typos.

---

### Tailwind classes not applying correctly

**Cause:** Class name is dynamically constructed (string interpolation), which Tailwind's purge/JIT cannot detect.

**Fix:** Never construct class names dynamically:

```typescript
// Bad — Tailwind cannot detect this
const color = `bg-${category.color_code}-100`;

// Good — use inline styles for dynamic values
<div style={{ backgroundColor: category.color_code }} />

// Good — use a lookup map for known variants
const statusColors: Record<string, string> = {
  Filed: 'bg-green-100 text-green-800',
  Archived: 'bg-slate-100 text-slate-700',
};
<span className={statusColors[status]} />
```

---

### `dnd-kit` drag-and-drop not working inside a Shadcn Dialog

**Cause:** The Dialog component uses a portal, which can interfere with pointer events for dnd-kit.

**Fix:** Add `modifiers={[restrictToWindowEdges]}` to the `DndContext` and ensure the drag overlay is rendered using a `DragOverlay` component attached to the same `DndContext`. Reference the dnd-kit docs for portal-aware configuration.

---

## 5. Installer / Deployment Issues

### Installer fails on Windows with `Error 1603: A fatal error occurred during installation`

**Cause:** Usually a permission issue or a conflicting previous installation.

**Fix:**
1. Run the installer as Administrator (right-click → Run as administrator).
2. Uninstall the previous version first via Add/Remove Programs.
3. Check the Windows Event Viewer → Application log for more specific MSI error codes.

---

### App data directory not found after upgrading

**Cause:** The `identifier` in `tauri.conf.json` was changed between versions, causing Tauri to use a different app data path.

**Fix:** **Never change `identifier` after the initial release.** It is used to determine the app data directory path on all platforms. Changing it effectively creates a new app with no data.

---

*End of Developer Troubleshooting*
*Next: `09_Developer_Notes.md`*


---

### Large file copy freezes or runs out of memory

**Cause:** File bytes are being passed through Tauri IPC instead of copied by the Rust backend from a selected source path.

**Fix:** Use the path-based upload contract (`UploadedFileRef`) and copy files in Rust after validating extension, size, and destination path safety. Warn above 250 MB and reject above 1 GB.

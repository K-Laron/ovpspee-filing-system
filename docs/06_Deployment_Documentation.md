# Deployment Documentation
## OVPSPEE Filing & Tracking System — CDHP Document 06

---

## 1. Target Environments

| Platform | Minimum Version | Build Output |
|---|---|---|
| **Windows** (primary) | Windows 10 64-bit | `.msi` installer or `.exe` (NSIS) |
| **Linux** (deferred post-MVP) | Ubuntu 20.04 LTS | `.AppImage` or `.deb` |

Linux support is not in scope for the initial release. All deployment documentation in this file covers **Windows only** unless otherwise noted.

---

## 2. Developer Machine Prerequisites

Install the following before building the project. All tools must be available on the developer's `PATH`.

| Tool | Version | Install |
|---|---|---|
| **Rust** | 1.76+ (stable) | `rustup` — https://rustup.rs |
| **Node.js** | 20 LTS | https://nodejs.org |
| **pnpm** | 8+ | `npm install -g pnpm` |
| **Tauri CLI** | 2.x | `cargo install tauri-cli --version "^2"` |
| **SQLx CLI** | Latest | `cargo install sqlx-cli --no-default-features --features sqlite` |
| **WebView2 Runtime** | Latest | Ships with Windows 10/11; download from Microsoft if absent |

### Windows Build Target

```powershell
# Ensure the MSVC toolchain is installed (not GNU)
rustup default stable-x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc
```

---

## 3. Project Setup (First Time)

```powershell
# 1. Clone the repository
git clone https://github.com/your-org/ovpspee-filing-system.git
cd ovpspee-filing-system

# 2. Install frontend dependencies
pnpm install

# 3. Set up SQLx offline mode (compile-time query checking)
#    Run this after any schema change
cd src-tauri
sqlx database create   # Creates a local dev SQLite DB at DATABASE_URL
sqlx migrate run       # Applies all migrations
cargo sqlx prepare     # Updates .sqlx/ query cache (commit this folder)
cd ..

# 4. Verify the dev build
cargo tauri dev
```

---

## 4. Environment Variables

Create a `.env` file in `src-tauri/` for development. **Never commit `.env` to source control.**

```env
DATABASE_URL=sqlite:../dev_filing_system.db
STORAGE_BASE_DIR=../dev_storage
```

For production builds, these values are not used — the application sets its own paths from the OS data directory at runtime.

---

## 5. Development Workflow

```powershell
# Start the full dev server (hot-reload React + live Rust recompile)
cargo tauri dev

# Run Rust unit tests only
cargo test --manifest-path src-tauri/Cargo.toml

# Run integration tests only
cargo test --manifest-path src-tauri/Cargo.toml --test integration

# Run frontend type checking
pnpm tsc --noEmit

# Lint frontend
pnpm eslint src/
```

---

## 6. Production Build

### Step 1 — Prepare SQLx queries

```powershell
cd src-tauri
sqlx migrate run  # Ensure migrations are applied to the build DB
cargo sqlx prepare  # Regenerate .sqlx/ query cache
cd ..
```

### Step 2 — Build the installer

```powershell
cargo tauri build
```

Build output location:
```
src-tauri/target/release/bundle/
  msi/
    OVPSPEE Filing System_1.0.0_x64_en-US.msi
  nsis/
    OVPSPEE Filing System_1.0.0_x64-setup.exe
```

Both formats are produced by default. Distribute the `.msi` for enterprise environments (Group Policy compatible); the `.exe` for direct user installation.

### Step 3 — Sign the installer (recommended before deployment)

Code signing prevents Windows SmartScreen warnings. Use a valid code-signing certificate issued to "University of Eastern Philippines" or your organization.

```powershell
# Sign the .msi using signtool (from Windows SDK)
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 `
  /f "path\to\certificate.pfx" /p "cert_password" `
  "src-tauri\target\release\bundle\msi\OVPSPEE Filing System_1.0.0_x64_en-US.msi"
```

If a code-signing certificate is not available for MVP, the installer will trigger a SmartScreen "Unknown Publisher" warning on first run. Users must click "More info → Run anyway." Document this in the deployment notes for the client.

---

## 7. Installer Behavior

### What the installer does

1. Copies the application executable to `C:\Program Files\OVPSPEE Filing System\`.
2. Creates a Desktop shortcut and Start Menu entry.
3. Registers the application in Add/Remove Programs (Programs and Features).
4. **Does not** create the data directory or database — these are created on first run.

### What happens on first run

1. The application detects that no `filing_system.db` exists in the data directory.
2. Runs all SQLite migrations automatically.
3. Seeds the database (roles, TRASH category, default settings).
4. Presents the **First-Run Setup** screen to the Admin.

### Data directory

```
%APPDATA%\ovpspee-filing-system\
  filing_system.db         ← SQLite database
  storage\                 ← File attachments and scanned pages
    documents\
    intake\
    profiles\
  logs\                    ← Application error logs (optional)
```

This directory is **preserved on uninstall** by default (Tauri behavior). The data is not deleted when the application is removed. To fully remove all data, the Admin must manually delete this folder.

### What happens on reinstall / upgrade

- The installer overwrites the application executable.
- The data directory and database are untouched.
- On startup, SQLx migrations apply any new migrations that weren't in the previous version.
- The application is immediately usable with all previous data.

---

## 8. First-Run Deployment Checklist (Admin)

Complete these steps after installing the application on the production machine.

- [ ] Run the installer; verify the application opens to the First-Run Setup screen.
- [ ] Create the initial Admin account (username, full name, strong password). Store credentials securely.
- [ ] Log in as Admin.
- [ ] Navigate to Master Data → Offices. Add all known sender offices.
- [ ] Navigate to Master Data → Categories. Create all required document categories with appropriate colors and icons.
- [ ] Navigate to Master Data → Folders. Create folders for each category.
- [ ] Navigate to Users. Create Secretary account(s). Provide credentials to the relevant staff.
- [ ] Navigate to Backup & Restore. Use the default local app-data backup folder or configure an external/network backup destination; daily schedule recommended.
- [ ] Test the backup manually: click "Create Backup Now" and verify the output folder.
- [ ] Log out.
- [ ] Log in as Secretary; verify document filing works end-to-end.
- [ ] Use no-login Staff/Head Viewer mode; verify only public non-hidden documents are visible.

---

## 9. Backup & Disaster Recovery

### Recommended backup strategy

- **Scheduled backup:** Daily, at 02:00 AM, to a separate physical drive or USB.
- **Retention:** Keep last 5 backups (auto-purge older ones).
- **Monthly portable export:** Export a `.ovpspee-backup` archive and store it off-site (external drive, cloud storage).

### Disaster recovery procedure

1. Install the application on the replacement machine.
2. Complete First-Run Setup (create any temporary Admin account — it will be overwritten).
3. Navigate to Backup & Restore → Import Backup Archive.
4. Select the latest `.ovpspee-backup` file.
5. Confirm. The application restarts and all data is restored.

---

## 10. Tauri Configuration Reference

Key fields in `src-tauri/tauri.conf.json`:

```json
{
  "productName": "OVPSPEE Filing System",
  "version": "1.0.0",
  "identifier": "ph.edu.uep.ovpspee-filing-system",
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  },
  "app": {
    "windows": [
      {
        "title": "OVPSPEE Filing and Tracking System",
        "width": 1280,
        "height": 800,
        "minWidth": 1024,
        "minHeight": 768,
        "resizable": true,
        "fullscreen": false
      }
    ]
  }
}
```

---

## 11. Version Numbering

Follow **Semantic Versioning** (`MAJOR.MINOR.PATCH`):

- `MAJOR` — Breaking change to data format or backup schema (requires migration notice to client).
- `MINOR` — New feature (new slice delivered).
- `PATCH` — Bug fix, no schema change.

Update `version` in `tauri.conf.json` and `src-tauri/Cargo.toml` before every release build. Tag the git commit: `git tag v1.0.0`.

---

## 12. Post-Deployment Monitoring

This is a single-machine desktop app with no server. "Monitoring" means:

- **Admin reviews audit logs** periodically to verify all operations are being logged.
- **Admin/IT Staff verifies backups** are being created on schedule (check the local backup folder date stamps and confirm periodic copy to external/network storage).
- **App error logs** (if enabled) are in `%APPDATA%\ovpspee-filing-system\logs\`.

---

*End of Deployment Documentation*
*Next: `07_Developer_Guidelines.md`*

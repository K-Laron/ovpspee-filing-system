# Installation Guide

Release commit: `8795489`

## Installer Files

- MSI installer: `D:\codex-target-ovpspee\release\bundle\msi\OVPSPEE Filing System_0.1.0_x64_en-US.msi`
- NSIS installer: `D:\codex-target-ovpspee\release\bundle\nsis\OVPSPEE Filing System_0.1.0_x64-setup.exe`

## Install

1. Run either installer on Windows.
2. Complete installer prompts.
3. Launch `OVPSPEE Filing System`.
4. On fresh database, complete first-run Admin setup.

## Confirmed Install Validation

- `cargo tauri build`: PASS.
- MSI output exists.
- NSIS output exists.
- Installed app launch: PASS.

## Notes

- App data, SQLite DB, storage, backups, and restore staging use local app data directories.
- Restore requires manual app restart after completion.


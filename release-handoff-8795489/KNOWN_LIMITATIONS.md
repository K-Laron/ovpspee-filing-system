# Known Limitations

Release commit: `8795489`

## Confirmed Limitations

- `manual-final-release-audit/` is untracked unless later committed or archived.
- Scheduled backup has settings/check command but no always-on background timer.
- Exact letterhead and certification wording is still placeholder.
- App restart after restore is manual by design.
- No OCR.
- No direct scanner integration.
- No cloud or web server deployment.

## Non-Blocking Notes

- SQLx offline cache is crate-local under `src-tauri/.sqlx`.
- Root offline check requires `SQLX_OFFLINE_DIR=src-tauri/.sqlx`.
- Installer output was built under `D:\codex-target-ovpspee` because project uses `CARGO_TARGET_DIR` to avoid C: disk pressure.


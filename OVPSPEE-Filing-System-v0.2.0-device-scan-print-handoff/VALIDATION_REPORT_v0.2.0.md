# Validation Report v0.2.0

Release tag: `v0.2.0-device-scan-print`

Tagged commit: `b9e4616`

Tag object: `33e7e60`

Branch: `master`

## Automated Validation

- `cargo check --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 116 tests
- `pnpm tsc --noEmit`: PASS
- `pnpm build`: PASS
- `cargo sqlx prepare -- --all-targets`: PASS
- `SQLX_OFFLINE=true SQLX_OFFLINE_DIR=src-tauri/.sqlx cargo check --manifest-path src-tauri/Cargo.toml`: PASS
- `git diff --check`: PASS
- `git status --short`: clean

## Manual Validation

- Admin dashboard opens.
- Admin Devices opens.
- EPSON scanner and Windows printers detected.
- Admin device defaults saved.
- Admin Audit Log opens and shows device/scan/print events.
- Admin Backup & Restore opens.
- Backup creation works.
- Admin still has no normal filing/create/edit/move/status navigation.
- Secretary Documents opens.
- Secretary print/export/attachment preview controls remain intact.
- Secretary Scan Intake opens.
- Manual import remains visible.
- Scanner capture produced pending PNG intake item.
- Public viewer browser opens.
- Public visible document detail has export/print controls.
- Hidden/confidential documents are absent from public viewer list.
- Audit logs show safe event summaries.
- No password, password hash, file contents, or secrets were seen in audit output.

## Notes

- Restore destructive flow was not manually re-executed during feature audit.
- Automated restore tests passed.
- Restore controls were visible.


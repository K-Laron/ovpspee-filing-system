# Plan 014 — MobileDevices revoke confirmation

**Source**: Finding #2 from UI/UX audit

**Goal**: Add `ConfirmDialog` with `requiredText` (device name) before revoking a device, matching app's own destructive-action security pattern

## Steps
1. In `MobileDevices.tsx`, import and add `ConfirmDialog` state (`showRevokeConfirm`, `deviceToRevoke`)
2. Wrap revoke handler: show confirm dialog instead of calling API directly
3. On confirm, call API with device name typed correctly
4. Wire `requiredText` to the device name displayed in the row

## Verification
- `pnpm verify` exit 0

## Done Criteria
- Revoke button opens confirm dialog requiring typed device name
- Cancel does nothing
- Confirm with correct name proceeds

## STOP Conditions
- None

## Dependencies
- None

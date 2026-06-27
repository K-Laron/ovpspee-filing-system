# Plan 013 — AuditLog message differentiation

**Source**: Finding #1 from UI/UX audit

**Goal**: Replace single `message` state with `{type: 'success' | 'error', text: string}` so users can visually distinguish success from failure

## Steps
1. Change `message` state type in `AuditLog.tsx` from `string` to `{type: 'success' | 'error'; text: string} | null` (line 28)
2. Update all `setMessage(...)` calls to pass type + text
3. Update message banner rendering to apply green/red styling based on type
4. Re-render test snapshots if needed

## Verification
- `pnpm verify` exit 0

## Done Criteria
- Green banner for success, red for error
- All existing behavior preserved

## STOP Conditions
- None

## Dependencies
- None

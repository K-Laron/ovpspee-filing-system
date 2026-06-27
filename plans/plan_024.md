# Plan 024 — Auto-dismiss message banners

**Source**: Finding #12 from UI/UX audit

**Goal**: Auto-dismiss success/error message banners after 5 seconds

## Steps
1. In `AuditLog.tsx` and any page using `setMessage`, add `useEffect` that starts a 5s timer when message is set
2. Timer calls `setMessage(null)` on expiry
3. Clear timer on unmount or if message changes
4. Keep manual dismiss (click/X) as alternative

## Verification
- `pnpm verify` exit 0

## Done Criteria
- Messages auto-dismiss after 5s
- Manual dismiss still works
- Timer resets if new message arrives before old one dismisses

## STOP Conditions
- None

## Dependencies
- Plan 013 (message type refactor) — apply after plan 013 for consistency

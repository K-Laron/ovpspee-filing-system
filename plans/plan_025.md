# Plan 025 — Toast notification system

**Source**: Finding #13 from UI/UX audit

**Goal**: Replace inline `setMessage` pattern with a global toast notification component (positioned, auto-dismiss, stacked)

## Steps
1. Create `src/components/Toast.tsx` — renders stacked toasts at top-right, each with type (success/error/info), auto-dismiss, close button
2. Create Toast store (Zustand slice) or React context: `addToast(type, text)`, `removeToast(id)`
3. Replace all `setMessage(...)` calls across pages with `addToast(...)`
4. Remove per-page message state where no longer needed
5. Keep page-level message only for inline banners that aren't toast-appropriate

## Verification
- `pnpm verify` exit 0

## Done Criteria
- Success/error toasts appear at top-right, auto-dismiss after 5s
- Multiple toasts stack vertically
- Clicking toast dismisses it
- All existing message patterns use toast system

## STOP Conditions
- None

## Dependencies
- Supersedes plans 013 and 024 (message color, auto-dismiss) — execute this one instead; skip those if this is done

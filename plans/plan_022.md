# Plan 022 — Mobile nav a11y improvements

**Source**: Finding #10 from UI/UX audit

**Goal**: Add focus trap, `aria-current="page"`, and `aria-expanded` to mobile navigation in AppShell

## Steps
1. In `AppShell.tsx:54-57`, add keyboard focus trap for mobile nav overlay (trap Tab within nav, Escape to close)
2. Add `aria-current="page"` on active `NavLink` (line 80-90 area)
3. Add `aria-expanded` on mobile menu toggle button
4. Test with keyboard navigation

## Verification
- `pnpm verify` exit 0
- Manual keyboard test: Tab cycle stays within nav when open, Escape closes

## Done Criteria
- Focus trapped in nav when open
- Active page announced by screen reader
- Menu button announces expanded state

## STOP Conditions
- None

## Dependencies
- None

# Plan 019 — Breadcrumbs

**Source**: Finding #7 from UI/UX audit

**Goal**: Add breadcrumb navigation component showing current page path hierarchy (e.g., "Admin > Users > Edit John")

## Steps
1. Create `src/components/Breadcrumbs.tsx` — takes `{segments: {label: string, href?: string}[]}`
2. Style with grey text, separators, link styling for non-current segments
3. Add breadcrumbs to pages with depth > 1: Users (list → edit), MasterData, possibly others
4. Wire breadcrumb segments from route state or current page context

## Verification
- `pnpm verify` exit 0

## Done Criteria
- Breadcrumbs visible on sub-pages
- Clicking non-current segment navigates correctly
- Current segment is plain text (not link)

## STOP Conditions
- None

## Dependencies
- None

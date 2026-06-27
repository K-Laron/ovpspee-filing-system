# Plan 023 — Loading skeletons

**Source**: Finding #11 from UI/UX audit

**Goal**: Replace "Loading..." text in table bodies with CSS skeleton/placeholder animations

## Steps
1. Create `src/components/Skeleton.tsx` — generic skeleton component with animated pulse/gradient
2. Create `TableSkeleton` variant: renders skeleton rows matching the table's column structure
3. Replace "Loading..." text in all pages (Documents, TrashManagement, AuditLog, Users, MobileDevices, etc.)
4. Reuse `TableSkeleton` across all list/table pages

## Verification
- `pnpm verify` exit 0

## Done Criteria
- All loading states show animated skeleton rows instead of "Loading..." text
- Skeletons match table column layout
- Animation is subtle (Tailwind `animate-pulse`)

## STOP Conditions
- None

## Dependencies
- None

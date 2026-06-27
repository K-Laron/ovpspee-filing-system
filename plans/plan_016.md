# Plan 016 — TrashManagement search/filter

**Source**: Finding #4 from UI/UX audit

**Goal**: Add search input to filter trashed documents by name

## Steps
1. Add `searchQuery` state in `TrashManagement.tsx`
2. Add search input above the table
3. Filter trashed documents client-side by name matching `searchQuery`
4. Show "No documents match your search" empty state when filter has no results vs "Trash is empty" when truly empty

## Verification
- `pnpm verify` exit 0

## Done Criteria
- Text input filters trash list as user types
- Empty state distinguishes "no results" from "no trashed documents"

## STOP Conditions
- None

## Dependencies
- None

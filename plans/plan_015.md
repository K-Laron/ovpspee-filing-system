# Plan 015 — ScanIntake document dropdown search/pagination

**Source**: Finding #3 from UI/UX audit

**Goal**: Add search input above document dropdown so users can filter documents by name instead of scrolling thousands of options

## Steps
1. Add `searchQuery` state in `ScanIntake.tsx`
2. Add text input above the `<select>` for document filter
3. Filter documents client-side by `document_name` matching `searchQuery` (case-insensitive)
4. Show filtered count vs total

## Verification
- `pnpm verify` exit 0

## Done Criteria
- Text input filters document list as user types
- Unfiltered state shows all documents (existing behavior)
- Count display: "Showing X of Y documents"

## STOP Conditions
- None

## Dependencies
- None

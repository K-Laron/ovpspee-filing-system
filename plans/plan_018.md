# Plan 018 — Remove auto-open first detail

**Source**: Finding #6 from UI/UX audit

**Goal**: Stop auto-selecting the first document/mobile submission on page load; user clicks to view details

## Steps
1. In `Documents.tsx`, remove or set to null the default selection of first document in the list
2. In `MobileSubmissions.tsx`, same treatment
3. Verify detail panel shows empty/placeholder state when nothing selected

## Verification
- `pnpm verify` exit 0

## Done Criteria
- No row pre-selected on page load
- Detail panel shows empty state until user clicks a row
- Clicking a row still opens its details

## STOP Conditions
- None

## Dependencies
- None

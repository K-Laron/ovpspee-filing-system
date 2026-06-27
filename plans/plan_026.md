# Plan 026 — MyActivity filter consistency

**Source**: Finding #14 from UI/UX audit

**Goal**: Change MyActivity's free-text Entity filter to a `<select>` dropdown matching the Action filter pattern

## Steps
1. In `MyActivity.tsx`, locate the Entity filter input
2. Replace free-text `<input>` with `<select>` populated from unique entity types in the data
3. Keep current filter behavior (filter by selected entity type)
4. Add "All" option as default

## Verification
- `pnpm verify` exit 0

## Done Criteria
- Entity filter is a `<select>` matching Action filter style
- Options derived from data
- "All" option shows everything
- Existing filter behavior preserved

## STOP Conditions
- None

## Dependencies
- None

# Plan 021 — Inline validation

**Source**: Finding #9 from UI/UX audit

**Goal**: Show field-level error messages next to inputs rather than only in the top message bar

## Steps
1. Create a small `FieldError` component (`<p className="text-red-500 text-sm">{message}</p>`)
2. In forms (AddDocument, Users, MasterData, Profile), restructure validation to associate errors with specific fields
3. Render `FieldError` below each invalid input
4. Keep top-level message for general/non-field errors

## Verification
- `pnpm verify` exit 0

## Done Criteria
- Invalid fields show red error text below them
- Top message bar still shows for general errors
- Validation still prevents submission

## STOP Conditions
- Scope creep into full form validation library migration

## Dependencies
- None

# Plan 020 — Required field indicators

**Source**: Finding #8 from UI/UX audit

**Goal**: Add visual asterisk (`*`) indicator to all required form fields across the app

## Steps
1. Identify all `<label>` tags or form labels in forms (AddDocument, Users, MasterData, Profile)
2. Add `*` after label text where field has `required` attribute or is semantically required
3. Ensure aria labels reflect required state (`aria-required="true"`)
4. Style asterisk in red (`text-red-500`)

## Verification
- `pnpm verify` exit 0

## Done Criteria
- All required fields show red asterisk
- Asterisk is part of label, visually distinct
- No visual change on optional fields

## STOP Conditions
- None

## Dependencies
- None

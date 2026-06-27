# Plan 027 — Form component deduplication

**Source**: Finding #15 from UI/UX audit

**Goal**: Extract duplicated `FormTitle`, `Status`, `IconButton`, `TextField` from `Users.tsx` and `MasterData.tsx` into shared components

## Steps
1. Identify exact duplicated component definitions in both files
2. Create `src/components/forms/FormTitle.tsx`, `Status.tsx`, `IconButton.tsx`, `TextField.tsx`
3. Update `Users.tsx` and `MasterData.tsx` to import shared versions
4. Remove local definitions
5. Search for further duplication opportunities across all pages

## Verification
- `pnpm verify` exit 0

## Done Criteria
- No duplicated form component definitions
- Both pages import from shared `src/components/forms/`
- Behavior unchanged

## STOP Conditions
- Scope creep into full design system migration

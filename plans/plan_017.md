# Plan 017 — AuditLog pagination total count

**Source**: Finding #5 from UI/UX audit

**Goal**: Add total count to AuditLog query result and display "Page X of Y" or "Showing X-Y of Z" in pagination

## Steps
1. Modify AuditLog backend query to include `COUNT(*) OVER()` for total count matching current filters
2. Return total count alongside existing results
3. Update frontend `AuditLog.tsx` pagination to display "Page X of Y" and "Showing X-Y of Z"
4. Update types if needed

## Verification
- `cargo build` clean
- `pnpm verify` exit 0

## Done Criteria
- Pagination shows contextual info ("Page 1 of 5", "Showing 1-50 of 247")
- Works correctly when filters change (total updates)

## STOP Conditions
- None

## Dependencies
- None

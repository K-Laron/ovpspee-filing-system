# Plan 004: Route-level code splitting

> **Executor instructions**: Follow step by step. When done, update `plans/README.md`.
>
> **Drift check**: `git diff --stat 9b4b638..HEAD -- src/App.tsx src/pages/`

## Status

- **Priority**: P2
- **Effort**: S
- **Risk**: LOW
- **Depends on**: 002 (verify script to confirm it works)
- **Category**: perf
- **Planned at**: commit `9b4b638`, 2026-06-27

## Why this matters

All 17 page components are eagerly imported in `App.tsx`, regardless of user role. Admin users download Secretary-only pages and vice versa, adding ~15-25 KB gzipped dead code per role. Code splitting via `React.lazy` defers loading to when the route is actually visited.

## Current state

`src/App.tsx` has 17 static imports at the top (lines 4–26):

```typescript
import { AuditLog } from './pages/admin/AuditLog';
import { BackupRestore } from './pages/admin/BackupRestore';
// ...14 more imports
import { ScanIntake } from './pages/secretary/ScanIntake';
```

Each import is eagerly resolved at module evaluation time. All 17 page components and their transitive dependencies are included in the initial bundle.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Build | `pnpm build` | exit 0 |
| Test | `pnpm test` | all pass |
| Verify | `pnpm verify` | exit 0 |

## Scope

**In scope**:
- `src/App.tsx` — replace static imports with `React.lazy`

**Out of scope**:
- No changes to page components themselves
- No changes to routing structure
- No changes to layouts (AdminLayout, SecretaryLayout, GuestLayout — these should stay eager since they're always rendered)

## Steps

### Step 1: Replace static imports with React.lazy

In `src/App.tsx`, replace each page component static import with `React.lazy`:

```typescript
import { lazy } from 'react';

const AuditLog = lazy(() => import('./pages/admin/AuditLog'));
const BackupRestore = lazy(() => import('./pages/admin/BackupRestore'));
const DeviceSettingsPage = lazy(() => import('./pages/admin/DeviceSettings'));
const MasterData = lazy(() => import('./pages/admin/MasterData'));
const MobileDevices = lazy(() => import('./pages/admin/MobileDevices'));
const TrashManagement = lazy(() => import('./pages/admin/TrashManagement'));
const Users = lazy(() => import('./pages/admin/Users'));
const FirstRunSetup = lazy(() => import('./pages/FirstRunSetup'));
const GuestLanding = lazy(() => import('./pages/GuestLanding'));
const Login = lazy(() => import('./pages/Login'));
const Profile = lazy(() => import('./pages/Profile'));
const AddDocument = lazy(() => import('./pages/secretary/AddDocument'));
const Documents = lazy(() => import('./pages/secretary/Documents'));
const MobileSubmissions = lazy(() => import('./pages/secretary/MobileSubmissions'));
const MyActivity = lazy(() => import('./pages/secretary/MyActivity'));
const ScanIntake = lazy(() => import('./pages/secretary/ScanIntake'));
```

Keep the layout imports eager (they're always rendered):
```typescript
import { AdminLayout } from './components/layout/AdminLayout';
import { GuestLayout } from './components/layout/GuestLayout';
import { ProtectedRoute } from './components/layout/ProtectedRoute';
import { SecretaryLayout } from './components/layout/SecretaryLayout';
```

Also keep eager: `invoke`, `SessionPayload`, `useSessionStore` — these are used in the component body.

### Step 2: Wrap routes in Suspense

In the JSX, wrap the `<Routes>` block with `<Suspense>`:

```typescript
return (
  <Suspense fallback={
    <div className="flex h-screen items-center justify-center bg-background text-secondary">
      <div className="rounded border border-border bg-surface px-5 py-4 shadow-sm">
        Loading...
      </div>
    </div>
  }>
    <Routes>
      {/* ... existing routes ... */}
    </Routes>
  </Suspense>
);
```

Use the same styling pattern as the existing `checking` state in `App.tsx` (lines 64–70) for visual consistency.

**Verify**: `pnpm verify` — exits 0, bundle builds without errors.

## Test plan

- Existing tests should pass. `React.lazy` works transparently with vitest — no test changes needed.
- Verify that navigating to each route renders the expected page (manual check is fine since this is a UI-only change).

## Done criteria

- [ ] `pnpm verify` exits 0
- [ ] `pnpm test` all pass
- [ ] Build output shows separate chunk files per lazy-loaded route (check `dist/` for multiple JS files after build)
- [ ] `grep -rn "^import.*from.*pages" src/App.tsx | grep -c "lazy"` returns a number ≥ 1 (at least some pages lazy-loaded) — or simply that the static page imports in the import block are replaced.
- [ ] `plans/README.md` status row updated

## STOP conditions

- If `React.lazy` causes type errors with named exports, the fix is `React.lazy(() => import('./path').then(m => ({ default: m.ComponentName })))` — but most page components are `export const Component` style. Check one first.
- If any layout component needs splitting (unlikely — layouts are small), keep it eager.

## Maintenance notes

- New page components added to `src/App.tsx` should be lazy-loaded following this pattern. Add this to `docs/07_Developer_Guidelines.md` step 8 checklist.
- The Suspense fallback matches the app's existing loading state styling. Keep them visually consistent.

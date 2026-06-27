# Plan 011: Frontend tests for core pages

> **Executor instructions**: Follow step by step. When done, update `plans/README.md`.
>
> **Drift check**: `git diff --stat 9b4b638..HEAD -- src/pages/`

## Status

- **Priority**: P3
- **Effort**: L
- **Risk**: LOW
- **Depends on**: 002 (verify script to validate)
- **Category**: tests
- **Planned at**: commit `9b4b638`, 2026-06-27

## Why this matters

18 of 21 frontend source files have zero tests. The core workflow pages (Documents, Login, ScanIntake, AddDocument, Profile, all admin pages, App shell) are uncovered. A regression in data fetching, state management, or rendering on any of these pages goes undetected until manual testing. This plan adds coverage for the highest-traffic pages.

## Current state

**Files with tests** (exemplars to follow):
- `src/components/ConfirmDialog.test.tsx` — renders, click handlers, keyboard events
- `src/components/EmptyState.test.tsx` — renders different variants
- `src/pages/GuestLanding.test.tsx` — mocks invoke, tests rendering with data
- `src/pages/secretary/MobileSubmissions.test.tsx` — mocks invoke, tests filters
- `src/lib/errors.test.ts`, `src/lib/dates.test.ts`, `src/lib/passwords.test.ts` — pure function tests

**Mocking pattern** (from `MobileSubmissions.test.tsx`):
```typescript
import { invoke } from '@tauri-apps/api/core';
vi.mocked(invoke).mockResolvedValue([]);
```

**Files needing tests** (priority-ordered):
1. `src/pages/Login.tsx` — login form, session creation, error display
2. `src/pages/secretary/Documents.tsx` — document list, filters, detail view, trash
3. `src/pages/secretary/ScanIntake.tsx` — scan list, import dialog, preview
4. `src/pages/Login.tsx` — login form

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| TS test | `pnpm test` | all pass |
| TS typecheck | `pnpm tsc --noEmit` | exit 0 |
| Verify | `pnpm verify` | exit 0 |

## Scope

**In scope** (create test files):
- `src/pages/Login.test.tsx` — login form test
- `src/pages/secretary/Documents.test.tsx` — document list + detail test

**Out of scope**:
- Admin pages (lower traffic, complex setup — defer)
- AddDocument test (complex file picker interaction — defer)
- Profile test (mostly form fields — lower priority)
- ScanIntake test (complex — defer)
- Integration tests with real Tauri backend (not possible in vitest/jsdom)

## Steps

### Step 1: Create Login.test.tsx

Create `src/pages/Login.test.tsx` modeled after `GuestLanding.test.tsx`.

The Login page:
- Renders a username + password form
- Has a submit handler that calls `invoke('login', {...})`
- Shows error messages from `getUserErrorMessage`
- Navigates on successful login (uses `useNavigate`)

```typescript
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import { Login } from './Login';

// Mock the router
const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const renderLogin = () =>
  render(
    <MemoryRouter>
      <Login />
    </MemoryRouter>
  );

describe('Login', () => {
  it('renders login form', () => {
    renderLogin();
    expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /login/i })).toBeInTheDocument();
  });

  it('shows error on failed login', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValueOnce('ERR_UNAUTHORIZED');
    renderLogin();
    fireEvent.change(screen.getByLabelText(/username/i), { target: { value: 'admin' } });
    fireEvent.change(screen.getByLabelText(/password/i), { target: { value: 'wrong' } });
    fireEvent.click(screen.getByRole('button', { name: /login/i }));
    await waitFor(() => {
      expect(screen.getByText(/invalid/i)).toBeInTheDocument();
    });
  });

  it('navigates on successful login', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValueOnce({
      session_id: 'abc', role: 'Admin', user_id: 1, display_name: 'Admin', profile_pic_path: null,
    });
    renderLogin();
    fireEvent.change(screen.getByLabelText(/username/i), { target: { value: 'admin' } });
    fireEvent.change(screen.getByLabelText(/password/i), { target: { value: 'password' } });
    fireEvent.click(screen.getByRole('button', { name: /login/i }));
    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalled();
    });
  });
});
```

**Verify**: `pnpm test` — Login tests pass.

### Step 2: Create Documents.test.tsx

Create `src/pages/secretary/Documents.test.tsx`.

This test is more complex because Documents integrates with `useSearchParams`, `useNavigate`, and has multiple views (active, trash).

```typescript
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import { Documents } from './Documents';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

const mockDocuments = [
  {
    document_id: 1, document_name: 'Test Doc', category_id: 1, category_name: 'BAC',
    folder_id: null, folder_name: null, office_id: null, office_name: null,
    date_received: '2026-01-15', date_added: '2026-01-15', remarks: null,
    status: 'Filed', is_hidden: false, is_trashed: false, attachment_count: 0,
    created_by: 1, created_by_name: 'Admin', updated_at: '2026-01-15',
  },
];

describe('Documents', () => {
  it('renders document list', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(mockDocuments);
    render(
      <MemoryRouter initialEntries={['/s/documents']}>
        <Documents />
      </MemoryRouter>
    );
    await waitFor(() => {
      expect(screen.getByText('Test Doc')).toBeInTheDocument();
    });
  });
});
```

Depending on the actual implementation, you may need to mock more hooks (session store, search params). Adjust based on the actual `Documents.tsx` imports and behavior.

**Verify**: `pnpm test` — Documents test passes.

### Step 3: Ensure mock patterns are consistent

All test files should follow the same pattern:
- Mock `@tauri-apps/api/core` at the top of the test file
- Mock `react-router-dom` hooks (`useNavigate`, `useSearchParams`) as needed
- Render with `<MemoryRouter>`
- Use `waitFor` for async operations (Tauri invoke calls)
- Use `fireEvent` for user interactions (not `@testing-library/user-event` — that dependency was removed)

## Test plan

- Login.test.tsx: 3 tests (renders form, shows error, navigates on success)
- Documents.test.tsx: 1 test initially (renders list) — expand with filter/view tests as time allows
- All existing tests must still pass

## Done criteria

- [ ] `pnpm test` — all tests pass, including new files
- [ ] `pnpm tsc --noEmit` exits 0
- [ ] `ls src/pages/Login.test.tsx src/pages/secretary/Documents.test.tsx` — both files exist
- [ ] `plans/README.md` status row updated

## STOP conditions

- If the Login or Documents page has complex internal state (useReducer, refs, etc.) that's hard to mock, test only the rendering and simple interactions. Don't try to cover every edge case.
- If `useSessionStore` (zustand) causes issues in tests, mock it: `vi.mock('../../store/sessionStore', () => ({ useSessionStore: vi.fn(() => ({ sessionId: 'test-session', clearSession: vi.fn() })) }))`.

## Maintenance notes

- New pages should include a co-located `.test.tsx` file as a convention. Update `docs/07_Developer_Guidelines.md` to mention this.
- The `vi.mocked(invoke).mockResolvedValue(...)` pattern is the standard way to provide test data. For error cases, use `mockRejectedValue`.
- As the app grows, consider extracting data-fetching hooks per domain to make testing easier — but that's a future refactoring, not needed for this plan.

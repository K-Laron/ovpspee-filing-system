# Frontend Component Documentation
## OVPSPEE Filing & Tracking System — CDHP Document 04

---

## 1. Project Structure

```
src/
  assets/
    fonts/            ← Inter, JetBrains Mono (bundled)
    images/           ← UEP logo, empty state illustrations
  components/
    ui/               ← Shadcn/ui base components (installed via CLI, first-party)
    layout/           ← Shell, Sidebar, TopNav, Breadcrumb
    common/           ← Shared reusable components (StatusBadge, ConfirmDialog, EmptyState, etc.)
    forms/            ← Shared form components (FormTitle, Status, IconButton, TextField, FieldError)
    documents/        ← Document-specific components
    scan-intake/      ← Scan Intake page components
    admin/            ← Admin-only page components
    Breadcrumbs.tsx   ← Navigation breadcrumb component
    Skeleton.tsx      ← Pulse-animated loading placeholder
    TableSkeleton.tsx ← Table-shaped skeleton grid
    Toast.tsx         ← Global toast notification system (context + provider + container)
  pages/
    GuestLanding.tsx
    Login.tsx
    FirstRunSetup.tsx
    secretary/
      Dashboard.tsx  (planned, not implemented — status counts integrated into Documents.tsx)
      Documents.tsx
      ScanIntake.tsx
      AddDocument.tsx
      Profile.tsx
      MyActivity.tsx
    admin/
      Users.tsx
      MasterData.tsx
      AuditLog.tsx
      BackupRestore.tsx
      Profile.tsx
      DeviceSettings.tsx
      MobileDevices.tsx
      TrashManagement.tsx
  lib/
    invoke (from `@tauri-apps/api/core`)
    errors.ts         ← handleError() utility
    helpers.ts        ← nullable, fileNameFromPath, normalizeSelectedPaths, safeFileName, sizeLabel, extensionFromName, formatBytes
    ConfirmDialog     ← inline useState<ConfirmAction | null>
    utils.ts          ← cn(), formatDate(), formatFileSize()
  store/
    sessionStore.ts   ← Zustand store for session state
    uiStore.ts        ← view mode (icon/list)
  styles/
    globals.css       ← CSS variables, Tailwind base
  App.tsx             ← Router root + ToastProvider
  main.tsx            ← Tauri app entry point
```

---

## 2. State Management

Use **Zustand** for global state (lightweight, no boilerplate). Use local `useState` for component-local UI state. Use React Query (TanStack Query) for data fetching and caching Tauri command responses.

### `sessionStore.ts`

```typescript
interface SessionState {
  sessionId: string | null;
  userId: number | null;
  role: 'Admin' | 'Secretary' | null;
  displayName: string;
  profilePicPath: string | null;
  setSession: (payload: SessionPayload) => void;
  clearSession: () => void;
}
```

### `uiStore.ts`

```typescript
interface UIState {
  viewMode: 'icon' | 'list';          // Documents page view toggle
  breadcrumb: BreadcrumbItem[];        // Current navigation path
  setViewMode: (mode: 'icon' | 'list') => void;
  setBreadcrumb: (items: BreadcrumbItem[]) => void;
}

interface BreadcrumbItem {
  label: string;
  onClick: () => void;
}
```

---

## 3. Routing

Use **React Router v6** (browser-history mode is not needed — use memory router since this is a desktop app).

```typescript
// App.tsx route structure
<Routes>
  {/* No-login Staff/Head Viewer routes — no auth required; share GuestLayout (top nav only, no sidebar) */}
  <Route element={<GuestLayout />}>
    <Route path="/"                                              element={<GuestLanding />} />
    <Route path="/category/:categoryId"                          element={<GuestCategoryView />} />
    <Route path="/category/:categoryId/folder/:folderId"         element={<GuestFolderView />} />
    <Route path="/document/:documentId"                          element={<GuestDocumentDetail />} />
  </Route>

  <Route path="/login"               element={<Login />} />
  <Route path="/first-run"           element={<FirstRunSetup />} />

  {/* Secretary routes — protected by RoleGuard role="Secretary" */}
  <Route element={<SecretaryLayout />}>
    {/* ponytail: Dashboard page not separately implemented; status counts shown on Documents page */}
    <Route path="/s/documents"       element={<Documents />} />
    <Route path="/s/scan-intake"     element={<ScanIntake />} />
    <Route path="/s/add-document"    element={<AddDocument />} />
    <Route path="/s/profile"         element={<SecretaryProfile />} />
    <Route path="/s/my-activity"     element={<MyActivity />} />
  </Route>

  {/* Admin routes — protected by RoleGuard role="Admin" */}
  <Route element={<AdminLayout />}>
    <Route path="/a/users"           element={<Users />} />
    <Route path="/a/master-data"     element={<MasterData />} />
    <Route path="/a/audit-log"       element={<AuditLog />} />
    <Route path="/a/backup"          element={<BackupRestore />} />
    <Route path="/a/profile"         element={<AdminProfile />} />
  </Route>
</Routes>
```

### `GuestLayout`

Wraps all guest-facing pages. Contains only the top nav (UEP logo on the left, Login button on the right). No sidebar. Content area is full-width with `p-6` padding.

```typescript
// components/layout/GuestLayout.tsx
const GuestLayout = () => (
  <div className="flex flex-col h-screen bg-background">
    <GuestTopNav />
    <main className="flex-1 overflow-y-auto p-6">
      <Outlet />
    </main>
  </div>
);
```

### RoleGuard

```typescript
// components/layout/RoleGuard.tsx
function RoleGuard({ role, children }: { role: 'Secretary' | 'Admin'; children: ReactNode }) {
  const { role: userRole } = useSession();
  if (userRole !== role) return <Navigate to="/" replace />;
  return <>{children}</>;
}
```

---

## 4. Layout Components

### `SecretaryLayout` / `AdminLayout`

Both layouts share the same shell structure. Differences are the sidebar nav items.

```typescript
// components/layout/AppShell.tsx
function AppShell({ navItems, children }: AppShellProps) {
  return (
    <div className="flex h-screen overflow-hidden bg-background">
      <Sidebar navItems={navItems} />
      <main className="flex-1 overflow-y-auto p-6">
        {children}
      </main>
    </div>
  );
}
```

### `Sidebar`

```typescript
interface NavItem {
  label: string;
  icon: LucideIcon;
  path: string;
}
// Always shows UEP logo at top.
// Active item highlighted with primary color left border + tinted background.
// Profile and Logout always at bottom, separated by a divider.
```

### `Breadcrumb`

```typescript
// components/layout/Breadcrumb.tsx
// Renders: Documents › BAC › PPMP 2025
// Each segment (except the last) is clickable.
// Separator: <ChevronRight size={14} className="text-muted" />
```

---

## 5. Common Components

### `StatusBadge`

```typescript
// components/common/StatusBadge.tsx
interface StatusBadgeProps {
  status: 'Filed' | 'Archived' | 'Confidential' | 'Other' | 'Hidden' | 'Trashed';
  label?: string;  // Optional override for 'Other' status
}
// Renders a colored pill badge per the design system color rules
```

---

### `ConfirmDialog`

```typescript
// components/common/ConfirmDialog.tsx
interface ConfirmDialogProps {
  open: boolean;
  title: string;
  description: string;
  confirmLabel?: string;      // Default: "Confirm"
  cancelLabel?: string;       // Default: "Cancel"
  variant?: 'default' | 'destructive';
  requiredText?: string;      // If set, user must type this string to enable confirm
  onConfirm: () => void;
  onCancel: () => void;
}
// Always required before any destructive action (trash, purge, delete, restore)
// When requiredText is set, the confirm button is disabled until the user types
// the exact string. Used for high-severity actions: purge, restore, revoke device.
```

---

### `EmptyState`

```typescript
// components/common/EmptyState.tsx
interface EmptyStateProps {
  title: string;
  description?: string;
  action?: ReactNode;  // Optional CTA button
  illustration?: 'documents' | 'folder' | 'search' | 'scan';
}
// Centered in its container; uses SVG illustrations from src/assets/images/
```

---

### `Breadcrumbs`

```typescript
// components/Breadcrumbs.tsx
interface BreadcrumbSegment {
  label: string;
  href?: string;          // Omitted for current (last) segment
}

interface BreadcrumbsProps {
  segments: BreadcrumbSegment[];
}
// Renders: Label1 › Label2 › Label3
// Last segment is plain text (current page)
// Non-last segments are React Router <Link> for navigation
// Used on all sub-pages with depth > 1: Users › Edit, Documents › Category, etc.
```

---

### `Skeleton` / `TableSkeleton`

```typescript
// components/Skeleton.tsx
interface SkeletonProps {
  className?: string;
}
// Renders: <div className="animate-pulse bg-gray-200 rounded" />
// Generic pulse-animated placeholder for any content shape

// components/TableSkeleton.tsx
interface TableSkeletonProps {
  rows?: number;    // Default: 5
  columns?: number; // Default: 4
}
// Renders a grid of Skeleton cells matching table column layout
// Column widths vary (first column wider, last narrower) for natural look
// Replaces "Loading..." text across all list/table pages
```

---

### `Toast` — Global Notification System

```typescript
// components/Toast.tsx
type ToastType = 'success' | 'error' | 'info';

// Usage via context hook:
const { addToast, removeToast } = useToast();
addToast('success', 'Document filed successfully.');
addToast('error', 'Failed to save document.');
addToast('info', 'Scan import completed.');

// ToastContainer renders at the app root (inside ToastProvider):
//   position: fixed, top-4 right-4, z-50
//   Stacked vertically, newest at top
//   Auto-dismiss after 5 seconds
//   Each toast: colored left border (green/red/blue), text, close button
//   Clicking toast dismisses it immediately

// App.tsx wraps the app:
<ToastProvider>
  <Router>
    <Routes>...</Routes>
  </Router>
  <ToastContainer />
</ToastProvider>
```

Supersedes the previous inline `setMessage<string | null>(...)` pattern across all pages. Each page imports `useToast` and calls `addToast` instead.

---

### Form Components (`src/components/forms/`)

Shared form components extracted from duplicated local definitions in Users.tsx, MasterData.tsx, and Profile.tsx.

```typescript
// components/forms/FormTitle.tsx
interface FormTitleProps {
  children: React.ReactNode;
}
// Renders: <h1 className="text-xl font-semibold">{children}</h1>

// components/forms/Status.tsx
interface StatusProps {
  status: string;  // e.g., 'Active', 'Inactive'
}
// Renders a colored pill badge (green for active, slate for inactive)

// components/forms/IconButton.tsx
interface IconButtonProps {
  icon: React.ReactNode;  // Lucide icon component instance
  onClick: () => void;
  label: string;          // aria-label for accessibility
}
// Renders a ghost-style icon button

// components/forms/TextField.tsx
interface TextFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  error?: string | null;    // Field-level error message
  required?: boolean;       // Shows red asterisk on label
  type?: string;            // Input type (default: 'text')
  placeholder?: string;
}
// Renders: label (with * if required) + <input> + FieldError (if error set)
// Adds aria-required, aria-invalid, aria-describedby for accessibility

// components/forms/FieldError.tsx
interface FieldErrorProps {
  message?: string | null;
}
// Renders: <p className="text-red-500 text-sm" role="alert">{message}</p>
// Returns null if no message (renders nothing)

// Barrel export:
import { FormTitle, Status, IconButton, TextField, FieldError } from '../components/forms';
```

---

### `SearchFilterBar`

```typescript
// components/common/SearchFilterBar.tsx
interface SearchFilterBarProps {
  onSearch: (query: string) => void;
  filters: FilterDefinition[];
  activeFilters: ActiveFilter[];
  onFilterChange: (filters: ActiveFilter[]) => void;
  onSortChange: (sort: SortOption) => void;
  currentSort: SortOption;
}
// Renders: Search input | Sort dropdown | Filter dropdown | Active filter chips
// Chips have × button to remove individual filters
// Debounces search input 300ms
```

**Simpler search pattern:** Some pages (TrashManagement, ScanIntake dropdown) use a lightweight inline search with `useState` + `Array.filter()` instead of the full SearchFilterBar. This is appropriate for single-field client-side filtering without sort/filter options.


---

### `AttachmentPreview`

```typescript
// components/common/AttachmentPreview.tsx
interface AttachmentPreviewProps {
  attachments: AttachmentItem[];
  sessionId: string | null;
  currentPage: number;
  onPageChange: (page: number) => void;
}
// Lazy-loads attachment pages — does not load all at once
// Renders: "PAGE {n} of {total}" navigation with prev/next buttons
// Supports images (rendered as <img>) and PDFs (rendered via object tag or canvas)
// Shows file name and size below the preview area
```

---

### `DocumentCard` (List Item)

```typescript
// components/documents/DocumentCard.tsx
interface DocumentCardProps {
  document: DocumentListItem;
  onView: () => void;
  onEdit?: () => void;          // Secretary only
  onMove?: () => void;          // Secretary only
  onHide?: () => void;          // Secretary only
  onTrash?: () => void;         // Secretary only
  onExportPdf: () => void;
  showActions: boolean;         // false for Staff/Head Viewer
}
// Renders document name, category, folder, sender, date received, status badge
// Actions on hover via kebab "⋮" menu
// Hidden documents show a faint <EyeOff> icon in the top-right corner (Secretary only)
```

---

## 6. Pages — Staff/Head Viewer Views

The no-login Staff/Head Viewer experience has four pages. All share `GuestLayout` (top nav only, no sidebar). Navigation between them uses React Router links and the breadcrumb component.

### `GuestLanding.tsx` — Landing Page (default on app open)

```typescript
// pages/GuestLanding.tsx
// Renders two sections:
//
// 1. RECENT DOCUMENTS
//    - Heading: "Recent Documents"
//    - Global search bar (searches across ALL public, non-hidden, non-trashed documents)
//    - Flat list of the latest 10 documents:
//        Columns: Document Name | Category › Folder | Date Received | Status Badge | →
//        Clicking any row → navigate('/document/:documentId')
//    - "Show more" link → navigates to a full search results view (future; for MVP
//      the list simply shows the last 10 and the search bar handles discovery)
//
// 2. BROWSE BY CATEGORY
//    - Heading: "Browse by Category"
//    - Responsive card grid (grid-cols-2 sm:grid-cols-3 lg:grid-cols-4)
//    - One CategoryCard per active, non-system category
//    - Sorted alphabetically
//    - Clicking a card → navigate('/category/:categoryId')
//
// Data fetching:
//   cmd<DocumentItem[]>('list_public_documents', { search, categoryId, folderId })
//
// UX features:
//   Debounced auto-search (300ms) — type in search, results update automatically.
//   `/` key focuses search from anywhere.
//   Clear button resets search and reloads.
//
// Empty states:
//   No recent documents → "No documents have been filed yet."
//   No categories → "No categories have been set up yet."
```

### `CategoryCard`

```typescript
// components/documents/CategoryCard.tsx
interface CategoryCardProps {
  category: CategoryItem;
  onClick: () => void;
}
// Displays: category icon (Lucide), category name, document count ("12 documents")
// Left border or top border accent using category.color_code
// Hover: slight shadow lift and border darkens
// Size: fixed height 120px, fills grid column width
```

---

### `GuestCategoryView.tsx` — Folder Grid for a Category

```typescript
// pages/GuestCategoryView.tsx
// Route param: categoryId
//
// Breadcrumb: Home › {categoryName}
//
// Renders:
//   - Page heading: category name (with color accent and icon)
//   - Responsive folder card grid (same grid breakpoints as CategoryCard grid)
//   - One FolderCard per active folder in this category
//   - Sorted alphabetically by folder name
//   - Clicking a folder card → navigate('/category/:categoryId/folder/:folderId')
//
// Empty state: "No folders in this category yet."
//
// Data fetching:
//   useQuery(['public-categories']) → list_public_categories() + find by ID (for heading)
//   useQuery(['folders', categoryId]) → list_folders({ category_id: categoryId })
```

### `FolderCard`

```typescript
// components/documents/FolderCard.tsx
interface FolderCardProps {
  folder: FolderItem;
  onClick: () => void;
}
// Displays: folder icon, folder name, document count
// Color accent from folder.folder_color
// Same size and hover behavior as CategoryCard
```

---

### `GuestFolderView.tsx` — Document List for a Folder

```typescript
// pages/GuestFolderView.tsx
// Route params: categoryId, folderId
//
// Breadcrumb: Home › {categoryName} › {folderName}
//
// Renders:
//   - Heading: folder name
//   - Search bar (scoped to this folder only)
//   - Sort dropdown: Date Received (desc default) | Document Name (A-Z) | Date Filed
//   - Filter dropdown: Status (Filed, Archived, Confidential, Other)
//   - Active filter chips (removable)
//   - Document list (rows):
//       Document Name | Status Badge | Date Received | →
//       Clicking a row → navigate('/document/:documentId')
//   - Pagination: 25 per page, "Previous / Next" controls at the bottom
//
// Empty state: "No documents in this folder." (or "No results match your search.")
//
// Data fetching:
//   useQuery(['documents', folderId, search, filters, sort, page])
//     → list_documents({ folder_id: folderId, session_id: null, ... })
```

---

### `GuestDocumentDetail.tsx` — Full Document Detail Page

```typescript
// pages/GuestDocumentDetail.tsx
// Route param: documentId
//
// Breadcrumb: Home › {categoryName} › {folderName} › {documentName}
//   (each segment is a clickable link back to the respective view)
//
// Renders:
//
//   HEADER ROW
//   - Document name (h1)
//   - Status badge
//   - [Export PDF] button (top-right) → calls export_document_pdf → Tauri save dialog
//
//   METADATA SECTION (two-column layout on wide screens)
//   - Sender:         Jane Doe
//   - Office:         BAC Office
//   - Receiver:       —  (shows dash if empty)
//   - Date Received:  May 12, 2026
//   - Date Filed:     May 13, 2026
//   - Remarks:        [text or dash]
//
//   ATTACHMENTS SECTION
//   - Heading: "Attachments ({count})"
//   - AttachmentPreview component (paginated, lazy-loaded)
//   - "PAGE N of N" navigation with ← → buttons
//   - File name and size shown below the preview area
//
// Data fetching:
//   useQuery(['document', documentId]) → get_document({ document_id: documentId, session_id: null })
//
// Guard: if document is hidden (is_hidden=1) or trashed (is_trashed=1) and user is Staff/Head Viewer,
//   the backend returns ERR_NOT_FOUND → display a "Document not found" error page.
//
// Back navigation: clicking breadcrumb segments. No dedicated "Back" button
//   (breadcrumb serves that purpose).
```

---

### `CategoryTabStrip` — REMOVED

The `CategoryTabStrip` component described in the original plan is **removed**. Category navigation for guests is now handled entirely through the `GuestLanding` category card grid and the breadcrumb. There is no horizontal tab strip in the no-login viewer layout.

---

## 7. Pages — Secretary

### `Dashboard.tsx`

Not implemented as a separate page. Document status counts are rendered inline at the top of `Documents.tsx` as a summary bar (counts by Filed/Archived/Confidential/Other from the current search results). See Documents.tsx for the inline summary implementation.

---

### `Documents.tsx`

Secretary's primary file system view. File-explorer style navigation: category card grid → folder card grid → document list → document detail. The structure mirrors the Staff/Head Viewer experience but with additional visibility and action layers.

**Differences from Staff/Head Viewer view:**
- All categories shown including TRASH (always last, slate-colored)
- Hidden documents visible with `<EyeOff>` indicator (Secretary only)
- Document rows have a kebab `⋮` action menu
- View mode toggle (icon / list) in the top-right of each level
- Breadcrumb: `Documents › {Category} › {Folder}`

**Secretary document actions (kebab menu on each document row):**

| Action | Condition | Calls |
|---|---|---|
| View | Always | Navigate to document detail |
| Edit | Not trashed | Opens `EditDocumentPanel` (slide-over) |
| Hide / Unhide | Not trashed | `set_document_hidden` |
| Move | Not trashed | Opens `MoveDocumentDialog` |
| Trash | Not trashed | `ConfirmDialog` → `trash_document` |
| Restore | Trashed only | `ConfirmDialog` → `restore_document` |

**UX features added:**
- **Debounced auto-search** (300ms) on search input and all filter changes — no manual Apply needed.
- **`/` key** focuses the search input from anywhere (skips if already in an input/textarea/select).
- **Escape** closes the document detail panel.
- **Close (X) button** on detail panel header.
- **Window title** updates to document name when detail is open (e.g., "Invoice #42 — Documents").
- **Document count** shown next to Refresh button (e.g., "23 documents").
- **Empty states** for zero search results and empty trash (uses EmptyState component).
- **Paginated listing** with "Load More" button (offset-based, 25 per page) and total count from `COUNT(*) OVER()`.
- **No auto-select** first document on page load — user clicks to view details.
- **Skeleton loading** replaces "Loading..." text during data fetch.

**Bulk actions (checkbox column added):**
- Select-all checkbox in table header.
- "Trash selected" button appears when ≥1 row selected.
- `Promise.all` loop calls `trash_document` per ID (frontend-only, no backend batch needed).

**Note:** Purge actions are not available to Secretary at all — no Purge button is rendered for Secretary sessions. Purge is Admin-only.

---

### TRASH View (within `Documents.tsx`)

When the Secretary navigates into the TRASH category card, the content area switches to the TRASH view. TRASH has no folders — it shows a flat document list directly.

```
┌─────────────────────────────────────────────────────────────┐
│  Documents › TRASH                          [🗑 Empty Trash] │  ← Admin only
│  ──────────────────────────────────────────────────────────  │
│  Document Name A    BAC › PPMP 2025    Purges in 12 days  [⋮]│
│  Document Name B    BOR › Minutes      Purges in 3 days   [⋮]│  ← yellow/orange text
│  Document Name C    Tracer › 2024      Purges today       [⋮]│  ← red text
│  Document Name D    BAC › Contracts    Purges in 28 days  [⋮]│
└─────────────────────────────────────────────────────────────┘
```

**TRASH list columns:**
- Document Name
- Original Location (e.g. "BAC › PPMP 2025" — links back to that folder if it still exists)
- Purge Countdown (color-coded; see below)
- Actions kebab `⋮`

**Search:** Text input filters trashed documents by name client-side.

**Empty states:**
- No documents in trash → "Trash is empty"
- Search with no matches → "No documents match your search"

**Purge countdown color coding:**

| `days_until_purge` | Display text | Text color |
|---|---|---|
| > 7 | "Purges in N days" | `text-muted` (grey) |
| 4–7 | "Purges in N days" | `text-yellow-600` |
| 1–3 | "Purges in N days" | `text-orange-600` |
| 0 | "Purges today" | `text-red-600 font-semibold` |
| < 0 | "Overdue for purge" | `text-red-700 font-semibold` |
| `null` (disabled) | "Auto-purge off" | `text-muted` |

**TRASH actions by role:**

| Action | Secretary | Admin |
|---|---|---|
| View document detail | ✅ | ✅ |
| Restore | ✅ | ✅ |
| Purge (individual) | ❌ (hidden) | ✅ |
| Empty Trash (bulk) | ❌ (hidden) | ✅ |

**"Empty Trash" button:** Shown only to Admin. Positioned top-right of the TRASH view. Opens a `ConfirmDialog`:
> "Permanently delete all X trashed documents? This cannot be undone."
> [Cancel] [Empty Trash]

**Restore feedback toast:**
- Normal restore: `"Document restored to BAC › PPMP 2025."`
- Folder-missing fallback: `"Restored to BAC — original folder no longer exists. You may want to move it to a folder."` (info toast, not error)

**Purge confirmation dialog (Admin only):**
> "Permanently delete '{document name}'? All attachments will be deleted. This cannot be undone."
> [Cancel] [Delete Permanently]

### `MoveDocumentDialog`

```typescript
// components/documents/MoveDocumentDialog.tsx
// Shows current location: "BAC › PPMP 2025"
// Category dropdown (active non-TRASH categories)
// Folder dropdown (folders for selected category; dynamically loaded)
// Confirm → calls move_document
```

### `EditDocumentPanel`

```typescript
// components/documents/EditDocumentPanel.tsx
// Slide-over panel (right side, 480px wide)
// Same form fields as Add Document metadata section
// Add / remove attachments inline
// Reorder attachments via drag-and-drop (using dnd-kit)
// Save button → calls update_document
```

---

### `AddDocument.tsx`

Two-tab form: **Upload** and **From Scan Intake**.

```typescript
// Tab 1: Upload
//   File picker (multi-select) → shows thumbnail strip of selected files
//   Metadata form (document name, sender, office, receiver, date received, remarks,
//                  category, folder, status, hidden toggle)
//   Save → calls create_document with uploaded_files

// Tab 2: From Scan Intake
//   "Pick from Scan Intake" button → opens IntakePicker slide-over
//   Shows selected scans as thumbnail strip (same as Upload tab)
//   Metadata form (identical to Tab 1)
//   Save → calls create_document with scan_intake_ids

// Both tabs allow mixing: select some uploads AND some scans for one document.
// Thumbnails strip shows source tag: "📄 Uploaded" or "🖷 Scanned"
// Reorderable via drag-and-drop before save
```

---

### `ScanIntake.tsx`

Staging area for scanned pages. See `11_Scan_Intake_Specification.md` for full detail.

```typescript
// Layout:
//   Header: "Scan Intake" | "Import Scans" button | scan count badge
//   Thumbnail grid of unclaimed scans
//     - File name, scanned timestamp, file size
//     - Checkbox for multi-select
//   Toolbar (appears when ≥1 selected):
//     "Delete Selected" (destructive, ConfirmDialog required)
//   Empty state: illustration + "No scans in intake. Import scan files to begin."
// Polling: 30s auto-refresh, skips when tab hidden (document.hidden check)
// Preview: lazy-loads preview images for selected scan
// Status filter: status column filtered via backend SQL, not client-side .filter()
// Document dropdown: search input filters documents by name client-side
```

---

## 8. Pages — Admin

### `Users.tsx`

```typescript
// Table: Full Name | Username | Role | Last Login | Status | Actions
// "+ Add User" button → AddUserModal
// Edit icon → EditUserModal
// Search bar (debounced) + filter by Role + filter by Status
```

### `MasterData.tsx`

Three tabs: Categories, Folders, Offices.

```typescript
// Categories tab:
//   Table: Name | Color Swatch | Icon | Status | Actions
//   + Add Category button → AddCategoryModal (name, description, color picker, icon picker)
//   Edit → EditCategoryModal
//   System categories (TRASH) show a lock icon; Edit is disabled

// Folders tab:
//   Table: Folder Name | Category | Description | Status | Actions
//   + Add Folder button → AddFolderModal (name, category dropdown, description, color)
//   Category filter dropdown at top

// Offices tab:
//   Table: Office Name | Description | Status | Actions
//   Standard add/edit flow
```

---

### `AuditLog.tsx`

```typescript
// Table: Action | Description | Username | IP | Timestamp
// Search bar + filter (Action type, User, Date range)
// Pagination (50/page): "Page X of Y", "Showing start-end of total"
// Total count from backend: COUNT(*) OVER() window function
// "Export PDF" button → calls export_audit_log_pdf → Tauri save dialog
// ⚙ Settings icon → RetentionPolicyModal
//   - Input: "Delete entries older than [N] months"
//   - "Save" + "Run Cleanup Now" button
```

---

### `BackupRestore.tsx`

Four sections within one scrollable page:

```typescript
// 1. Create Backup
//    - Destination folder picker
//    - "Create Backup Now" button + loading indicator
//    - Last backup: "May 12, 2026 at 2:30 PM — D:\Backups\ovpspee_backup_..."

// 2. Scheduled Backup
//    - Frequency: Disabled | Daily | Weekly
//    - Time picker
//    - Destination folder picker
//    - Retention count (keep last N backups)

// 3. Export / Import Portable Backup
//    - "Export Backup Archive" button → save .ovpspee-backup file
//    - "Import Backup Archive" button → file picker → validation → ConfirmDialog → restore

// 4. Restore from Backup Folder
//    - Folder picker
//    - Validation status (green = valid, red = invalid)
//    - "Restore" button (destructive) → ConfirmDialog → restore + app restart
```

---

## 9. Modals Reference

| Modal | Trigger | Primary Action |
|---|---|---|
| `AddUserModal` | + Add User (Admin > Users) | `create_user` |
| `EditUserModal` | Edit icon (Admin > Users) | `update_user` + `admin_reset_password` |
| `AddCategoryModal` | + Add Category | `create_category` |
| `EditCategoryModal` | Edit icon | `update_category` |
| `AddFolderModal` | + Add Folder | `create_folder` |
| `EditFolderModal` | Edit icon | `update_folder` |
| `AddOfficeModal` | + Add Office | `create_office` |
| `EditOfficeModal` | Edit icon | `update_office` |
| `MoveDocumentDialog` | Move action on document | `move_document` |
| `DocumentViewPanel` | View action / click document | `get_document` (read-only) |
| `EditDocumentPanel` | Edit action on document | `update_document` |
| `IntakePicker` | "Pick from Scan Intake" in Add Document | Returns selected intake_ids |
| `RetentionPolicyModal` | ⚙ on Audit Log page | `update_retention_setting` |
| `ConfirmDialog` | Any destructive action | Calls caller-specified command |

---

## 10. Typed Invoke Wrappers

All Tauri IPC calls use `{ invoke }` from `@tauri-apps/api/core` directly:

```typescript
import { invoke } from '@tauri-apps/api/core';
const result = await invoke<ReturnType>('command_name', { param });
```

---

## 11. Error Handling Pattern

```typescript
// src/lib/errors.ts
import { addToast } from '../components/Toast'; // or useToast() hook

export function handleError(error: unknown, fallback = 'An error occurred') {
  const message = typeof error === 'string' ? error : fallback;
  addToast('error', message);
  console.error('[AppError]', error);
}

// Usage in components:
import { useToast } from '../components/Toast';

const { addToast } = useToast();
try {
  await login(username, password);
  addToast('success', 'Login successful.');
} catch (e) {
  handleError(e, 'Login failed. Please check your credentials.');
}
```

---

## 12. Key Libraries Summary

| Library | Version | Purpose |
|---|---|---|
| `react` | 18+ | UI framework |
| `react-router-dom` | 6+ | Client-side routing (memory router) |
| `@tauri-apps/api` | 2.x | Tauri IPC bridge |
| `zustand` | 4+ | Global state (session, UI preferences) |
| `@tanstack/react-query` | 5+ | Data fetching, caching, invalidation |
| `shadcn/ui` | Latest | Base UI components |
| `tailwindcss` | 3.x | Utility CSS |
| `lucide-react` | Latest | Icons |
| `@dnd-kit/core` | Latest | Drag-and-drop (attachment reorder, scan picker) |
| `date-fns` | Latest | Date formatting utilities |

---

*End of Frontend Component Documentation*
*Next: `05_Testing_Strategy_Documentation.md`*

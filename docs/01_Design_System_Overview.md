# Design System Overview
## OVPSPEE Filing & Tracking System — CDHP Document 01

---

## 1. Component Library

| Tool | Version | Purpose |
|---|---|---|
| **Shadcn/ui** | Latest stable | Core UI components (dialogs, tables, inputs, buttons, badges, dropdowns) |
| **Tailwind CSS** | v3.x | Utility-first styling; no custom CSS framework |
| **Lucide React** | Latest | Icon set — consistent, accessible, tree-shakeable |
| **React** | 18+ | UI framework |
| **TypeScript** | 5+ | Type safety |

All components from Shadcn/ui are installed into `src/components/ui/` via the CLI and treated as first-party code (editable). Do not import from the registry at runtime.

---

## 2. Color Palette

### System Colors

| Token | Hex | Usage |
|---|---|---|
| `primary` | `#C0392B` (UEP Red) | Primary buttons, active states, key accents |
| `primary-foreground` | `#FFFFFF` | Text on primary background |
| `secondary` | `#2C3E50` | Sidebar, header backgrounds |
| `secondary-foreground` | `#FFFFFF` | Text on secondary |
| `background` | `#F5F6FA` | Main content area background |
| `surface` | `#FFFFFF` | Cards, modals, panels |
| `border` | `#E2E8F0` | Dividers, input borders |
| `muted` | `#94A3B8` | Placeholder text, secondary labels |
| `destructive` | `#E53E3E` | Delete actions, error states |
| `success` | `#38A169` | Confirmation toasts, success states |
| `warning` | `#DD6B20` | Warning dialogs |

All tokens are defined in `tailwind.config.ts` and in `src/styles/globals.css` as CSS custom properties.

### Category Colors

Categories have admin-assigned `color_code` values (hex). These appear as:
- Background tint on category cards in icon view
- Left border accent on category list items
- Tab background on the document browser

The system provides a color picker in the Admin interface. Recommended palette (pre-filled as defaults):

| Example Category | Suggested Color |
|---|---|
| BAC | `#2563EB` (Blue) |
| BOR | `#7C3AED` (Violet) |
| Tracer Studies | `#059669` (Green) |
| OVPSPEE General | `#D97706` (Amber) |
| TRASH | `#64748B` (Slate) — System assigned, not configurable |

---

## 3. Typography

| Role | Font | Size | Weight |
|---|---|---|---|
| App Title | Inter | 18px | 600 |
| Section Heading (h1) | Inter | 24px | 700 |
| Page Heading (h2) | Inter | 20px | 600 |
| Card Title | Inter | 14px | 600 |
| Body / Labels | Inter | 14px | 400 |
| Small / Metadata | Inter | 12px | 400 |
| Monospace (paths, IDs) | JetBrains Mono | 13px | 400 |

Font is loaded via the Tauri build — no CDN dependency at runtime. Both `Inter` and `JetBrains Mono` are bundled as static assets in `src/assets/fonts/`.

---

## 4. Spacing System

Tailwind default spacing scale applies (4px base unit). Key usage rules:

- **Page padding:** `p-6` (24px) on main content areas
- **Card padding:** `p-4` (16px)
- **Form field gaps:** `gap-4` (16px)
- **Modal padding:** `p-6` (24px)
- **Sidebar width:** Fixed at 240px (`w-60`)
- **Top nav height:** 56px (`h-14`)

---

## 5. Layout Modes

### 5.1 Staff/Head Viewer Layout (No Login)

The staff/head viewer experience has four distinct views, all sharing the same top nav. Navigation uses the breadcrumb — there is no sidebar.

**View 1 — Landing Page (default on app open)**
```
┌─────────────────────────────────────────────────────────┐
│  [UEP Logo]   OVPSPEE Filing and Tracking System  [Login]│  ← Top Nav (h-14)
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Recent Documents                    [Search Bar] [🔍]  │
│  ─────────────────────────────────────────────────────  │
│  Document Name          BAC › PPMP 2025    May 12, 2026  │
│  Document Name          BOR › Minutes      May 10, 2026  │
│  Document Name          Tracer › 2024      May 08, 2026  │
│  [ Show more ]                                           │
│                                                          │
│  Browse by Category                                      │
│  ─────────────────────────────────────────────────────  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │  ← Category Cards
│  │  📁 BAC  │  │  📁 BOR  │  │ 📁 Tracer│              │
│  │12 docs   │  │ 8 docs   │  │20 docs   │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────┘
```

**View 2 — Category View (after clicking a category card)**
```
┌─────────────────────────────────────────────────────────┐
│  [UEP Logo]   OVPSPEE Filing and Tracking System  [Login]│
├─────────────────────────────────────────────────────────┤
│  Home › BAC                          [Search Bar] [🔍]  │  ← Breadcrumb
│  ─────────────────────────────────────────────────────  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │  ← Folder Cards
│  │ PPMP 2025│  │Minutes   │  │ Contracts│              │
│  │ 5 docs   │  │ 3 docs   │  │ 4 docs   │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────┘
```

**View 3 — Folder View (after clicking a folder card)**
```
┌─────────────────────────────────────────────────────────┐
│  [UEP Logo]   OVPSPEE Filing and Tracking System  [Login]│
├─────────────────────────────────────────────────────────┤
│  Home › BAC › PPMP 2025    [Search] [Sort ▼] [Filter ▼] │  ← Breadcrumb + tools
│  ─────────────────────────────────────────────────────  │
│  Document Name A          Filed      May 12, 2026  [→]  │
│  Document Name B          Archived   Apr 28, 2026  [→]  │
│  Document Name C          Filed      Apr 15, 2026  [→]  │
└─────────────────────────────────────────────────────────┘
```

**View 4 — Document Detail Page (after clicking a document row)**
```
┌─────────────────────────────────────────────────────────┐
│  [UEP Logo]   OVPSPEE Filing and Tracking System  [Login]│
├─────────────────────────────────────────────────────────┤
│  Home › BAC › PPMP 2025 › Document Name A               │  ← Breadcrumb
│  ─────────────────────────────────────────────────────  │
│  Document Name A                      [Export PDF]       │
│                                                          │
│  Sender:       Jane Doe (BAC Office)                     │
│  Received:     May 12, 2026                              │
│  Status:       Filed                                     │
│  Remarks:      —                                         │
│                                                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │                                                  │   │  ← Attachment Preview
│  │             [Page preview renders here]          │   │     (paginated, lazy loaded)
│  │                                                  │   │
│  └──────────────────────────────────────────────────┘   │
│              ← PAGE 1 of 4 →                             │
└─────────────────────────────────────────────────────────┘
```

**Navigation rules:**
- Every breadcrumb segment is clickable and navigates back to that view.
- "Home" in the breadcrumb always returns to the Landing Page (Recent Documents + category grid).
- The Search bar on the Landing Page searches **across all public documents** (all categories/folders). Results show document name, category, folder, and date — clicking a result goes directly to the Document Detail page.
- The Search bar inside a Folder View searches **within that folder only**.
- There is no tab strip. Category navigation is always done through the card grid.

### 5.2 Secretary Layout (Authenticated)

```
┌──────────┬──────────────────────────────────────────────┐
│  Sidebar │  Content Area                                 │
│  (240px) │                                               │
│          │                                               │
│ [Logo]   │  [Page-specific content]                     │
│          │                                               │
│ Dashboard│                                               │
│ Documents│                                               │
│ Scan     │                                               │
│ Intake   │                                               │
│ Add Doc  │                                               │
│          │                                               │
│ ─────── │                                               │
│ Profile  │                                               │
│ Logout   │                                               │
└──────────┴──────────────────────────────────────────────┘
```

### 5.3 Admin Layout (Authenticated)

```
┌──────────┬──────────────────────────────────────────────┐
│  Sidebar │  Content Area                                 │
│  (240px) │                                               │
│          │                                               │
│ [Logo]   │  [Page-specific content]                     │
│          │                                               │
│ Users    │                                               │
│ Master   │                                               │
│  Data    │                                               │
│ Audit Log│                                               │
│ Backup & │                                               │
│ Restore  │                                               │
│          │                                               │
│ ─────── │                                               │
│ Profile  │                                               │
│ Logout   │                                               │
└──────────┴──────────────────────────────────────────────┘
```

---

## 6. Document Browser — Icon vs. List View

The Documents page (Secretary) and the Staff/Head Viewer landing page both support two view modes toggled by a button in the top-right of the content area.

### Icon View (default)

Categories displayed as icon cards in a responsive grid (`grid-cols-2 sm:grid-cols-3 lg:grid-cols-4`). Each card shows the category icon, name, and document count.

```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   📁 BAC     │  │   📁 BOR     │  │  📁 Tracer   │
│  12 documents│  │   8 documents│  │  20 documents│
└──────────────┘  └──────────────┘  └──────────────┘
```

### List View

Categories displayed as a vertical list with name, document count, and last modified date. Expanding a row shows folders; expanding a folder shows documents inline.

```
▶ BAC                           12 documents    May 12, 2026
▶ BOR                            8 documents    Apr 28, 2026
▶ Tracer Studies                20 documents    May 10, 2026
```

### File System Navigation

Clicking a category → shows its folders in icon or list view.
Clicking a folder → shows its documents in icon or list view.
Breadcrumb at the top of the content area tracks position:
`Documents › BAC › PPMP 2025`
Each breadcrumb segment is clickable.

---

## 7. Component Patterns

### Buttons

| Variant | Usage |
|---|---|
| `default` (red fill) | Primary action (Save, Create, Add) |
| `secondary` (outline) | Secondary action (Cancel, Back) |
| `destructive` (red outline → fill on hover) | Destructive action (Delete, Purge) |
| `ghost` | Icon-only buttons, toolbar actions |

All destructive actions must be followed by a confirmation dialog before execution. For high-severity actions (purge, restore, revoke device), the ConfirmDialog supports `requiredText` — user must type the exact string to enable the confirm button.

### Toasts

Use the custom `Toast` component (see `src/components/Toast.tsx`). Rules:
- Duration: 5 seconds auto-dismiss for all types
- Position: top-right, stacked vertically
- Types: success (green left border), error (red left border), info (blue left border)
- Usage via `useToast()` hook: `addToast('success', 'message')`, `addToast('error', 'message')`
- Supersedes the previous inline `setMessage()` pattern and the `sonner` library

### Loading States

Use `TableSkeleton` for table/list loading states instead of "Loading..." text:
- `TableSkeleton rows={5} columns={4}` renders an animated pulse grid
- `Skeleton` for individual placeholder elements (preview panels, text lines)
- Both use Tailwind `animate-pulse` for subtle animation

### Modals / Dialogs

Use Shadcn/ui `<Dialog>` for all modals. Rules:
- Width: `max-w-md` for simple forms, `max-w-2xl` for complex pickers
- Always include a close (×) button in the top right
- Footer: Cancel (secondary) on the left, Primary action on the right
- Destructive dialogs: include a warning icon and red primary button

### Toasts

Use Shadcn/ui `<Sonner>` (or `useToast`). Rules:
- Duration: 4 seconds for success, persistent for errors (until dismissed)
- Position: bottom-right
- Types: success (green), error (red), info (blue)

### Tables

Use Shadcn/ui `<Table>` components. Rules:
- Alternating row shading (`even:bg-slate-50`)
- Actions in the rightmost column as icon buttons
- Empty state: centered illustration + message when no data

### Badges / Status Pills

Document statuses displayed as colored badges:

| Status | Color |
|---|---|---|
| Filed | `bg-green-100 text-green-800` |
| Confidential | `bg-yellow-100 text-yellow-800` |
| Archived | `bg-slate-100 text-slate-700` |
| Other | `bg-blue-100 text-blue-800` |
| Hidden | `bg-purple-100 text-purple-800` (Secretary only) |
| Trashed | `bg-red-100 text-red-700` (TRASH view only) |

### Forms

Required fields show a red asterisk (`*`) after the label text. Invalid fields display an inline error message via the `FieldError` component directly below the input. Inputs use `aria-required`, `aria-invalid`, and `aria-describedby` for accessibility.

---

## 8. Icon Usage

Use `lucide-react` exclusively. Do not mix icon libraries. Key icons:

| Icon | Component | Usage |
|---|---|---|
| `<Folder>` | Category/Folder items | |
| `<FileText>` | Document items | |
| `<Search>` | Search bars | |
| `<Plus>` | Add/Create actions | |
| `<Pencil>` | Edit actions | |
| `<Trash2>` | Delete/Trash actions | |
| `<Eye>` | View action | |
| `<EyeOff>` | Hide/Hidden state | |
| `<MoveRight>` | Move document action | |
| `<Download>` | Export/Download action | |
| `<Upload>` | Upload action | |
| `<Scan>` | Scan Intake | |
| `<Lock>` | Restricted/Admin-only items | |
| `<RotateCcw>` | Restore from trash | |
| `<Shield>` | Security/Admin badge | |
| `<ChevronRight>` | Breadcrumb separator | |
| `<LayoutGrid>` | Icon view toggle | |
| `<List>` | List view toggle | |

---

## 9. Responsive Considerations

This is a **desktop application**. The Tauri window has a minimum size of **1024 × 768px**. Do not optimize for mobile breakpoints. Do support window resizing down to the minimum.

Grid columns collapse gracefully:
- 4 columns → 3 columns → 2 columns as window narrows
- Tables gain horizontal scroll when columns overflow
- Sidebar never collapses (no hamburger menu)

---

## 10. Dark Mode

Dark mode is **not in scope for MVP**. All color tokens are defined for light mode only. Use Tailwind's `dark:` variants only if dark mode support is added in a future release. Do not block MVP on dark mode implementation.

---

## 11. Accessibility Baseline

- All interactive elements must be keyboard-accessible (Tab, Enter, Space, Arrow keys)
- All icon-only buttons must have `aria-label` attributes
- All form inputs must be associated with visible `<label>` elements
- Color is never the sole means of conveying information (always pair color with text or icon)
- Focus ring must be visible on all focusable elements (Tailwind `focus-visible:ring-2`)

---

*End of Design System Overview*
*Next: `02_Database_Schema_Documentation.md`*


---

## 12. Confidential Warning Helper

When `Confidential` status is selected, show a warning/helper text near the status field: `Confidential documents are hidden from viewer access.` The hidden state should be visually indicated with the EyeOff icon wherever document cards or list rows are shown to Secretary/Admin users.

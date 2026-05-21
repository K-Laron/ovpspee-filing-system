# Final Mobile App Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a production-ready OVPSPEE desktop + Android document intake system for office-network use.

**Architecture:** Keep Tauri desktop as the source of truth and office PC hub. Keep Android as a Secretary-only capture client that submits files plus full Add Document metadata into a pending desktop review queue. Use small native Android modules only where React Native lacks reliable file/device APIs.

**Tech Stack:** Tauri 2, Rust, SQLite/sqlx, React 19, TypeScript, Vite, React Native 0.85, Kotlin Android native modules, Gradle 8.14.3, PowerShell release scripts.

---

## Implementation Status - 2026-05-21

- Phases 1-5 are implemented and committed on `codex/final-mobile-app-implementation`.
- Phase 6 is prepared with `docs/final-office-pilot-checklist.md`, but real phone install, camera/file picker proof, offline retry pilot, and backup/restore drill are blocked because `adb devices -l` returned no authorized Android device.
- Phase 7 automated release-scope checks passed for desktop build, desktop Vitest, mobile Jest/typecheck, signed release APK, and mobile/security/review Rust slices.
- Full `cargo test` after a clean Rust target rebuild did not finish within 30 minutes on 2026-05-21. Release-scope Rust tests passed serially with `CARGO_BUILD_JOBS=1`, `RUSTFLAGS=-C debuginfo=0`, and `CARGO_TARGET_DIR=D:\OVPSPEE_CDHP_cargo_target_final`.
- Final release tag `v1.0-final-mobile` is intentionally not created until the Phase 6 office pilot and backup/restore drill pass.

---

## Current State

Current `master` is v1.1 release candidate:

- Desktop review queue exists.
- Android app builds debug and signed release APKs.
- Persistent mobile draft/queue storage exists.
- Device token gate exists.
- HTTPS proxy tooling exists.
- Real Android phone install is blocked until an authorized ADB device is connected.
- Real camera/file picker is not implemented; capture buttons still stage placeholder file URIs.

## Final Definition

Final app means:

- Actual Android camera capture and file picker work on a real phone.
- Android upload survives disconnects, retries, and duplicate taps.
- Desktop can approve/reject with attachment preview and complete audit trail.
- Office PC setup can be done from docs/scripts by non-developer staff.
- Signed APK and desktop installer are archived with validation evidence.
- Pilot run proves no data loss, no duplicates, and backup/restore works.

## File Map

- `mobile/android/src/AppRoot.tsx`: top-level mobile flow, queue sync, session lock.
- `mobile/android/src/screens/*.tsx`: mobile UI screens.
- `mobile/android/src/api/client.ts`: mobile HTTP API client, retry, metadata payload.
- `mobile/android/src/storage/drafts.ts`: durable mobile draft/queue/device profile storage.
- `mobile/android/android/app/src/main/java/com/ovpspeemobile/*.kt`: Android native modules.
- `src-tauri/src/mobile_api.rs`: office PC HTTP API.
- `src-tauri/src/mobile_submissions.rs`: mobile submission DB/domain logic.
- `src-tauri/migrations/*.sql`: SQLite schema changes.
- `src/pages/secretary/MobileSubmissions.tsx`: desktop review queue.
- `src/components/AttachmentPreview.tsx`: existing preview patterns to reuse.
- `scripts/*.ps1`, `scripts/*.mjs`: release, HTTPS, install helpers.
- `docs/android-mobile-v1-setup.md`: operator setup guide.

---

## Phase 1: Real Android Capture

**Purpose:** Replace placeholder attachment URIs with real camera and file picker output.

**Files:**
- Create: `mobile/android/android/app/src/main/java/com/ovpspeemobile/OvpspeeCaptureModule.kt`
- Create: `mobile/android/android/app/src/main/java/com/ovpspeemobile/OvpspeeCapturePackage.kt`
- Modify: `mobile/android/android/app/src/main/java/com/ovpspeemobile/MainApplication.kt`
- Modify: `mobile/android/android/app/src/main/AndroidManifest.xml`
- Modify: `mobile/android/src/native/capture.ts`
- Modify: `mobile/android/src/AppRoot.tsx`
- Modify: `mobile/android/src/screens/AttachmentReviewScreen.tsx`
- Test: `mobile/android/src/__tests__/capture-module.test.ts`
- Test: `mobile/android/src/__tests__/mobile-flow.test.tsx`

### Task 1: Add Capture API Contract

- [ ] **Step 1: Create failing TypeScript test**

Create `mobile/android/src/__tests__/capture-module.test.ts`:

```ts
import { normalizePickedFile } from '../native/capture';

describe('capture native adapter', () => {
  it('normalizes native file payloads for upload drafts', () => {
    expect(
      normalizePickedFile({
        uri: 'content://office/memo.pdf',
        name: 'memo.pdf',
        mimeType: 'application/pdf',
        sizeBytes: 4096
      })
    ).toEqual({
      uri: 'content://office/memo.pdf',
      name: 'memo.pdf',
      type: 'application/pdf',
      sizeBytes: 4096
    });
  });
});
```

- [ ] **Step 2: Run red test**

Run:

```powershell
cd mobile/android
npm test -- --runInBand src/__tests__/capture-module.test.ts
```

Expected: FAIL because `../native/capture` does not exist.

- [ ] **Step 3: Implement adapter**

Create `mobile/android/src/native/capture.ts`:

```ts
import { NativeModules } from 'react-native';
import type { MobileAttachmentDraft } from '../types';

interface NativePickedFile {
  uri: string;
  name: string;
  mimeType: string;
  sizeBytes?: number;
}

interface CaptureModule {
  pickFile(): Promise<NativePickedFile>;
  capturePhoto(): Promise<NativePickedFile>;
}

const module = NativeModules.OvpspeeCapture as CaptureModule | undefined;

export const normalizePickedFile = (file: NativePickedFile): MobileAttachmentDraft => ({
  uri: file.uri,
  name: file.name,
  type: file.mimeType,
  sizeBytes: file.sizeBytes
});

export const pickFile = async (): Promise<MobileAttachmentDraft> => {
  if (!module) throw new Error('Android file picker is unavailable.');
  return normalizePickedFile(await module.pickFile());
};

export const capturePhoto = async (): Promise<MobileAttachmentDraft> => {
  if (!module) throw new Error('Android camera is unavailable.');
  return normalizePickedFile(await module.capturePhoto());
};
```

- [ ] **Step 4: Run green test**

Run:

```powershell
cd mobile/android
npm test -- --runInBand src/__tests__/capture-module.test.ts
```

Expected: PASS.

### Task 2: Add Android Native File Picker

- [ ] **Step 1: Write failing flow test**

Update `mobile/android/src/__tests__/mobile-flow.test.tsx` mock:

```ts
jest.mock('../native/capture', () => ({
  pickFile: jest.fn(async () => ({
    uri: 'content://office/mobile-picked.pdf',
    name: 'mobile-picked.pdf',
    type: 'application/pdf',
    sizeBytes: 4096
  })),
  capturePhoto: jest.fn(async () => ({
    uri: 'file:///cache/mobile-photo.jpg',
    name: 'mobile-photo.jpg',
    type: 'image/jpeg',
    sizeBytes: 2048
  }))
}));
```

Assert after pressing `Add file`:

```ts
expect(await screen.findByText('mobile-picked.pdf')).toBeTruthy();
```

- [ ] **Step 2: Run red test**

Run:

```powershell
cd mobile/android
npm test -- --runInBand src/__tests__/mobile-flow.test.tsx
```

Expected: FAIL until `AppRoot` uses `pickFile()`.

- [ ] **Step 3: Implement native Kotlin picker**

Create `mobile/android/android/app/src/main/java/com/ovpspeemobile/OvpspeeCaptureModule.kt` with Android `ACTION_OPEN_DOCUMENT`, `ActivityEventListener`, and promise resolution containing `uri`, `name`, `mimeType`, `sizeBytes`.

- [ ] **Step 4: Wire package**

Create `OvpspeeCapturePackage.kt` and add `OvpspeeCapturePackage()` to `MainApplication.kt`.

- [ ] **Step 5: Update AppRoot**

Replace placeholder `onAddFile` logic with:

```ts
onAddFile={() => {
  void pickFile()
    .then((file) => updateDraft((current) => ({ ...current, attachments: [...current.attachments, file] })))
    .catch(() => undefined);
}}
```

- [ ] **Step 6: Run tests**

Run:

```powershell
cd mobile/android
npm test -- --runInBand src/__tests__/mobile-flow.test.tsx
npm run typecheck
```

Expected: PASS.

### Task 3: Add Camera Capture

- [ ] **Step 1: Write failing camera test**

Extend `mobile-flow.test.tsx`:

```ts
fireEvent.press(screen.getByText('Camera capture'));
expect(await screen.findByText('mobile-photo.jpg')).toBeTruthy();
```

- [ ] **Step 2: Run red test**

Expected: FAIL until camera path uses native module.

- [ ] **Step 3: Implement Kotlin camera capture**

Use `MediaStore.ACTION_IMAGE_CAPTURE` with a cache file from `cacheDir`. Return a `file://` URI, `image/jpeg`, and actual file size. Add `android.permission.CAMERA` to manifest.

- [ ] **Step 4: Update AppRoot**

Replace placeholder `onCapture` with `capturePhoto()`.

- [ ] **Step 5: Build APK**

Run:

```powershell
cd mobile/android
$env:JAVA_HOME="C:\Program Files\Java\jdk-24"
$env:ANDROID_HOME="$env:LOCALAPPDATA\Android\Sdk"
$env:ANDROID_SDK_ROOT=$env:ANDROID_HOME
$env:JAVA_TOOL_OPTIONS="--enable-native-access=ALL-UNNAMED"
.\android\gradlew.bat -p android :app:assembleDebug
```

Expected: BUILD SUCCESSFUL.

- [ ] **Step 6: Commit**

```powershell
git add mobile/android
git commit -m "feat(mobile): add real Android capture"
```

---

## Phase 2: Reliable Mobile Upload

**Purpose:** Make upload safe under weak Wi-Fi and repeated user actions.

**Files:**
- Modify: `mobile/android/src/api/client.ts`
- Modify: `mobile/android/src/storage/drafts.ts`
- Modify: `mobile/android/src/screens/SubmissionHistoryScreen.tsx`
- Modify: `src-tauri/src/mobile_submissions.rs`
- Test: `mobile/android/src/__tests__/api-client.test.ts`
- Test: `mobile/android/src/__tests__/submission-queue.test.ts`
- Test: `src-tauri/tests/mobile_submissions_slice18.rs`

### Task 4: Add Queue Status States

- [ ] **Step 1: Write failing queue test**

Add to `submission-queue.test.ts`:

```ts
expect(await loadQueuedSubmissions()).toEqual([
  expect.objectContaining({ syncStatus: 'failed' })
]);
```

- [ ] **Step 2: Run red test**

Expected: FAIL because `syncStatus` is absent.

- [ ] **Step 3: Add type**

Add to `QueuedSubmission`:

```ts
syncStatus: 'pending' | 'retrying' | 'failed';
```

- [ ] **Step 4: Update queue mutators**

Set `pending` on save, `retrying` before upload, `failed` on failed attempt.

- [ ] **Step 5: Run tests**

```powershell
cd mobile/android
npm test -- --runInBand src/__tests__/submission-queue.test.ts
```

Expected: PASS.

### Task 5: Add Server Idempotency Assertion

- [ ] **Step 1: Extend Rust test**

In `mobile_submissions_slice18.rs`, assert only one row exists after duplicate submit:

```rust
let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mobile_submission WHERE client_submission_id = 'mobile-client-1'")
    .fetch_one(&fx.pool)
    .await
    .expect("count");
assert_eq!(count, 1);
```

- [ ] **Step 2: Run test**

```powershell
cd src-tauri
cargo test --test mobile_submissions_slice18 duplicate_client_submission_id_returns_existing_pending_submission
```

Expected: PASS. If fail, fix idempotency query before proceeding.

### Task 6: Commit Upload Reliability

- [ ] **Step 1: Run phase checks**

```powershell
cd mobile/android
npm test -- --runInBand
npm run typecheck
cd ..\..\src-tauri
cargo test --test mobile_submissions_slice18
```

- [ ] **Step 2: Commit**

```powershell
git add mobile/android src-tauri
git commit -m "fix(mobile): harden retry queue states"
```

---

## Phase 3: Security Finalization

**Purpose:** Move from shared token to approved device management.

**Files:**
- Create: `src-tauri/migrations/0007_mobile_devices.sql`
- Create: `src-tauri/src/mobile_devices.rs`
- Modify: `src-tauri/src/mobile_api.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/admin/MobileDevices.tsx`
- Modify: `src/App.tsx`
- Modify: `src/components/layout/AdminLayout.tsx`
- Test: `src-tauri/tests/mobile_devices_slice19.rs`
- Test: `src/pages/admin/MobileDevices.test.tsx`

### Task 7: Add Mobile Device Table

- [ ] **Step 1: Create migration**

`0007_mobile_devices.sql`:

```sql
CREATE TABLE IF NOT EXISTS mobile_device (
    mobile_device_id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL UNIQUE,
    device_name TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    last_seen_at TEXT,
    created_by INTEGER REFERENCES user(user_id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_mobile_device_active ON mobile_device(is_active);
```

- [ ] **Step 2: Write Rust table test**

Create `mobile_devices_slice19.rs` asserting table exists after `create_test_pool()`.

- [ ] **Step 3: Run red/green**

Run:

```powershell
cd src-tauri
cargo test --test mobile_devices_slice19
```

Expected: PASS after migration.

### Task 8: Add Device Registration Commands

- [ ] **Step 1: Write failing Rust tests**

Test:

- Admin can create device token.
- Secretary cannot create device token.
- Revoked token cannot use mobile API.

- [ ] **Step 2: Implement `mobile_devices.rs`**

Functions:

```rust
pub async fn create_mobile_device(pool: &DbPool, session_id: &str, device_name: &str) -> AppResult<CreatedMobileDevice>
pub async fn list_mobile_devices(pool: &DbPool, session_id: &str) -> AppResult<Vec<MobileDeviceItem>>
pub async fn revoke_mobile_device(pool: &DbPool, session_id: &str, device_id: &str) -> AppResult<()>
pub async fn validate_mobile_device(pool: &DbPool, device_id: &str, token: &str) -> AppResult<()>
```

- [ ] **Step 3: Wire mobile API**

Require headers:

```text
X-OVPSPEE-Device-Id
X-OVPSPEE-Device-Token
```

- [ ] **Step 4: Run tests**

```powershell
cd src-tauri
cargo test --test mobile_devices_slice19 --test mobile_api_slice18
```

Expected: PASS.

### Task 9: Add Admin Device UI

- [ ] **Step 1: Write failing UI test**

Test shows:

- Device list.
- Create token button.
- Revoke button.

- [ ] **Step 2: Implement page**

Create `src/pages/admin/MobileDevices.tsx`.

- [ ] **Step 3: Wire nav/route/invoke/types**

Add route `/a/mobile-devices`.

- [ ] **Step 4: Run UI tests**

```powershell
pnpm exec vitest run src/pages/admin/MobileDevices.test.tsx --pool=threads
pnpm build
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add src src-tauri
git commit -m "feat(security): add mobile device registry"
```

---

## Phase 4: Desktop Review Final UX

**Purpose:** Make desktop review fast and complete.

**Files:**
- Modify: `src/pages/secretary/MobileSubmissions.tsx`
- Modify: `src-tauri/src/mobile_submissions.rs`
- Add: mobile attachment preview command or reuse existing attachment preview pipeline.
- Test: `src/pages/secretary/MobileSubmissions.test.tsx`
- Test: `src-tauri/tests/mobile_submissions_slice18.rs`

### Task 10: Add Mobile Attachment Preview

- [ ] **Step 1: Write failing UI test**

Assert selected submission attachment has Preview button:

```ts
expect(screen.getByRole('button', { name: /preview/i })).toBeInTheDocument();
```

- [ ] **Step 2: Add backend preview command**

Add:

```rust
pub async fn get_mobile_submission_attachment_preview_page(...)
```

Use same preview logic as existing `documents`/`scan_intake` preview.

- [ ] **Step 3: Add UI preview panel**

Display PDF/image/text/unsupported states.

- [ ] **Step 4: Run tests**

```powershell
pnpm exec vitest run src/pages/secretary/MobileSubmissions.test.tsx --pool=threads
cd src-tauri
cargo test --test mobile_submissions_slice18
```

- [ ] **Step 5: Commit**

```powershell
git add src src-tauri
git commit -m "feat(secretary): preview mobile attachments"
```

### Task 11: Add Keyboard Review Flow

- [ ] **Step 1: Write failing UI test**

Assert `Approve` and `Reject` buttons support keyboard focus and no overlap at 900px width.

- [ ] **Step 2: Implement keyboard shortcuts**

Use:

- `Ctrl+Enter`: approve.
- `Ctrl+Backspace`: open reject dialog.
- `Escape`: close dialog.

- [ ] **Step 3: Verify in browser**

Run app and inspect `/s/mobile-submissions` at:

- 390x844
- 900x640
- 1440x900

- [ ] **Step 4: Commit**

```powershell
git add src/pages/secretary/MobileSubmissions.tsx
git commit -m "feat(secretary): speed mobile review flow"
```

---

## Phase 5: Release Packaging

**Purpose:** Produce installable desktop + Android release bundle.

**Files:**
- Modify: `mobile/android/android/app/build.gradle`
- Modify: `scripts/install-android-apk.ps1`
- Create: `scripts/build-final-release.ps1`
- Modify: `docs/android-mobile-v1-setup.md`
- Create: `release-handoff-final/`

### Task 12: Add One-Command Final Build

- [ ] **Step 1: Create build script**

Create `scripts/build-final-release.ps1`:

```powershell
$ErrorActionPreference = "Stop"
pnpm build
Push-Location mobile/android
$env:JAVA_HOME = "C:\Program Files\Java\jdk-24"
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
$env:ANDROID_SDK_ROOT = $env:ANDROID_HOME
$env:JAVA_TOOL_OPTIONS = "--enable-native-access=ALL-UNNAMED"
.\android\gradlew.bat -p android :app:assembleRelease
Pop-Location
Push-Location src-tauri
cargo test --test mobile_submissions_slice18 --test mobile_api_slice18
Pop-Location
```

- [ ] **Step 2: Run script**

```powershell
.\scripts\build-final-release.ps1
```

Expected: all checks pass and release APK exists.

- [ ] **Step 3: Commit**

```powershell
git add scripts docs
git commit -m "chore(release): add final build script"
```

### Task 13: Archive Handoff

- [ ] **Step 1: Create release folder**

```powershell
New-Item -ItemType Directory -Force release-handoff-final
Copy-Item mobile\android\android\app\build\outputs\apk\release\app-release.apk release-handoff-final\
Copy-Item docs\android-mobile-v1-setup.md release-handoff-final\
```

- [ ] **Step 2: Add validation report**

Create `release-handoff-final/VALIDATION_REPORT.md` with:

- Exact commit hash.
- Commands run.
- APK signature digest.
- Known limitation list.
- Pilot checklist.

- [ ] **Step 3: Commit report only if release folder should be versioned**

If release artifacts are too large, commit report only and keep APK path documented.

---

## Phase 6: Office Pilot

**Purpose:** Prove final app works in real office conditions.

**Files:**
- Create: `docs/final-office-pilot-checklist.md`
- Create: `manual-final-mobile-pilot/` evidence screenshots/photos if needed.

### Task 14: Real Phone Install

- [ ] **Step 1: Connect Android phone**

Enable Developer Options and USB debugging. Accept RSA prompt.

- [ ] **Step 2: Verify ADB**

```powershell
$adb="$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe"
& $adb devices -l
```

Expected: one line ending with `device`.

- [ ] **Step 3: Install and launch**

```powershell
.\scripts\install-android-apk.ps1 -BuildType release
```

Expected: install succeeds and app launches.

### Task 15: Pilot Submission Matrix

- [ ] **Step 1: Submit samples**

Submit:

- 3 camera photos.
- 3 PDFs.
- 2 image files.
- 1 invalid file type.
- 1 offline queued upload.

- [ ] **Step 2: Review desktop**

Approve 7 valid items. Reject invalid/incorrect items with templates.

- [ ] **Step 3: Verify DB**

No duplicate `client_submission_id`.

```powershell
cd src-tauri
cargo test --test mobile_submissions_slice18
```

### Task 16: Backup Restore Drill

- [ ] **Step 1: Create backup after pilot data**

Use desktop Backup page.

- [ ] **Step 2: Restore backup**

Restore into test PC/profile.

- [ ] **Step 3: Verify restored data**

Check mobile submissions, approved documents, attachments, audit logs.

---

## Phase 7: Final Production Cut

**Purpose:** Freeze, tag, and hand off.

### Task 17: Final Regression

- [ ] **Step 1: Run full automated checks**

```powershell
pnpm build
pnpm exec vitest run --pool=threads
cd mobile/android
npm test -- --runInBand
npm run typecheck
.\android\gradlew.bat -p android :app:assembleRelease
cd ..\..\src-tauri
cargo fmt --check
cargo test
```

- [ ] **Step 2: Fix only must-fix bugs**

No new features after this point.

### Task 18: Tag Final Release

- [ ] **Step 1: Confirm clean tree**

```powershell
git status --short --branch
```

Expected: clean.

- [ ] **Step 2: Tag**

```powershell
git tag -a v1.0-final-mobile -m "OVPSPEE final mobile release"
```

- [ ] **Step 3: Export final artifacts**

Save:

- Desktop installer.
- Signed Android APK.
- Setup guide.
- Validation report.
- Pilot checklist.
- Keystore custody note.

---

## Final Validation Checklist

- [ ] Real Android phone install passed.
- [ ] Camera capture upload passed.
- [ ] File picker upload passed.
- [ ] Offline retry passed.
- [ ] Duplicate submit prevented.
- [ ] Unknown/revoked device blocked.
- [ ] HTTPS phone connection passed.
- [ ] Desktop approve/reject passed.
- [ ] Attachment preview passed.
- [ ] Audit trail complete.
- [ ] Backup/restore passed.
- [ ] Signed APK verified.
- [ ] Desktop installer verified.
- [ ] Release docs complete.

## Execution Order

1. Phase 1.
2. Phase 2.
3. Phase 3.
4. Phase 4.
5. Phase 5.
6. Phase 6.
7. Phase 7.

Do not start Phase 6 until Phases 1-5 pass on `master`.

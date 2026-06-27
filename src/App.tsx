import { lazy, Suspense, useEffect, useState } from 'react';
import { Navigate, Route, Routes, useNavigate } from 'react-router-dom';

import { AdminLayout } from './components/layout/AdminLayout';
import { GuestLayout } from './components/layout/GuestLayout';
import { ProtectedRoute } from './components/layout/ProtectedRoute';
import { SecretaryLayout } from './components/layout/SecretaryLayout';
import { invoke } from '@tauri-apps/api/core';
import type { SessionPayload } from './types';
const AddDocument = lazy(() =>
  import('./pages/secretary/AddDocument').then((m) => ({ default: m.AddDocument })),
);
const AuditLog = lazy(() =>
  import('./pages/admin/AuditLog').then((m) => ({ default: m.AuditLog })),
);
const BackupRestore = lazy(() =>
  import('./pages/admin/BackupRestore').then((m) => ({ default: m.BackupRestore })),
);
const DeviceSettingsPage = lazy(() =>
  import('./pages/admin/DeviceSettings').then((m) => ({ default: m.DeviceSettingsPage })),
);
const Documents = lazy(() =>
  import('./pages/secretary/Documents').then((m) => ({ default: m.Documents })),
);
const FirstRunSetup = lazy(() =>
  import('./pages/FirstRunSetup').then((m) => ({ default: m.FirstRunSetup })),
);
const GuestLanding = lazy(() =>
  import('./pages/GuestLanding').then((m) => ({ default: m.GuestLanding })),
);
const Login = lazy(() => import('./pages/Login').then((m) => ({ default: m.Login })));
const MasterData = lazy(() =>
  import('./pages/admin/MasterData').then((m) => ({ default: m.MasterData })),
);
const MobileDevices = lazy(() =>
  import('./pages/admin/MobileDevices').then((m) => ({ default: m.MobileDevices })),
);
const MobileSubmissions = lazy(() =>
  import('./pages/secretary/MobileSubmissions').then((m) => ({ default: m.MobileSubmissions })),
);
const MyActivity = lazy(() =>
  import('./pages/secretary/MyActivity').then((m) => ({ default: m.MyActivity })),
);
const Profile = lazy(() => import('./pages/Profile').then((m) => ({ default: m.Profile })));
const ScanIntake = lazy(() =>
  import('./pages/secretary/ScanIntake').then((m) => ({ default: m.ScanIntake })),
);
const TrashManagement = lazy(() =>
  import('./pages/admin/TrashManagement').then((m) => ({ default: m.TrashManagement })),
);
const Users = lazy(() => import('./pages/admin/Users').then((m) => ({ default: m.Users })));
import { useSessionStore } from './store/sessionStore';

export const App = () => {
  const navigate = useNavigate();
  const { sessionId, setSession, clearSession } = useSessionStore();
  const [checking, setChecking] = useState(true);

  useEffect(() => {
    let mounted = true;

    const bootstrap = async () => {
      try {
        const needsSetup = await invoke<boolean>('first_run_check');
        if (!mounted) return;
        if (needsSetup) {
          navigate('/first-run', { replace: true });
          return;
        }

        if (sessionId) {
          const session = await invoke<SessionPayload>('validate_session', { sessionId });
          if (mounted) setSession(session);
        }
      } catch {
        clearSession();
      } finally {
        if (mounted) setChecking(false);
      }
    };

    void bootstrap();
    return () => {
      mounted = false;
    };
  }, [clearSession, navigate, sessionId, setSession]);

  if (checking) {
    return (
      <div className="flex h-screen items-center justify-center bg-background text-secondary">
        <div className="rounded border border-border bg-surface px-5 py-4 shadow-sm">
          Checking system setup...
        </div>
      </div>
    );
  }

  return (
    <Suspense
      fallback={
        <div className="flex h-screen items-center justify-center bg-background text-secondary">
          <div className="rounded border border-border bg-surface px-5 py-4 shadow-sm">
            Loading...
          </div>
        </div>
      }
    >
      <Routes>
        <Route element={<GuestLayout />}>
          <Route path="/" element={<GuestLanding />} />
        </Route>
        <Route path="/login" element={<Login />} />
        <Route path="/first-run" element={<FirstRunSetup />} />
        <Route
          path="/a"
          element={
            <ProtectedRoute role="Admin">
              <AdminLayout />
            </ProtectedRoute>
          }
        >
          <Route index element={<Navigate to="users" replace />} />
          <Route path="users" element={<Users />} />
          <Route path="master-data" element={<MasterData />} />
          <Route path="trash" element={<TrashManagement />} />
          <Route path="audit-log" element={<AuditLog />} />
          <Route path="backup-restore" element={<BackupRestore />} />
          <Route path="devices" element={<DeviceSettingsPage />} />
          <Route path="mobile-devices" element={<MobileDevices />} />
          <Route path="profile" element={<Profile />} />
        </Route>
        <Route
          path="/s"
          element={
            <ProtectedRoute role="Secretary">
              <SecretaryLayout />
            </ProtectedRoute>
          }
        >
          <Route index element={<Navigate to="documents" replace />} />
          <Route path="profile" element={<Profile />} />
          <Route path="documents" element={<Documents />} />
          <Route path="documents/new" element={<AddDocument />} />
          <Route path="scan-intake" element={<ScanIntake />} />
          <Route path="mobile-submissions" element={<MobileSubmissions />} />
          <Route path="devices" element={<DeviceSettingsPage />} />
          <Route path="my-activity" element={<MyActivity />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </Suspense>
  );
};

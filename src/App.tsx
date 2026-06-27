import { useEffect, useState } from 'react';
import { Navigate, Route, Routes, useNavigate } from 'react-router-dom';

import { AdminLayout } from './components/layout/AdminLayout';
import { GuestLayout } from './components/layout/GuestLayout';
import { ProtectedRoute } from './components/layout/ProtectedRoute';
import { SecretaryLayout } from './components/layout/SecretaryLayout';
import { invoke } from '@tauri-apps/api/core';
import type { SessionPayload } from './types';
import { AuditLog } from './pages/admin/AuditLog';
import { BackupRestore } from './pages/admin/BackupRestore';
import { DeviceSettingsPage } from './pages/admin/DeviceSettings';
import { MasterData } from './pages/admin/MasterData';
import { MobileDevices } from './pages/admin/MobileDevices';
import { TrashManagement } from './pages/admin/TrashManagement';
import { Users } from './pages/admin/Users';
import { FirstRunSetup } from './pages/FirstRunSetup';
import { GuestLanding } from './pages/GuestLanding';
import { Login } from './pages/Login';
import { Profile } from './pages/Profile';
import { AddDocument } from './pages/secretary/AddDocument';
import { Documents } from './pages/secretary/Documents';
import { MobileSubmissions } from './pages/secretary/MobileSubmissions';
import { MyActivity } from './pages/secretary/MyActivity';
import { ScanIntake } from './pages/secretary/ScanIntake';
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
  );
};

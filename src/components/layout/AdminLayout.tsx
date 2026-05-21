import { ClipboardList, Database, DatabaseBackup, Printer, Smartphone, Trash2, UserCircle, Users } from 'lucide-react';

import { AppShell } from './AppShell';

export const AdminLayout = () => (
  <AppShell
    title="Admin Console"
    subtitle="IT Staff"
    navItems={[
      { label: 'Users', path: '/a/users', icon: Users },
      { label: 'Master Data', path: '/a/master-data', icon: Database },
      { label: 'Devices', path: '/a/devices', icon: Printer },
      { label: 'Mobile Devices', path: '/a/mobile-devices', icon: Smartphone },
      { label: 'Backup & Restore', path: '/a/backup-restore', icon: DatabaseBackup },
      { label: 'Audit Log', path: '/a/audit-log', icon: ClipboardList },
      { label: 'Trash', path: '/a/trash', icon: Trash2 }
    ]}
    profileItem={{ label: 'Profile', path: '/a/profile', icon: UserCircle }}
  />
);

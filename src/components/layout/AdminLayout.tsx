import { ClipboardList, Database, Trash2, UserCircle, Users } from 'lucide-react';

import { AppShell } from './AppShell';

export const AdminLayout = () => (
  <AppShell
    title="Admin Console"
    subtitle="IT Staff"
    navItems={[
      { label: 'Users', path: '/a/users', icon: Users },
      { label: 'Master Data', path: '/a/master-data', icon: Database },
      { label: 'Trash', path: '/a/trash', icon: Trash2 },
      { label: 'Profile', path: '/a/profile', icon: UserCircle },
      { label: 'Audit Log', path: '/a', icon: ClipboardList }
    ]}
  />
);

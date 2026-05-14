import { FilePlus, Files, LayoutDashboard, UserCircle } from 'lucide-react';

import { AppShell } from './AppShell';

export const SecretaryLayout = () => (
  <AppShell
    title="Secretary Workspace"
    subtitle="Filing operations"
    navItems={[
      { label: 'Dashboard', path: '/s', icon: LayoutDashboard },
      { label: 'Profile', path: '/s/profile', icon: UserCircle },
      { label: 'Documents', path: '/s', icon: Files },
      { label: 'Add Document', path: '/s', icon: FilePlus }
    ]}
  />
);

import { Activity, FilePlus, Files, FileScan, LayoutDashboard, UserCircle } from 'lucide-react';

import { AppShell } from './AppShell';

export const SecretaryLayout = () => (
  <AppShell
    title="Secretary Workspace"
    subtitle="Filing operations"
    navItems={[
      { label: 'Dashboard', path: '/s', icon: LayoutDashboard },
      { label: 'Profile', path: '/s/profile', icon: UserCircle },
      { label: 'Documents', path: '/s/documents', icon: Files },
      { label: 'Scan Intake', path: '/s/scan-intake', icon: FileScan },
      { label: 'My Activity', path: '/s/my-activity', icon: Activity },
      { label: 'Add Document', path: '/s/documents/new', icon: FilePlus }
    ]}
  />
);

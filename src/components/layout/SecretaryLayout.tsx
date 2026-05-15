import { Activity, FilePlus, Files, FileScan, LayoutDashboard, Printer, UserCircle } from 'lucide-react';

import { AppShell } from './AppShell';

export const SecretaryLayout = () => (
  <AppShell
    title="Secretary Workspace"
    subtitle="Filing operations"
    navItems={[
      { label: 'Dashboard', path: '/s', icon: LayoutDashboard },
      { label: 'Documents', path: '/s/documents', icon: Files },
      { label: 'Devices', path: '/s/devices', icon: Printer },
      {
        label: 'Scan Intake',
        path: '/s/scan-intake',
        icon: FileScan,
        description: 'Use Scan Intake for scanned/imported files that still need to be reviewed and filed.'
      },
      {
        label: 'Add Document',
        path: '/s/documents/new',
        icon: FilePlus,
        description: 'Use Add Document to create an official document record with metadata and attachments.'
      },
      { label: 'Activity', path: '/s/my-activity', icon: Activity }
    ]}
    profileItem={{ label: 'Profile', path: '/s/profile', icon: UserCircle }}
  />
);

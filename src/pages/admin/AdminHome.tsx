import { Shield } from 'lucide-react';

export const AdminHome = () => (
  <section>
    <h1 className="text-2xl font-bold text-secondary">Admin Dashboard</h1>
    <p className="mt-1 text-sm text-muted">Manage users, filing lists, devices, backups, audit logs, and trash controls.</p>
    <div className="mt-6 rounded border border-border bg-surface p-5 shadow-sm">
      <Shield className="mb-3 text-primary" size={28} />
      <h2 className="font-semibold text-secondary">System management</h2>
      <p className="mt-1 text-sm text-muted">
        Use the left menu to manage accounts, categories, folders, offices, backups, devices, and audit records.
      </p>
    </div>
  </section>
);

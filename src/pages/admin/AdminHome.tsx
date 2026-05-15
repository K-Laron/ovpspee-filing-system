import { Shield } from 'lucide-react';

export const AdminHome = () => (
  <section>
    <h1 className="text-2xl font-bold text-secondary">Admin Shell</h1>
    <p className="mt-1 text-sm text-muted">Admin setup tools are available for IT Staff.</p>
    <div className="mt-6 rounded border border-border bg-surface p-5 shadow-sm">
      <Shield className="mb-3 text-primary" size={28} />
      <h2 className="font-semibold text-secondary">Later admin features</h2>
      <p className="mt-1 text-sm text-muted">
        Audit log browsing is available. Backup tools and deployment packaging remain deferred to later slices.
      </p>
    </div>
  </section>
);

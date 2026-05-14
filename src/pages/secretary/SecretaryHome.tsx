import { Files } from 'lucide-react';

export const SecretaryHome = () => (
  <section>
    <h1 className="text-2xl font-bold text-secondary">Secretary Shell</h1>
    <p className="mt-1 text-sm text-muted">Authenticated workspace layout is available.</p>
    <div className="mt-6 rounded border border-border bg-surface p-5 shadow-sm">
      <Files className="mb-3 text-primary" size={28} />
      <h2 className="font-semibold text-secondary">Document filing deferred</h2>
      <p className="mt-1 text-sm text-muted">
        Document filing, scan intake, PDF export, and trash workflows start in later slices.
      </p>
    </div>
  </section>
);

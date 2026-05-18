import { Files } from 'lucide-react';

export const SecretaryHome = () => (
  <section>
    <h1 className="text-2xl font-bold text-secondary">Secretary Dashboard</h1>
    <p className="mt-1 text-sm text-muted">File documents, review scans, manage attachments, and track your activity.</p>
    <div className="mt-6 rounded border border-border bg-surface p-5 shadow-sm">
      <Files className="mb-3 text-primary" size={28} />
      <h2 className="font-semibold text-secondary">Document filing</h2>
      <p className="mt-1 text-sm text-muted">
        Start with Add Document for new records or Scan Intake for scanned files that need review.
      </p>
    </div>
  </section>
);

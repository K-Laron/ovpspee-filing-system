import { Folder, Search } from 'lucide-react';

export const GuestLanding = () => (
  <section className="space-y-8">
    <div className="rounded border border-border bg-surface p-5 shadow-sm">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-secondary">Recent Documents</h1>
          <p className="mt-1 text-sm text-muted">
            Document browsing starts in Slice 4. Auth shell is active now.
          </p>
        </div>
        <div className="flex min-w-80 items-center gap-2 rounded border border-border bg-background px-3 py-2 text-muted">
          <Search size={16} />
          <span className="text-sm">Search disabled until document filing slice</span>
        </div>
      </div>
      <div className="mt-6 rounded border border-dashed border-border bg-background p-6 text-sm text-muted">
        No documents are filed yet.
      </div>
    </div>

    <div>
      <h2 className="mb-3 text-xl font-semibold text-secondary">Browse by Category</h2>
      <div className="grid grid-cols-4 gap-4">
        <div className="rounded border border-dashed border-border bg-surface p-4 text-muted">
          <Folder className="mb-3 text-primary" size={28} />
          <p className="text-sm font-semibold text-secondary">Document browsing deferred</p>
          <p className="mt-1 text-xs">Category browsing starts with document filing.</p>
        </div>
      </div>
    </div>
  </section>
);

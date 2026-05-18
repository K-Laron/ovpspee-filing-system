import { RefreshCw, Save, Search, ShieldCheck } from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';

import {
  getAuditRetentionSettings,
  listAuditEventTypes,
  listAuditLogs,
  updateAuditRetentionSettings
} from '../../lib/invoke';
import { formatDateTime } from '../../lib/dates';
import { getUserErrorMessage } from '../../lib/errors';
import { useSessionStore } from '../../store/sessionStore';
import type { AuditLogEntry, AuditRetentionSettings } from '../../types';

const pageSizeOptions = [25, 50, 100, 200];

export const AuditLog = () => {
  const sessionId = useSessionStore((state) => state.sessionId);
  const [entries, setEntries] = useState<AuditLogEntry[]>([]);
  const [eventTypes, setEventTypes] = useState<string[]>([]);
  const [retention, setRetention] = useState<AuditRetentionSettings | null>(null);
  const [retentionDraft, setRetentionDraft] = useState('36');
  const [search, setSearch] = useState('');
  const [actorSearch, setActorSearch] = useState('');
  const [action, setAction] = useState('');
  const [entityType, setEntityType] = useState('');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [limit, setLimit] = useState(50);
  const [offset, setOffset] = useState(0);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState('');

  const load = async (nextOffset = offset) => {
    if (!sessionId) return;
    setLoading(true);
    setMessage('');
    try {
      const page = await listAuditLogs({
        sessionId,
        search: nullable(search),
        actorSearch: nullable(actorSearch),
        action: nullable(action),
        entityType: nullable(entityType),
        dateFrom: nullable(dateFrom),
        dateTo: nullable(dateTo),
        limit,
        offset: nextOffset
      });
      setEntries(page.entries);
      setOffset(page.offset);
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not load audit logs.'));
    } finally {
      setLoading(false);
    }
  };

  const loadLookups = async () => {
    if (!sessionId) return;
    const [types, settings] = await Promise.all([
      listAuditEventTypes(sessionId),
      getAuditRetentionSettings(sessionId)
    ]);
    setEventTypes(types);
    setRetention(settings);
    setRetentionDraft(String(settings.retention_months));
  };

  useEffect(() => {
    void loadLookups().catch((err) => setMessage(getUserErrorMessage(err, 'Could not load audit settings.')));
    void load(0);
  }, [sessionId]);

  const submitFilters = (event: FormEvent) => {
    event.preventDefault();
    void load(0);
  };

  const saveRetention = async () => {
    if (!sessionId) return;
    try {
      const next = await updateAuditRetentionSettings({
        sessionId,
        retentionMonths: Number(retentionDraft)
      });
      setRetention(next);
      setRetentionDraft(String(next.retention_months));
      setMessage('Audit retention updated.');
      await loadLookups();
      await load(0);
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not update audit retention.'));
    }
  };

  return (
    <section className="mx-auto max-w-7xl space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-bold text-secondary">Audit Log</h1>
          <p className="mt-1 text-sm text-muted">All system activity. Read-only.</p>
        </div>
        <button className="btn" disabled={loading} onClick={() => void load()} type="button">
          <RefreshCw size={16} />
          Refresh
        </button>
      </div>

      <form className="grid gap-3 rounded border border-border bg-surface p-4 shadow-sm lg:grid-cols-[1fr_180px_180px_140px_140px_120px_auto]" onSubmit={submitFilters}>
        <label>
          <span className="form-label">Search</span>
          <input className="input" onChange={(event) => setSearch(event.target.value)} value={search} />
        </label>
        <label>
          <span className="form-label">Actor</span>
          <input className="input" onChange={(event) => setActorSearch(event.target.value)} value={actorSearch} />
        </label>
        <label>
          <span className="form-label">Action</span>
          <select className="input" onChange={(event) => setAction(event.target.value)} value={action}>
            <option value="">All</option>
            {eventTypes.map((type) => <option key={type} value={type}>{type}</option>)}
          </select>
        </label>
        <label>
          <span className="form-label">Entity</span>
          <input className="input" onChange={(event) => setEntityType(event.target.value)} value={entityType} />
        </label>
        <label>
          <span className="form-label">From</span>
          <input className="input" onChange={(event) => setDateFrom(event.target.value)} type="date" value={dateFrom} />
        </label>
        <label>
          <span className="form-label">Limit</span>
          <select className="input" onChange={(event) => setLimit(Number(event.target.value))} value={limit}>
            {pageSizeOptions.map((size) => <option key={size} value={size}>{size}</option>)}
          </select>
        </label>
        <button className="btn btn-primary self-end" type="submit"><Search size={16} />Apply</button>
        <label className="lg:col-start-5">
          <span className="form-label">To</span>
          <input className="input" onChange={(event) => setDateTo(event.target.value)} type="date" value={dateTo} />
        </label>
      </form>

      <section className="flex flex-wrap items-end gap-3 rounded border border-border bg-surface p-4 shadow-sm">
        <ShieldCheck className="mb-2 text-primary" size={20} />
        <label>
          <span className="form-label">Retention months</span>
          <input className="input w-32" max={retention?.max_months ?? 36} min={retention?.min_months ?? 24} onChange={(event) => setRetentionDraft(event.target.value)} type="number" value={retentionDraft} />
        </label>
        <button className="btn btn-primary" onClick={() => void saveRetention()} type="button"><Save size={16} />Save</button>
        <p className="pb-2 text-sm text-muted">
          Records are kept for {retentionDraft || retention?.retention_months || 36} months. Allowed range: {retention?.min_months ?? 24}-{retention?.max_months ?? 36} months.
        </p>
      </section>

      {message && <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div>}

      <AuditTable entries={entries} loading={loading} />
      <Pager count={entries.length} limit={limit} offset={offset} onPage={(next) => void load(next)} />
    </section>
  );
};

const AuditTable = ({ entries, loading }: { entries: AuditLogEntry[]; loading: boolean }) => (
  <div className="overflow-hidden rounded border border-border bg-surface shadow-sm">
    <table className="w-full table-fixed text-left text-sm">
      <thead className="border-b border-border bg-background text-xs uppercase text-muted">
        <tr>
          <th className="w-36 p-3">When</th>
          <th className="w-28 p-3">Action</th>
          <th className="w-44 p-3">Actor</th>
          <th className="w-36 p-3">Target</th>
          <th className="p-3">Details</th>
        </tr>
      </thead>
      <tbody className="divide-y divide-border">
        {loading && <tr><td className="p-4 text-center text-muted" colSpan={5}>Loading...</td></tr>}
        {!loading && entries.length === 0 && <tr><td className="p-4 text-center text-muted" colSpan={5}>No audit records.</td></tr>}
        {!loading && entries.map((entry) => (
          <tr key={entry.id}>
            <td className="p-3 text-xs text-muted">{formatDateTime(entry.created_at)}</td>
            <td className="p-3"><span className="rounded bg-background px-2 py-1 text-xs font-semibold text-secondary">{entry.action}</span></td>
            <td className="p-3">
              <p className="truncate font-medium text-secondary">{entry.actor_display_name ?? 'System'}</p>
              <p className="truncate text-xs text-muted">{entry.actor_username ?? 'No user'}{entry.actor_role ? ` · ${entry.actor_role}` : ''}</p>
            </td>
            <td className="p-3 text-xs text-muted">{entry.entity_type ?? '-'}{entry.entity_id ? ` #${entry.entity_id}` : ''}</td>
            <td className="p-3 text-secondary">{entry.summary}</td>
          </tr>
        ))}
      </tbody>
    </table>
  </div>
);

const Pager = ({ count, limit, offset, onPage }: { count: number; limit: number; offset: number; onPage: (offset: number) => void }) => (
  <div className="flex items-center justify-end gap-2">
    <button className="btn" disabled={offset === 0} onClick={() => onPage(Math.max(0, offset - limit))} type="button">Previous</button>
    <span className="text-sm text-muted">Offset {offset}</span>
    <button className="btn" disabled={count < limit} onClick={() => onPage(offset + limit)} type="button">Next</button>
  </div>
);

const nullable = (value: string) => {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
};

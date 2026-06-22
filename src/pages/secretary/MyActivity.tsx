import { RefreshCw, Search } from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';

import { cmd } from '../../lib/invoke';
import { formatDateTime } from '../../lib/dates';
import { getUserErrorMessage } from '../../lib/errors';
import { useSessionStore } from '../../store/sessionStore';
import type { AuditLogEntry, AuditLogPage } from '../../types';

const pageSizeOptions = [25, 50, 100];

export const MyActivity = () => {
  const sessionId = useSessionStore((state) => state.sessionId);
  const [entries, setEntries] = useState<AuditLogEntry[]>([]);
  const [eventTypes, setEventTypes] = useState<string[]>([]);
  const [search, setSearch] = useState('');
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
      const page = await cmd<AuditLogPage>('list_my_activity', {
        sessionId,
        search: nullable(search),
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
      setMessage(getUserErrorMessage(err, 'Could not load activity.'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (!sessionId) return;
    void cmd<string[]>('list_my_activity_event_types', { sessionId }).then(setEventTypes).catch((err) => setMessage(getUserErrorMessage(err, 'Could not load activity filters.')));
    void load(0);
  }, [sessionId]);

  const submitFilters = (event: FormEvent) => {
    event.preventDefault();
    void load(0);
  };

  return (
    <section className="mx-auto max-w-6xl space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-bold text-secondary">My Activity</h1>
          <p className="mt-1 text-sm text-muted">Your activity history. Read-only.</p>
        </div>
        <button className="btn" disabled={loading} onClick={() => void load()} type="button">
          <RefreshCw size={16} />
          Refresh
        </button>
      </div>

      <form className="grid gap-3 rounded border border-border bg-surface p-4 shadow-sm md:grid-cols-[1fr_160px_140px_140px_120px_auto]" onSubmit={submitFilters}>
        <label>
          <span className="form-label">Search</span>
          <input className="input" onChange={(event) => setSearch(event.target.value)} value={search} />
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
        <label className="md:col-start-4">
          <span className="form-label">To</span>
          <input className="input" onChange={(event) => setDateTo(event.target.value)} type="date" value={dateTo} />
        </label>
      </form>

      {message && <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div>}

      <div className="overflow-hidden rounded border border-border bg-surface shadow-sm">
        <table className="w-full table-fixed text-left text-sm">
          <thead className="border-b border-border bg-background text-xs uppercase text-muted">
            <tr>
              <th className="w-36 p-3">When</th>
              <th className="w-28 p-3">Action</th>
              <th className="w-36 p-3">Target</th>
              <th className="p-3">Details</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {loading && <tr><td className="p-4 text-center text-muted" colSpan={4}>Loading...</td></tr>}
            {!loading && entries.length === 0 && <tr><td className="p-4 text-center text-muted" colSpan={4}>No activity records.</td></tr>}
            {!loading && entries.map((entry) => (
              <tr key={entry.id}>
                <td className="p-3 text-xs text-muted">{formatDateTime(entry.created_at)}</td>
                <td className="p-3"><span className="rounded bg-background px-2 py-1 text-xs font-semibold text-secondary">{entry.action}</span></td>
                <td className="p-3 text-xs text-muted">{entry.entity_type ?? '-'}{entry.entity_id ? ` #${entry.entity_id}` : ''}</td>
                <td className="p-3 text-secondary">{entry.summary}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="flex items-center justify-end gap-2">
        <button className="btn" disabled={offset === 0} onClick={() => void load(Math.max(0, offset - limit))} type="button">Previous</button>
        <span className="text-sm text-muted">Offset {offset}</span>
        <button className="btn" disabled={entries.length < limit} onClick={() => void load(offset + limit)} type="button">Next</button>
      </div>
    </section>
  );
};

const nullable = (value: string) => {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
};

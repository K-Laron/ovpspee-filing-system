import { RefreshCw, ShieldAlert, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';

import { ConfirmDialog } from '../../components/ConfirmDialog';
import { formatDateOnly } from '../../lib/dates';
import { getUserErrorMessage } from '../../lib/errors';
import { cmd } from '../../lib/invoke';
import { useConfirmAction } from '../../lib/confirm';
import { useSessionStore } from '../../store/sessionStore';
import type { DocumentItem } from '../../types';

export const TrashManagement = () => {
  const sessionId = useSessionStore((state) => state.sessionId);
  const [documents, setDocuments] = useState<DocumentItem[]>([]);
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const { confirmAction, setConfirmAction, clearConfirmAction } = useConfirmAction();

  const loadTrash = async () => {
    if (!sessionId) return;
    setDocuments(await cmd<DocumentItem[]>('list_trash_documents', { sessionId }));
  };

  useEffect(() => {
    void loadTrash().catch((err) => setMessage(getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.')));
  }, [sessionId]);

  const purgeOne = async (documentId: number) => {
    if (!sessionId || busy) return;
    setBusy(true);
    try {
      await cmd<void>('purge_document', { sessionId, documentId });
      setMessage('Document purged.');
      await loadTrash();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not purge document.'));
    } finally {
      setBusy(false);
    }
  };

  const purgeAll = async () => {
    if (!sessionId || busy) return;
    setBusy(true);
    try {
      const count = await cmd<number>('empty_trash', { sessionId });
      setMessage(`${count} document(s) purged.`);
      await loadTrash();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not empty Trash.'));
    } finally {
      setBusy(false);
    }
  };

  const confirmPurgeOne = (doc: DocumentItem) => {
    setConfirmAction({
      title: 'Purge document permanently?',
      body: <>Purge <strong>{doc.document_name}</strong> permanently. This cannot be undone.</>,
      confirmLabel: 'Purge Document',
      requiredText: 'PURGE',
      onConfirm: () => purgeOne(doc.document_id)
    });
  };

  const confirmPurgeAll = () => {
    setConfirmAction({
      title: 'Empty Trash permanently?',
      body: `Purge ${documents.length} document(s) permanently. This cannot be undone.`,
      confirmLabel: 'Empty Trash',
      requiredText: 'PURGE',
      onConfirm: purgeAll
    });
  };

  const handleConfirmAction = async () => {
    if (!confirmAction) return;
    try {
      await confirmAction.onConfirm();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not complete confirmation action.'));
    } finally {
      clearConfirmAction();
    }
  };

  return (
    <section className="space-y-5">
      {confirmAction && (
        <ConfirmDialog
          body={confirmAction.body}
          confirmLabel={confirmAction.confirmLabel}
          onCancel={() => clearConfirmAction()}
          onConfirm={() => handleConfirmAction()}
          requiredText={confirmAction.requiredText}
          title={confirmAction.title}
        />
      )}

      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-secondary">Trash Management</h1>
          <p className="mt-1 text-sm text-muted">Permanent purge controls for IT Staff.</p>
        </div>
        <div className="flex gap-2">
          <button className="btn" onClick={() => void loadTrash().catch((err) => setMessage(getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.')))} type="button">
            <RefreshCw size={16} />
            Refresh
          </button>
          <button className="btn btn-primary" disabled={busy || documents.length === 0} onClick={confirmPurgeAll} type="button">
            <Trash2 size={16} />
            Empty Trash
          </button>
        </div>
      </div>

      <div className="flex gap-2 rounded border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">
        <ShieldAlert className="shrink-0" size={18} />
        Purge permanently deletes document records and attachment files.
      </div>

      {message && <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div>}

      <div className="overflow-hidden rounded border border-border bg-surface shadow-sm">
        <table className="w-full text-left text-sm">
          <thead className="border-b border-border bg-background text-xs uppercase text-muted">
            <tr><th className="p-3">Document</th><th className="p-3">Location</th><th className="p-3">Date</th><th className="p-3">Action</th></tr>
          </thead>
          <tbody>
            {documents.map((doc) => (
              <tr className="border-b border-border" key={doc.document_id}>
                <td className="p-3">
                  <p className="font-semibold text-secondary">{doc.document_name}</p>
                  <p className="text-xs text-muted">{doc.status} · {doc.attachment_count} file(s)</p>
                </td>
                <td className="p-3 text-muted">{doc.category_name}{doc.folder_name ? ` / ${doc.folder_name}` : ''}</td>
                <td className="p-3 text-muted">{formatDateOnly(doc.date_received)}</td>
                <td className="p-3">
                  <button className="btn" disabled={busy} onClick={() => confirmPurgeOne(doc)} type="button">
                    <Trash2 size={16} />
                    Purge
                  </button>
                </td>
              </tr>
            ))}
            {documents.length === 0 && (
              <tr><td className="p-4 text-sm text-muted" colSpan={4}>Trash is empty.</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
};

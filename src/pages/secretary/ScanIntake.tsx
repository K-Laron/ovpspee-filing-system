import { open } from '@tauri-apps/plugin-dialog';
import { FileScan, Link2, RefreshCw, Save, Trash2, Upload, X } from 'lucide-react';
import { FormEvent, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import {
  attachScanToDocument,
  fileScanAsDocument,
  importScanFiles,
  listDocumentOffices,
  listDocuments,
  listPublicCategories,
  listPublicFolders,
  listScanIntake,
  removeScanIntake,
  updateScanIntakeNotes
} from '../../lib/invoke';
import { useSessionStore } from '../../store/sessionStore';
import type { CategoryItem, DocumentItem, DocumentStatus, FolderItem, OfficeItem, ScanIntakeItem } from '../../types';

const today = new Date().toISOString().slice(0, 10);
const scanFilters = [
  {
    name: 'Scanned PDF and image files',
    extensions: ['pdf', 'jpg', 'jpeg', 'png', 'tif', 'tiff']
  }
];

const emptyForm = {
  documentName: '',
  categoryId: '',
  folderId: '',
  officeId: '',
  dateReceived: today,
  remarks: '',
  status: 'Filed' as DocumentStatus
};

const fileNameFromPath = (path: string) => path.split(/[\\/]/).pop() ?? path;
const normalizeSelectedPaths = (selected: string | string[] | null) => {
  if (!selected) return [];
  return Array.isArray(selected) ? selected : [selected];
};
const sizeLabel = (bytes: number) => `${Math.ceil(bytes / 1024)} KB`;

export const ScanIntake = () => {
  const navigate = useNavigate();
  const sessionId = useSessionStore((state) => state.sessionId);
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [items, setItems] = useState<ScanIntakeItem[]>([]);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [categories, setCategories] = useState<CategoryItem[]>([]);
  const [folders, setFolders] = useState<FolderItem[]>([]);
  const [offices, setOffices] = useState<OfficeItem[]>([]);
  const [documents, setDocuments] = useState<DocumentItem[]>([]);
  const [form, setForm] = useState(emptyForm);
  const [existingDocumentId, setExistingDocumentId] = useState('');
  const [notesDraft, setNotesDraft] = useState('');
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);

  const selectedItems = useMemo(
    () => items.filter((item) => selectedIds.includes(item.scan_intake_id)),
    [items, selectedIds]
  );

  const loadLookups = async () => {
    if (!sessionId) return;
    const [nextCategories, nextOffices, nextDocuments] = await Promise.all([
      listPublicCategories(),
      listDocumentOffices(sessionId),
      listDocuments({ sessionId })
    ]);
    setCategories(nextCategories);
    setOffices(nextOffices);
    setDocuments(nextDocuments);
  };

  const loadIntake = async () => {
    if (!sessionId) return;
    const rows = await listScanIntake(sessionId);
    setItems(rows);
    setSelectedIds((current) => current.filter((id) => rows.some((row) => row.scan_intake_id === id)));
  };

  useEffect(() => {
    void Promise.all([loadLookups(), loadIntake()]).catch((err) => setMessage(String(err)));
  }, [sessionId]);

  useEffect(() => {
    const categoryId = Number(form.categoryId);
    if (!categoryId) {
      setFolders([]);
      return;
    }
    void listPublicFolders(categoryId)
      .then(setFolders)
      .catch((err) => setMessage(String(err)));
  }, [form.categoryId]);

  useEffect(() => {
    const firstSelected = selectedItems[0];
    if (!firstSelected) {
      setNotesDraft('');
      return;
    }
    setNotesDraft(firstSelected.notes ?? '');
    if (!form.documentName) {
      setForm((current) => ({ ...current, documentName: firstSelected.original_file_name.replace(/\.[^.]+$/, '') }));
    }
  }, [selectedIds]);

  const chooseFiles = async () => {
    const selected = await open({
      multiple: true,
      directory: false,
      filters: scanFilters
    });
    const paths = normalizeSelectedPaths(selected);
    if (paths.length) setSelectedPaths((current) => Array.from(new Set([...current, ...paths])));
  };

  const importSelected = async () => {
    if (!sessionId || selectedPaths.length === 0 || busy) return;
    setBusy(true);
    setMessage('');
    try {
      await importScanFiles({ sessionId, sourcePaths: selectedPaths });
      setSelectedPaths([]);
      setMessage('Scan file(s) imported.');
      await loadIntake();
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  const toggleSelected = (id: number) => {
    setSelectedIds((current) => current.includes(id) ? current.filter((itemId) => itemId !== id) : [...current, id]);
  };

  const saveNotes = async () => {
    if (!sessionId || selectedItems.length !== 1) return;
    await updateScanIntakeNotes({
      sessionId,
      scanIntakeId: selectedItems[0].scan_intake_id,
      notes: notesDraft || null
    });
    setMessage('Notes saved.');
    await loadIntake();
  };

  const removeSelected = async () => {
    if (!sessionId || selectedIds.length === 0 || busy) return;
    setBusy(true);
    setMessage('');
    try {
      for (const scanIntakeId of selectedIds) {
        await removeScanIntake({ sessionId, scanIntakeId });
      }
      setSelectedIds([]);
      setMessage('Pending scan removed from active intake.');
      await loadIntake();
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  const fileAsDocument = async (event: FormEvent) => {
    event.preventDefault();
    if (!sessionId || selectedIds.length === 0 || busy) return;
    setBusy(true);
    setMessage('');
    try {
      const documentId = await fileScanAsDocument({
        sessionId,
        scanIntakeIds: selectedIds,
        documentName: form.documentName,
        categoryId: Number(form.categoryId),
        folderId: form.folderId ? Number(form.folderId) : null,
        officeId: form.officeId ? Number(form.officeId) : null,
        dateReceived: form.dateReceived,
        remarks: form.remarks || null,
        status: form.status
      });
      setSelectedIds([]);
      setForm(emptyForm);
      navigate(`/s/documents?created=${documentId}`, { replace: false });
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  const attachToDocument = async () => {
    if (!sessionId || selectedIds.length === 0 || !existingDocumentId || busy) return;
    setBusy(true);
    setMessage('');
    try {
      await attachScanToDocument({
        sessionId,
        scanIntakeIds: selectedIds,
        documentId: Number(existingDocumentId)
      });
      setSelectedIds([]);
      setExistingDocumentId('');
      setMessage('Scan file(s) attached to document.');
      await Promise.all([loadIntake(), loadLookups()]);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="space-y-5">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-bold text-secondary">Scan Intake</h1>
          <p className="mt-1 text-sm text-muted">Import scanned PDF/image files, then file them as documents.</p>
        </div>
        <button className="btn" onClick={() => void Promise.all([loadLookups(), loadIntake()]).catch((err) => setMessage(String(err)))} type="button">
          <RefreshCw size={16} />Refresh
        </button>
      </div>

      {message && <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div>}

      <div className="grid gap-5 xl:grid-cols-[0.9fr_1.1fr]">
        <div className="space-y-5">
          <div className="rounded border border-border bg-surface p-5 shadow-sm">
            <div className="mb-3 flex items-center gap-2">
              <FileScan size={18} className="text-primary" />
              <h2 className="font-semibold text-secondary">Import Scans</h2>
            </div>
            <button className="btn w-full justify-center" onClick={() => void chooseFiles().catch((err) => setMessage(String(err)))} type="button">
              <Upload size={16} />Choose Files
            </button>
            <p className="mt-2 text-xs text-muted">Allowed: PDF, JPG, PNG, TIFF. Max 1 GB each; files above 250 MB are marked large.</p>
            <div className="mt-4 space-y-2">
              {selectedPaths.length === 0 ? (
                <p className="rounded border border-dashed border-border p-3 text-sm text-muted">No scan files selected.</p>
              ) : selectedPaths.map((sourcePath) => (
                <div className="flex items-center justify-between gap-3 rounded border border-border p-3 text-sm" key={sourcePath}>
                  <div className="min-w-0">
                    <p className="truncate font-medium text-secondary">{fileNameFromPath(sourcePath)}</p>
                    <p className="truncate text-xs text-muted">{sourcePath}</p>
                  </div>
                  <button className="icon-btn shrink-0" onClick={() => setSelectedPaths((current) => current.filter((path) => path !== sourcePath))} title="Remove selected file" type="button">
                    <X size={15} />
                  </button>
                </div>
              ))}
            </div>
            <button className="btn btn-primary mt-4 w-full justify-center" disabled={busy || selectedPaths.length === 0} onClick={() => void importSelected()} type="button">
              <Upload size={16} />Import Selected
            </button>
          </div>

          <div className="overflow-hidden rounded border border-border bg-surface shadow-sm">
            <div className="border-b border-border p-4">
              <h2 className="font-semibold text-secondary">Pending Intake</h2>
              <p className="text-sm text-muted">{selectedIds.length} selected</p>
            </div>
            <div className="divide-y divide-border">
              {items.length === 0 ? (
                <p className="p-4 text-sm text-muted">No pending scan intake files.</p>
              ) : items.map((item) => (
                <label className="flex cursor-pointer gap-3 p-4 hover:bg-background" key={item.scan_intake_id}>
                  <input
                    checked={selectedIds.includes(item.scan_intake_id)}
                    className="mt-1"
                    onChange={() => toggleSelected(item.scan_intake_id)}
                    type="checkbox"
                  />
                  <span className="min-w-0 flex-1">
                    <span className="block truncate font-medium text-secondary">{item.original_file_name}</span>
                    <span className="block text-xs text-muted">{item.mime_type} · {sizeLabel(item.file_size_bytes)}{item.is_large ? ' · Large file' : ''}</span>
                    {item.notes && <span className="mt-1 block text-xs text-muted">{item.notes}</span>}
                  </span>
                </label>
              ))}
            </div>
          </div>
        </div>

        <div className="space-y-5">
          <form className="rounded border border-border bg-surface p-5 shadow-sm" onSubmit={fileAsDocument}>
            <div className="mb-4 flex items-center gap-2">
              <Save size={18} className="text-primary" />
              <h2 className="font-semibold text-secondary">File as New Document</h2>
            </div>
            <div className="grid gap-4 md:grid-cols-2">
              <label className="md:col-span-2">
                <span className="form-label">Document title</span>
                <input className="input" value={form.documentName} onChange={(e) => setForm({ ...form, documentName: e.target.value })} required />
              </label>
              <label>
                <span className="form-label">Category</span>
                <select className="input" value={form.categoryId} onChange={(e) => setForm({ ...form, categoryId: e.target.value, folderId: '' })} required>
                  <option value="">Select category</option>
                  {categories.map((category) => <option key={category.category_id} value={category.category_id}>{category.category_name}</option>)}
                </select>
              </label>
              <label>
                <span className="form-label">Folder</span>
                <select className="input" value={form.folderId} onChange={(e) => setForm({ ...form, folderId: e.target.value })}>
                  <option value="">Category root</option>
                  {folders.map((folder) => <option key={folder.folder_id} value={folder.folder_id}>{folder.folder_name}</option>)}
                </select>
              </label>
              <label>
                <span className="form-label">Sender office</span>
                <select className="input" value={form.officeId} onChange={(e) => setForm({ ...form, officeId: e.target.value })}>
                  <option value="">Not specified</option>
                  {offices.map((office) => <option key={office.office_id} value={office.office_id}>{office.office_name}</option>)}
                </select>
              </label>
              <label>
                <span className="form-label">Date received</span>
                <input className="input" max={today} type="date" value={form.dateReceived} onChange={(e) => setForm({ ...form, dateReceived: e.target.value })} required />
              </label>
              <label>
                <span className="form-label">Status</span>
                <select className="input" value={form.status} onChange={(e) => setForm({ ...form, status: e.target.value as DocumentStatus })}>
                  <option>Filed</option>
                  <option>Archived</option>
                  <option>Confidential</option>
                  <option>Other</option>
                </select>
              </label>
              <label className="md:col-span-2">
                <span className="form-label">Remarks</span>
                <textarea className="input min-h-24" value={form.remarks} onChange={(e) => setForm({ ...form, remarks: e.target.value })} />
              </label>
            </div>
            <button className="btn btn-primary mt-4" disabled={busy || selectedIds.length === 0} type="submit">
              <Save size={16} />File Selected Scan(s)
            </button>
          </form>

          <div className="rounded border border-border bg-surface p-5 shadow-sm">
            <div className="mb-4 flex items-center gap-2">
              <Link2 size={18} className="text-primary" />
              <h2 className="font-semibold text-secondary">Attach to Existing Document</h2>
            </div>
            <div className="grid gap-3 md:grid-cols-[1fr_auto]">
              <label>
                <span className="form-label">Document</span>
                <select className="input" value={existingDocumentId} onChange={(e) => setExistingDocumentId(e.target.value)}>
                  <option value="">Select document</option>
                  {documents.map((document) => <option key={document.document_id} value={document.document_id}>{document.document_name}</option>)}
                </select>
              </label>
              <button className="btn btn-primary self-end" disabled={busy || selectedIds.length === 0 || !existingDocumentId} onClick={() => void attachToDocument()} type="button">
                <Link2 size={16} />Attach
              </button>
            </div>
          </div>

          <div className="rounded border border-border bg-surface p-5 shadow-sm">
            <h2 className="mb-4 font-semibold text-secondary">Selected Scan Metadata</h2>
            {selectedItems.length !== 1 ? (
              <p className="text-sm text-muted">Select one pending scan to edit notes.</p>
            ) : (
              <div className="space-y-3">
                <p className="text-sm text-secondary">{selectedItems[0].original_file_name}</p>
                <textarea className="input min-h-24" value={notesDraft} onChange={(e) => setNotesDraft(e.target.value)} />
                <button className="btn" onClick={() => void saveNotes().catch((err) => setMessage(String(err)))} type="button">
                  <Save size={16} />Save Notes
                </button>
              </div>
            )}
            <button className="btn mt-4" disabled={busy || selectedIds.length === 0} onClick={() => void removeSelected()} type="button">
              <Trash2 size={16} />Remove Pending
            </button>
          </div>
        </div>
      </div>
    </section>
  );
};

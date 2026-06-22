import { open } from '@tauri-apps/plugin-dialog';
import { AlertTriangle, FilePlus, Paperclip, Save, X } from 'lucide-react';
import { FormEvent, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { EmptyState } from '../../components/EmptyState';
import { formatDateInputValue } from '../../lib/dates';
import { cmd } from '../../lib/invoke';
import { getUserErrorMessage } from '../../lib/errors';
import { useSessionStore } from '../../store/sessionStore';
import type { CategoryItem, DocumentStatus, FolderItem, OfficeItem } from '../../types';

const today = formatDateInputValue();
const emptyForm = {
  documentName: '',
  categoryId: '',
  folderId: '',
  officeId: '',
  dateReceived: today,
  remarks: '',
  status: 'Filed' as DocumentStatus
};

const attachmentFilters = [
  {
    name: 'Allowed document and image files',
    extensions: ['pdf', 'doc', 'docx', 'xls', 'xlsx', 'jpg', 'jpeg', 'png', 'tif', 'tiff', 'txt']
  }
];

const fileNameFromPath = (path: string) => path.split(/[\\/]/).pop() ?? path;

const normalizeSelectedPaths = (selected: string | string[] | null) => {
  if (!selected) return [];
  return Array.isArray(selected) ? selected : [selected];
};

export const AddDocument = () => {
  const navigate = useNavigate();
  const sessionId = useSessionStore((state) => state.sessionId);
  const [categories, setCategories] = useState<CategoryItem[]>([]);
  const [folders, setFolders] = useState<FolderItem[]>([]);
  const [offices, setOffices] = useState<OfficeItem[]>([]);
  const [form, setForm] = useState(emptyForm);
  const [attachmentPaths, setAttachmentPaths] = useState<string[]>([]);
  const [message, setMessage] = useState('');
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!sessionId) return;
    void Promise.all([cmd<CategoryItem[]>('list_public_categories'), cmd<OfficeItem[]>('list_document_offices', { sessionId })])
      .then(([nextCategories, nextOffices]) => {
        setCategories(nextCategories);
        setOffices(nextOffices);
      })
      .catch((err) => setMessage(getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.')));
  }, [sessionId]);

  useEffect(() => {
    const categoryId = Number(form.categoryId);
    if (!categoryId) {
      setFolders([]);
      return;
    }
    void cmd<FolderItem[]>('list_public_folders', { categoryId })
      .then(setFolders)
      .catch((err) => setMessage(getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.')));
  }, [form.categoryId]);

  const selectedStatus = form.status;
  const attachmentList = useMemo(() => attachmentPaths.map((path) => path.trim()).filter(Boolean), [attachmentPaths]);

  const chooseAttachments = async () => {
    const selected = await open({
      multiple: true,
      directory: false,
      filters: attachmentFilters
    });
    const paths = normalizeSelectedPaths(selected);
    if (paths.length) {
      setAttachmentPaths((current) => Array.from(new Set([...current, ...paths])));
    }
  };

  const removeSelectedAttachment = (sourcePath: string) => {
    setAttachmentPaths((current) => current.filter((path) => path !== sourcePath));
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!sessionId || saving) return;
    setSaving(true);
    setMessage('');
    try {
      const documentId = await cmd<number>('create_document', {
        sessionId,
        documentName: form.documentName,
        categoryId: Number(form.categoryId),
        folderId: form.folderId ? Number(form.folderId) : null,
        officeId: form.officeId ? Number(form.officeId) : null,
        dateReceived: form.dateReceived,
        remarks: form.remarks || null,
        status: form.status
      });
      for (const [index, sourcePath] of attachmentList.entries()) {
        await cmd<number>('add_attachment', { sessionId, documentId, sourcePath, sortOrder: index + 1 });
      }
      navigate(`/s/documents?created=${documentId}`, { replace: true });
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not save the document. Check the required fields and try again.'));
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="max-w-5xl">
      <div className="mb-5 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-secondary">Add Document</h1>
          <p className="mt-1 text-sm text-muted">Use Add Document to create an official document record with metadata and attachments.</p>
        </div>
        <FilePlus className="text-primary" size={28} />
      </div>

      <form className="grid gap-5 lg:grid-cols-[1.1fr_0.9fr]" onSubmit={submit}>
        <div className="rounded border border-border bg-surface p-5 shadow-sm">
          {message && <div className="mb-4 rounded border border-red-200 bg-red-50 p-3 text-sm text-red-700">{message}</div>}
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
              <textarea className="input min-h-28" value={form.remarks} onChange={(e) => setForm({ ...form, remarks: e.target.value })} />
            </label>
          </div>
          {selectedStatus === 'Confidential' && (
            <div className="mt-4 flex gap-2 rounded border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">
              <AlertTriangle className="shrink-0" size={18} />
              Confidential documents are hidden from Staff/Head Viewer.
            </div>
          )}
        </div>

        <div className="rounded border border-border bg-surface p-5 shadow-sm">
          <div className="mb-3 flex items-center gap-2">
            <Paperclip size={18} className="text-primary" />
            <h2 className="font-semibold text-secondary">Attachments</h2>
          </div>
          <button className="btn w-full justify-center" onClick={() => void chooseAttachments().catch((err) => setMessage(getUserErrorMessage(err, 'Could not add the attachment. Check the file type and try again.')))} type="button">
            <Paperclip size={16} />
            Choose Files
          </button>
          <p className="mt-2 text-xs text-muted">Allowed: PDF, DOC/DOCX, XLS/XLSX, JPG, PNG, TIFF, TXT. Max 1 GB each; backend rejects unsupported or oversized files.</p>
          <div className="mt-4 space-y-2">
            {attachmentList.length === 0 ? (
              <EmptyState
                actionLabel="Choose Files"
                message="Choose PDF, Office, image, or text files to attach to this document."
                onAction={() => void chooseAttachments().catch((err) => setMessage(getUserErrorMessage(err, 'Could not add the attachment. Check the file type and try again.')))}
                title="No attachments selected"
              />
            ) : attachmentList.map((sourcePath) => (
              <div className="flex items-center justify-between gap-3 rounded border border-border p-3 text-sm" key={sourcePath}>
                <div className="min-w-0">
                  <p className="truncate font-medium text-secondary">{fileNameFromPath(sourcePath)}</p>
                  <p className="truncate text-xs text-muted">{sourcePath}</p>
                </div>
                <button aria-label={`Remove selected file ${fileNameFromPath(sourcePath)}`} className="icon-btn shrink-0" onClick={() => removeSelectedAttachment(sourcePath)} title="Remove selected file" type="button">
                  <X size={15} />
                </button>
              </div>
            ))}
          </div>
          <button className="btn btn-primary mt-5 w-full" disabled={saving} type="submit">
            <Save size={16} />
            {saving ? 'Saving...' : 'Save Document'}
          </button>
        </div>
      </form>
    </section>
  );
};

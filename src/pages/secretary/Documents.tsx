import { open } from '@tauri-apps/plugin-dialog';
import { Edit3, ExternalLink, Paperclip, RefreshCw, Save, Search, Trash2, X } from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';

import {
  addAttachment,
  getAttachmentFilePath,
  getDocument,
  listDocumentOffices,
  listDocuments,
  listPublicCategories,
  listPublicFolders,
  removeAttachment,
  updateDocument
} from '../../lib/invoke';
import { useSessionStore } from '../../store/sessionStore';
import type { CategoryItem, DocumentDetail, DocumentItem, DocumentStatus, FolderItem, OfficeItem } from '../../types';

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

export const Documents = () => {
  const sessionId = useSessionStore((state) => state.sessionId);
  const [categories, setCategories] = useState<CategoryItem[]>([]);
  const [folders, setFolders] = useState<FolderItem[]>([]);
  const [offices, setOffices] = useState<OfficeItem[]>([]);
  const [documents, setDocuments] = useState<DocumentItem[]>([]);
  const [detail, setDetail] = useState<DocumentDetail | null>(null);
  const [search, setSearch] = useState('');
  const [categoryId, setCategoryId] = useState('');
  const [folderId, setFolderId] = useState('');
  const [officeId, setOfficeId] = useState('');
  const [editing, setEditing] = useState(false);
  const [pendingAttachmentPaths, setPendingAttachmentPaths] = useState<string[]>([]);
  const [message, setMessage] = useState('');

  const loadLookups = async () => {
    if (!sessionId) return;
    const [nextCategories, nextOffices] = await Promise.all([listPublicCategories(), listDocumentOffices(sessionId)]);
    setCategories(nextCategories);
    setOffices(nextOffices);
  };

  const loadFolders = async (nextCategoryId: string) => {
    setFolderId('');
    if (!nextCategoryId) {
      setFolders([]);
      return;
    }
    setFolders(await listPublicFolders(Number(nextCategoryId)));
  };

  const loadDocuments = async () => {
    if (!sessionId) return;
    const rows = await listDocuments({
      sessionId,
      search: search || null,
      categoryId: categoryId ? Number(categoryId) : null,
      folderId: folderId ? Number(folderId) : null,
      officeId: officeId ? Number(officeId) : null
    });
    setDocuments(rows);
    if (!detail && rows[0]) {
      await openDetail(rows[0].document_id);
    }
  };

  const openDetail = async (documentId: number) => {
    if (!sessionId) return;
    setDetail(await getDocument(sessionId, documentId));
    setEditing(false);
  };

  useEffect(() => {
    void loadLookups().catch((err) => setMessage(String(err)));
  }, [sessionId]);

  useEffect(() => {
    void loadDocuments().catch((err) => setMessage(String(err)));
  }, [sessionId]);

  const submitSearch = (event: FormEvent) => {
    event.preventDefault();
    void loadDocuments().catch((err) => setMessage(String(err)));
  };

  const saveEdit = async () => {
    if (!sessionId || !detail) return;
    await updateDocument({
      sessionId,
      documentId: detail.document.document_id,
      documentName: detail.document.document_name,
      categoryId: detail.document.category_id,
      folderId: detail.document.folder_id,
      officeId: detail.document.office_id,
      dateReceived: detail.document.date_received,
      remarks: detail.document.remarks,
      status: detail.document.status
    });
    setMessage('Document updated.');
    await openDetail(detail.document.document_id);
    await loadDocuments();
  };

  const chooseAttachments = async () => {
    const selected = await open({
      multiple: true,
      directory: false,
      filters: attachmentFilters
    });
    const paths = normalizeSelectedPaths(selected);
    if (paths.length) {
      setPendingAttachmentPaths((current) => Array.from(new Set([...current, ...paths])));
    }
  };

  const attach = async () => {
    if (!sessionId || !detail || pendingAttachmentPaths.length === 0) return;
    for (const [index, sourcePath] of pendingAttachmentPaths.entries()) {
      await addAttachment({
        sessionId,
        documentId: detail.document.document_id,
        sourcePath,
        sortOrder: detail.attachments.length + index + 1
      });
    }
    setPendingAttachmentPaths([]);
    setMessage('Attachment added.');
    await openDetail(detail.document.document_id);
  };

  const remove = async (attachmentId: number) => {
    if (!sessionId || !detail) return;
    await removeAttachment({ sessionId, attachmentId });
    setMessage('Attachment removed.');
    await openDetail(detail.document.document_id);
  };

  const showPath = async (attachmentId: number) => {
    if (!sessionId) return;
    const path = await getAttachmentFilePath(attachmentId, sessionId);
    setMessage(path);
  };

  return (
    <section className="space-y-5">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-secondary">Documents</h1>
          <p className="mt-1 text-sm text-muted">Secretary filing list and document detail.</p>
        </div>
        <button className="btn" onClick={() => void loadDocuments()} type="button"><RefreshCw size={16} />Refresh</button>
      </div>

      <form className="grid gap-3 rounded border border-border bg-surface p-4 shadow-sm md:grid-cols-[1fr_180px_180px_180px_auto]" onSubmit={submitSearch}>
        <label>
          <span className="form-label">Search</span>
          <input className="input" value={search} onChange={(e) => setSearch(e.target.value)} />
        </label>
        <label>
          <span className="form-label">Category</span>
          <select className="input" value={categoryId} onChange={(e) => { setCategoryId(e.target.value); void loadFolders(e.target.value); }}>
            <option value="">All</option>
            {categories.map((category) => <option key={category.category_id} value={category.category_id}>{category.category_name}</option>)}
          </select>
        </label>
        <label>
          <span className="form-label">Folder</span>
          <select className="input" value={folderId} onChange={(e) => setFolderId(e.target.value)}>
            <option value="">All</option>
            {folders.map((folder) => <option key={folder.folder_id} value={folder.folder_id}>{folder.folder_name}</option>)}
          </select>
        </label>
        <label>
          <span className="form-label">Office</span>
          <select className="input" value={officeId} onChange={(e) => setOfficeId(e.target.value)}>
            <option value="">All</option>
            {offices.map((office) => <option key={office.office_id} value={office.office_id}>{office.office_name}</option>)}
          </select>
        </label>
        <button className="btn btn-primary self-end" type="submit"><Search size={16} />Apply</button>
      </form>

      {message && <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div>}

      <div className="grid gap-5 xl:grid-cols-[0.9fr_1.1fr]">
        <div className="overflow-hidden rounded border border-border bg-surface shadow-sm">
          <table className="w-full text-left text-sm">
            <thead className="border-b border-border bg-background text-xs uppercase text-muted">
              <tr><th className="p-3">Document</th><th className="p-3">Location</th><th className="p-3">Date</th></tr>
            </thead>
            <tbody>
              {documents.map((doc) => (
                <tr className="cursor-pointer border-b border-border hover:bg-background" key={doc.document_id} onClick={() => void openDetail(doc.document_id)}>
                  <td className="p-3"><p className="font-semibold text-secondary">{doc.document_name}</p><p className="text-xs text-muted">{doc.status} · {doc.attachment_count} file(s)</p></td>
                  <td className="p-3 text-muted">{doc.category_name}{doc.folder_name ? ` / ${doc.folder_name}` : ''}</td>
                  <td className="p-3 text-muted">{doc.date_received}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="rounded border border-border bg-surface p-5 shadow-sm">
          {!detail ? <p className="text-sm text-muted">Select a document.</p> : (
            <div className="space-y-4">
              <div className="flex items-start justify-between gap-3">
                <div>
                  {editing ? (
                    <input className="input text-lg font-semibold" value={detail.document.document_name} onChange={(e) => setDetail({ ...detail, document: { ...detail.document, document_name: e.target.value } })} />
                  ) : (
                    <h2 className="text-xl font-bold text-secondary">{detail.document.document_name}</h2>
                  )}
                  <p className="mt-1 text-sm text-muted">{detail.document.category_name}{detail.document.folder_name ? ` / ${detail.document.folder_name}` : ''}</p>
                </div>
                <button className="btn" onClick={() => setEditing(!editing)} type="button"><Edit3 size={16} />Edit</button>
              </div>

              <div className="grid gap-3 md:grid-cols-2">
                <label>
                  <span className="form-label">Date received</span>
                  <input className="input" disabled={!editing} type="date" value={detail.document.date_received} onChange={(e) => setDetail({ ...detail, document: { ...detail.document, date_received: e.target.value } })} />
                </label>
                <label>
                  <span className="form-label">Status</span>
                  <select className="input" disabled={!editing} value={detail.document.status} onChange={(e) => setDetail({ ...detail, document: { ...detail.document, status: e.target.value as DocumentStatus } })}>
                    <option>Filed</option><option>Archived</option><option>Confidential</option><option>Other</option>
                  </select>
                </label>
                <label className="md:col-span-2">
                  <span className="form-label">Remarks</span>
                  <textarea className="input min-h-24" disabled={!editing} value={detail.document.remarks ?? ''} onChange={(e) => setDetail({ ...detail, document: { ...detail.document, remarks: e.target.value } })} />
                </label>
              </div>
              {editing && <button className="btn btn-primary" onClick={() => void saveEdit().catch((err) => setMessage(String(err)))} type="button"><Save size={16} />Save Changes</button>}

              <div className="border-t border-border pt-4">
                <div className="mb-3 flex items-center gap-2"><Paperclip size={17} className="text-primary" /><h3 className="font-semibold text-secondary">Attachments</h3></div>
                <div className="space-y-2">
                  {detail.attachments.map((file) => (
                    <div className="flex items-center justify-between rounded border border-border p-3 text-sm" key={file.attachment_id}>
                      <div><p className="font-medium text-secondary">{file.original_file_name}</p><p className="text-xs text-muted">{Math.ceil(file.file_size_bytes / 1024)} KB</p></div>
                      <div className="flex gap-2">
                        <button className="icon-btn" title="Show file path" onClick={() => void showPath(file.attachment_id).catch((err) => setMessage(String(err)))} type="button"><ExternalLink size={15} /></button>
                        <button className="icon-btn" title="Remove attachment" onClick={() => void remove(file.attachment_id).catch((err) => setMessage(String(err)))} type="button"><Trash2 size={15} /></button>
                      </div>
                    </div>
                  ))}
                </div>
                <div className="mt-3 space-y-3">
                  <button className="btn" onClick={() => void chooseAttachments().catch((err) => setMessage(String(err)))} type="button">
                    <Paperclip size={16} />
                    Add Attachments
                  </button>
                  {pendingAttachmentPaths.length > 0 && (
                    <div className="space-y-2">
                      {pendingAttachmentPaths.map((sourcePath) => (
                        <div className="flex items-center justify-between gap-3 rounded border border-border p-3 text-sm" key={sourcePath}>
                          <div className="min-w-0">
                            <p className="truncate font-medium text-secondary">{fileNameFromPath(sourcePath)}</p>
                            <p className="truncate text-xs text-muted">{sourcePath}</p>
                          </div>
                          <button
                            className="icon-btn shrink-0"
                            onClick={() => setPendingAttachmentPaths((current) => current.filter((path) => path !== sourcePath))}
                            title="Remove selected file"
                            type="button"
                          >
                            <X size={15} />
                          </button>
                        </div>
                      ))}
                      <button className="btn btn-primary" onClick={() => void attach().catch((err) => setMessage(String(err)))} type="button">
                        Save Selected Attachments
                      </button>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </section>
  );
};

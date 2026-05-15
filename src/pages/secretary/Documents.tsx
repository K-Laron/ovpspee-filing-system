import { open, save } from '@tauri-apps/plugin-dialog';
import { Download, Edit3, ExternalLink, Eye, EyeOff, MoveRight, Paperclip, Printer, RefreshCw, RotateCcw, Save, Search, Trash2, X } from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';

import { AttachmentPreview } from '../../components/AttachmentPreview';
import {
  addAttachment,
  exportDocumentPdf,
  getAttachmentFilePath,
  getDocument,
  listPrintPrinters,
  listDocumentOffices,
  listDocuments,
  listPublicCategories,
  listPublicFolders,
  listTrashDocuments,
  moveDocument,
  removeAttachment,
  restoreDocument,
  setDocumentStatus,
  setDocumentHidden,
  trashDocument,
  updateDocument,
  printDocumentPdf
} from '../../lib/invoke';
import { useSessionStore } from '../../store/sessionStore';
import type { CategoryItem, DocumentDetail, DocumentItem, DocumentStatus, FolderItem, OfficeItem, PrinterDevice } from '../../types';

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
  const [printers, setPrinters] = useState<PrinterDevice[]>([]);
  const [selectedPrinterId, setSelectedPrinterId] = useState('');
  const [copies, setCopies] = useState(1);
  const [documents, setDocuments] = useState<DocumentItem[]>([]);
  const [detail, setDetail] = useState<DocumentDetail | null>(null);
  const [search, setSearch] = useState('');
  const [categoryId, setCategoryId] = useState('');
  const [folderId, setFolderId] = useState('');
  const [officeId, setOfficeId] = useState('');
  const [statusFilter, setStatusFilter] = useState('');
  const [editing, setEditing] = useState(false);
  const [moving, setMoving] = useState(false);
  const [moveCategoryId, setMoveCategoryId] = useState('');
  const [moveFolderId, setMoveFolderId] = useState('');
  const [moveFolders, setMoveFolders] = useState<FolderItem[]>([]);
  const [statusDraft, setStatusDraft] = useState<DocumentStatus>('Filed');
  const [pendingAttachmentPaths, setPendingAttachmentPaths] = useState<string[]>([]);
  const [message, setMessage] = useState('');
  const [view, setView] = useState<'active' | 'trash'>('active');
  const [exporting, setExporting] = useState(false);
  const [printing, setPrinting] = useState(false);
  const [previewAttachmentId, setPreviewAttachmentId] = useState<number | null>(null);

  const loadLookups = async () => {
    if (!sessionId) return;
    const [nextCategories, nextOffices, nextPrinters] = await Promise.all([
      listPublicCategories(),
      listDocumentOffices(sessionId),
      listPrintPrinters(sessionId)
    ]);
    setCategories(nextCategories);
    setOffices(nextOffices);
    setPrinters(nextPrinters);
    setSelectedPrinterId((current) => current || nextPrinters.find((printer) => printer.is_default)?.printer_id || nextPrinters[0]?.printer_id || '');
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
    let rows = view === 'trash'
      ? await listTrashDocuments(sessionId)
      : await listDocuments({
          sessionId,
          search: search || null,
          categoryId: categoryId ? Number(categoryId) : null,
          folderId: folderId ? Number(folderId) : null,
          officeId: officeId ? Number(officeId) : null
        });
    if (view === 'active' && statusFilter) {
      rows = rows.filter((row) => row.status === statusFilter);
    }
    setDocuments(rows);
    if (detail && !rows.some((row) => row.document_id === detail.document.document_id)) {
      setDetail(null);
    }
    if (!detail && rows[0]) {
      await openDetail(rows[0].document_id);
    }
  };

  const openDetail = async (documentId: number) => {
    if (!sessionId) return;
    const nextDetail = await getDocument(sessionId, documentId);
    setDetail(nextDetail);
    setStatusDraft(nextDetail.document.status);
    setMoveCategoryId(String(nextDetail.document.category_id));
    setMoveFolderId(nextDetail.document.folder_id ? String(nextDetail.document.folder_id) : '');
    setMoveFolders(await listPublicFolders(nextDetail.document.category_id));
    setEditing(false);
    setMoving(false);
    setPreviewAttachmentId(nextDetail.attachments[0]?.attachment_id ?? null);
  };

  useEffect(() => {
    void loadLookups().catch((err) => setMessage(String(err)));
  }, [sessionId]);

  useEffect(() => {
    void loadDocuments().catch((err) => setMessage(String(err)));
  }, [sessionId, view]);

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

  const toggleHidden = async () => {
    if (!sessionId || !detail) return;
    const nextHidden = !detail.document.is_hidden;
    await setDocumentHidden({
      sessionId,
      documentId: detail.document.document_id,
      isHidden: nextHidden
    });
    setMessage(nextHidden ? 'Document hidden.' : 'Document unhidden.');
    await openDetail(detail.document.document_id);
    await loadDocuments();
  };

  const moveToTrash = async () => {
    if (!sessionId || !detail) return;
    await trashDocument({ sessionId, documentId: detail.document.document_id });
    setMessage('Document moved to trash.');
    setDetail(null);
    await loadDocuments();
  };

  const restore = async () => {
    if (!sessionId || !detail) return;
    await restoreDocument({ sessionId, documentId: detail.document.document_id });
    setMessage('Document restored.');
    setDetail(null);
    await loadDocuments();
  };

  const loadMoveFolders = async (nextCategoryId: string) => {
    setMoveCategoryId(nextCategoryId);
    setMoveFolderId('');
    setMoveFolders(nextCategoryId ? await listPublicFolders(Number(nextCategoryId)) : []);
  };

  const saveMove = async () => {
    if (!sessionId || !detail || !moveCategoryId) return;
    await moveDocument({
      sessionId,
      documentId: detail.document.document_id,
      categoryId: Number(moveCategoryId),
      folderId: moveFolderId ? Number(moveFolderId) : null
    });
    setMessage('Document moved.');
    await openDetail(detail.document.document_id);
    await loadDocuments();
  };

  const saveStatus = async () => {
    if (!sessionId || !detail) return;
    await setDocumentStatus({
      sessionId,
      documentId: detail.document.document_id,
      status: statusDraft
    });
    setMessage(statusDraft === 'Confidential' ? 'Status changed. Document is now hidden.' : 'Status changed.');
    await openDetail(detail.document.document_id);
    await loadDocuments();
  };

  const exportPdf = async () => {
    if (!sessionId || !detail || exporting) return;
    setExporting(true);
    setMessage('');
    try {
      const outputPath = await save({
        defaultPath: `${safeFileName(detail.document.document_name)}.pdf`,
        filters: [{ name: 'PDF', extensions: ['pdf'] }]
      });
      if (!outputPath) return;
      const savedPath = await exportDocumentPdf({
        sessionId,
        documentId: detail.document.document_id,
        outputPath
      });
      setMessage(`Exported PDF: ${savedPath}`);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setExporting(false);
    }
  };

  const printPdf = async () => {
    if (!sessionId || !detail || !selectedPrinterId || printing) return;
    setPrinting(true);
    setMessage('');
    try {
      const result = await printDocumentPdf({
        sessionId,
        documentId: detail.document.document_id,
        printerId: selectedPrinterId,
        copies
      });
      setMessage(`Print submitted to ${result.printer_name}.`);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setPrinting(false);
    }
  };

  const isTrashView = view === 'trash';

  return (
    <section className="space-y-5">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-secondary">Documents</h1>
          <p className="mt-1 text-sm text-muted">Secretary filing list and document detail.</p>
        </div>
        <button className="btn" onClick={() => void loadDocuments()} type="button"><RefreshCw size={16} />Refresh</button>
      </div>

      <div className="inline-flex rounded border border-border bg-surface p-1 text-sm shadow-sm">
        <button className={view === 'active' ? 'btn btn-primary' : 'btn'} onClick={() => { setView('active'); setDetail(null); }} type="button">Documents</button>
        <button className={view === 'trash' ? 'btn btn-primary' : 'btn'} onClick={() => { setView('trash'); setDetail(null); }} type="button">Trash</button>
      </div>

      {!isTrashView && <form className="grid gap-3 rounded border border-border bg-surface p-4 shadow-sm md:grid-cols-[1fr_160px_160px_160px_150px_auto]" onSubmit={submitSearch}>
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
        <label>
          <span className="form-label">Status</span>
          <select className="input" value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)}>
            <option value="">All</option>
            <option>Filed</option>
            <option>Archived</option>
            <option>Confidential</option>
            <option>Other</option>
          </select>
        </label>
        <button className="btn btn-primary self-end" type="submit"><Search size={16} />Apply</button>
      </form>}

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
                  <td className="p-3">
                    <p className="font-semibold text-secondary">{doc.document_name}</p>
                    <p className="text-xs text-muted">
                      <span className="rounded bg-background px-2 py-0.5 text-[11px] font-semibold text-secondary">{doc.status}</span> · {doc.attachment_count} file(s){doc.is_hidden ? ' · Hidden' : ''}{doc.is_trashed ? ' · Trashed' : ''}
                    </p>
                  </td>
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
                  <p className="mt-2 inline-flex rounded bg-background px-2 py-1 text-xs font-semibold text-secondary">{detail.document.status}{detail.document.is_hidden ? ' · Hidden' : ''}</p>
                </div>
                <div className="flex flex-wrap justify-end gap-2">
                  {!isTrashView && <button className="btn" onClick={() => setEditing(!editing)} type="button"><Edit3 size={16} />Edit</button>}
                  {!isTrashView && <button className="btn btn-primary" disabled={exporting} onClick={() => void exportPdf()} type="button"><Download size={16} />{exporting ? 'Exporting...' : 'Export PDF'}</button>}
                  {!isTrashView && <button className="btn" onClick={() => setMoving(!moving)} type="button"><MoveRight size={16} />Move</button>}
                  {!isTrashView && (
                    <button className="btn" onClick={() => void toggleHidden().catch((err) => setMessage(String(err)))} type="button">
                      {detail.document.is_hidden ? <Eye size={16} /> : <EyeOff size={16} />}
                      {detail.document.is_hidden ? 'Unhide' : 'Hide'}
                    </button>
                  )}
                  {!isTrashView && <button className="btn" onClick={() => void moveToTrash().catch((err) => setMessage(String(err)))} type="button"><Trash2 size={16} />Move to Trash</button>}
                  {isTrashView && <button className="btn btn-primary" onClick={() => void restore().catch((err) => setMessage(String(err)))} type="button"><RotateCcw size={16} />Restore</button>}
                </div>
              </div>

              <div className="grid gap-3 md:grid-cols-2">
                <label>
                  <span className="form-label">Date received</span>
                  <input className="input" disabled={!editing || isTrashView} type="date" value={detail.document.date_received} onChange={(e) => setDetail({ ...detail, document: { ...detail.document, date_received: e.target.value } })} />
                </label>
                <label>
                  <span className="form-label">Status</span>
                  <input className="input" disabled value={detail.document.status} />
                </label>
                <label className="md:col-span-2">
                  <span className="form-label">Remarks</span>
                  <textarea className="input min-h-24" disabled={!editing || isTrashView} value={detail.document.remarks ?? ''} onChange={(e) => setDetail({ ...detail, document: { ...detail.document, remarks: e.target.value } })} />
                </label>
              </div>
              {editing && !isTrashView && <button className="btn btn-primary" onClick={() => void saveEdit().catch((err) => setMessage(String(err)))} type="button"><Save size={16} />Save Changes</button>}

              {!isTrashView && <div className="grid gap-3 rounded border border-border bg-background p-4 md:grid-cols-[1fr_110px_auto]">
                <label>
                  <span className="form-label">Printer</span>
                  <select className="input" value={selectedPrinterId} onChange={(e) => setSelectedPrinterId(e.target.value)}>
                    <option value="">Select printer</option>
                    {printers.map((printer) => <option key={printer.printer_id} value={printer.printer_id}>{printer.name}{printer.is_default ? ' (Windows default)' : ''}</option>)}
                  </select>
                </label>
                <label>
                  <span className="form-label">Copies</span>
                  <input className="input" min={1} max={20} type="number" value={copies} onChange={(e) => setCopies(Number(e.target.value))} />
                </label>
                <button className="btn btn-primary self-end" disabled={printing || !selectedPrinterId} onClick={() => void printPdf()} type="button">
                  <Printer size={16} />{printing ? 'Printing...' : 'Print PDF'}
                </button>
              </div>}

              {!isTrashView && moving && <div className="grid gap-3 rounded border border-border bg-background p-4 md:grid-cols-[1fr_1fr_auto]">
                <label>
                  <span className="form-label">Move to category</span>
                  <select className="input" value={moveCategoryId} onChange={(e) => void loadMoveFolders(e.target.value).catch((err) => setMessage(String(err)))}>
                    <option value="">Select category</option>
                    {categories.map((category) => <option key={category.category_id} value={category.category_id}>{category.category_name}</option>)}
                  </select>
                </label>
                <label>
                  <span className="form-label">Move to folder</span>
                  <select className="input" value={moveFolderId} onChange={(e) => setMoveFolderId(e.target.value)}>
                    <option value="">Category root</option>
                    {moveFolders.map((folder) => <option key={folder.folder_id} value={folder.folder_id}>{folder.folder_name}</option>)}
                  </select>
                </label>
                <button className="btn btn-primary self-end" onClick={() => void saveMove().catch((err) => setMessage(String(err)))} type="button"><MoveRight size={16} />Save Move</button>
              </div>}

              {!isTrashView && <div className="grid gap-3 rounded border border-border bg-background p-4 md:grid-cols-[1fr_auto]">
                <label>
                  <span className="form-label">Change status</span>
                  <select className="input" value={statusDraft} onChange={(e) => setStatusDraft(e.target.value as DocumentStatus)}>
                    <option>Filed</option><option>Archived</option><option>Confidential</option><option>Other</option>
                  </select>
                </label>
                <button className="btn btn-primary self-end" onClick={() => void saveStatus().catch((err) => setMessage(String(err)))} type="button"><Save size={16} />Save Status</button>
              </div>}

              <div className="border-t border-border pt-4">
                <div className="mb-3 flex items-center gap-2"><Paperclip size={17} className="text-primary" /><h3 className="font-semibold text-secondary">Attachments</h3></div>
                <div className="space-y-2">
                  {detail.attachments.map((file) => (
                    <div className="flex items-center justify-between rounded border border-border p-3 text-sm" key={file.attachment_id}>
                      <div><p className="font-medium text-secondary">{file.original_file_name}</p><p className="text-xs text-muted">{Math.ceil(file.file_size_bytes / 1024)} KB</p></div>
                      <div className="flex gap-2">
                        <button className="icon-btn" title="Preview attachment" onClick={() => setPreviewAttachmentId(file.attachment_id)} type="button"><Eye size={15} /></button>
                        <button className="icon-btn" title="Show file path" onClick={() => void showPath(file.attachment_id).catch((err) => setMessage(String(err)))} type="button"><ExternalLink size={15} /></button>
                        {!isTrashView && <button className="icon-btn" title="Remove attachment" onClick={() => void remove(file.attachment_id).catch((err) => setMessage(String(err)))} type="button"><Trash2 size={15} /></button>}
                      </div>
                    </div>
                  ))}
                </div>
                {!isTrashView && <div className="mt-3 space-y-3">
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
                </div>}
                <div className="mt-4">
                  <AttachmentPreview
                    attachment={detail.attachments.find((file) => file.attachment_id === previewAttachmentId) ?? detail.attachments[0] ?? null}
                    onError={(error) => setMessage(error)}
                    sessionId={sessionId}
                  />
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </section>
  );
};

const safeFileName = (value: string) => value.replace(/[<>:"/\\|?*]+/g, '-').slice(0, 80) || 'document';

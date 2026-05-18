import { save } from '@tauri-apps/plugin-dialog';
import { Download, FileText, Folder, Printer, Search, X } from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';

import { AttachmentPreview } from '../components/AttachmentPreview';
import { EmptyState } from '../components/EmptyState';
import { formatDateOnly } from '../lib/dates';
import {
  exportDocumentPdf,
  getPublicDocument,
  listPrintPrinters,
  listPublicCategories,
  listPublicDocuments,
  listPublicFolders,
  printDocumentPdf
} from '../lib/invoke';
import { getUserErrorMessage } from '../lib/errors';
import type { CategoryItem, DocumentDetail, DocumentItem, FolderItem, PrinterDevice } from '../types';

export const GuestLanding = () => {
  const [categories, setCategories] = useState<CategoryItem[]>([]);
  const [folders, setFolders] = useState<FolderItem[]>([]);
  const [documents, setDocuments] = useState<DocumentItem[]>([]);
  const [printers, setPrinters] = useState<PrinterDevice[]>([]);
  const [selectedPrinterId, setSelectedPrinterId] = useState('');
  const [copies, setCopies] = useState(1);
  const [detail, setDetail] = useState<DocumentDetail | null>(null);
  const [search, setSearch] = useState('');
  const [categoryId, setCategoryId] = useState('');
  const [folderId, setFolderId] = useState('');
  const [message, setMessage] = useState('');
  const [exporting, setExporting] = useState(false);
  const [printing, setPrinting] = useState(false);
  const [previewAttachmentId, setPreviewAttachmentId] = useState<number | null>(null);
  const isGlobalSearch = search.trim().length > 0;

  const load = async () => {
    setCategories(await listPublicCategories());
    try {
      const nextPrinters = normalizePrinters(await listPrintPrinters(null));
      setPrinters(nextPrinters);
      setSelectedPrinterId((current) => (
        nextPrinters.some((printer) => printer.printer_id === current)
          ? current
          : nextPrinters.find((printer) => printer.is_default)?.printer_id || nextPrinters[0]?.printer_id || ''
      ));
    } catch {
      setPrinters([]);
      setSelectedPrinterId('');
      setMessage('Printers are not available right now. You can still view and export public documents.');
    }
    const rows = await listPublicDocuments({
      search: search || null,
      categoryId: isGlobalSearch ? null : categoryId ? Number(categoryId) : null,
      folderId: isGlobalSearch ? null : folderId ? Number(folderId) : null
    });
    setDocuments(rows);
    if (!detail && rows[0]) setDetail(await getPublicDocument(rows[0].document_id));
    if (detail && !rows.some((row) => row.document_id === detail.document.document_id)) setDetail(null);
  };

  useEffect(() => {
    void load().catch((err) => setMessage(getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.')));
  }, []);

  const selectCategory = async (id: number) => {
    setCategoryId(String(id));
    setFolderId('');
    setFolders(await listPublicFolders(id));
    setDocuments(await listPublicDocuments({ search: search || null, categoryId: id }));
    setDetail(null);
  };

  const submitSearch = (event: FormEvent) => {
    event.preventDefault();
    setDetail(null);
    void load().catch((err) => setMessage(getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.')));
  };

  const clearSearch = () => {
    setSearch('');
    setDetail(null);
    void listPublicDocuments({
      categoryId: categoryId ? Number(categoryId) : null,
      folderId: folderId ? Number(folderId) : null
    }).then(async (rows) => {
      setDocuments(rows);
      if (rows[0]) setDetail(await getPublicDocument(rows[0].document_id));
    }).catch((err) => setMessage(getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.')));
  };

  const openDocument = async (documentId: number) => {
    setDetail(await getPublicDocument(documentId));
  };

  const exportPdf = async () => {
    if (!detail || exporting) return;
    setExporting(true);
    setMessage('');
    try {
      const outputPath = await save({
        defaultPath: `${safeFileName(detail.document.document_name)}.pdf`,
        filters: [{ name: 'PDF', extensions: ['pdf'] }]
      });
      if (!outputPath) return;
      const savedPath = await exportDocumentPdf({
        documentId: detail.document.document_id,
        outputPath,
        sessionId: null
      });
      setMessage(`Exported PDF: ${savedPath}`);
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not export the PDF. Choose another save location and try again.'));
    } finally {
      setExporting(false);
    }
  };

  const printPdf = async () => {
    if (!detail || !selectedPrinterId || printing) return;
    setPrinting(true);
    setMessage('');
    try {
      const result = await printDocumentPdf({
        sessionId: null,
        documentId: detail.document.document_id,
        printerId: selectedPrinterId,
        copies
      });
      setMessage(`Print submitted to ${result.printer_name}.`);
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not print the document. Check the selected printer and try again.'));
    } finally {
      setPrinting(false);
    }
  };

  return (
    <section className="space-y-5">
      <div className="rounded border border-border bg-surface p-5 shadow-sm">
        <div className="flex items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold text-secondary">Browse Documents</h1>
            <p className="mt-1 text-sm text-muted">Read-only Staff/Head Viewer. Hidden, confidential, and trashed records are excluded.</p>
          </div>
          <form className="flex min-w-[360px] items-end gap-2" onSubmit={submitSearch}>
            <label className="flex-1">
              <span className="form-label">Search all public documents</span>
              <input className="input" value={search} onChange={(e) => setSearch(e.target.value)} />
            </label>
            <button className="btn btn-primary" type="submit"><Search size={16} />Search</button>
            {search && <button className="btn" onClick={clearSearch} type="button"><X size={16} />Clear</button>}
          </form>
        </div>
        {isGlobalSearch && (
          <p className="mt-3 rounded border border-border bg-background px-3 py-2 text-sm text-muted">
            Searching all public documents. Category and folder filters are ignored while this search is active.
          </p>
        )}
      </div>

      {message && <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div>}

      <div className="grid gap-5 xl:grid-cols-[280px_0.9fr_1.1fr]">
        <aside className="space-y-4">
          <div className="rounded border border-border bg-surface p-4 shadow-sm">
            <h2 className="mb-3 font-semibold text-secondary">Categories</h2>
            <div className="space-y-2">
              {categories.map((category) => (
                <button
                  className={`flex w-full items-center justify-between rounded border px-3 py-2 text-left text-sm ${categoryId === String(category.category_id) ? 'border-primary bg-red-50 text-secondary' : 'border-border bg-white text-muted hover:bg-background'}`}
                  key={category.category_id}
                  onClick={() => void selectCategory(category.category_id).catch((err) => setMessage(getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.')))}
                  type="button"
                >
                  <span className="flex items-center gap-2"><Folder size={15} />{category.category_name}</span>
                  <span>{category.document_count}</span>
                </button>
              ))}
            </div>
          </div>
          <div className="rounded border border-border bg-surface p-4 shadow-sm">
            <h2 className="mb-3 font-semibold text-secondary">Folders</h2>
            <button className="mb-2 w-full rounded border border-border px-3 py-2 text-left text-sm text-muted hover:bg-background" onClick={() => { setFolderId(''); void load(); }} type="button">Category root</button>
            {folders.map((folder) => (
              <button
                className={`mb-2 w-full rounded border px-3 py-2 text-left text-sm ${folderId === String(folder.folder_id) ? 'border-primary bg-red-50 text-secondary' : 'border-border text-muted hover:bg-background'}`}
                key={folder.folder_id}
                onClick={() => {
                  setFolderId(String(folder.folder_id));
                  void listPublicDocuments({ search: search || null, categoryId: Number(categoryId), folderId: folder.folder_id }).then(setDocuments).catch((err) => setMessage(getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.')));
                }}
                type="button"
              >
                {folder.folder_name}
              </button>
            ))}
          </div>
        </aside>

        <div className="overflow-hidden rounded border border-border bg-surface shadow-sm">
          <table className="w-full text-left text-sm">
            <thead className="border-b border-border bg-background text-xs uppercase text-muted">
              <tr><th className="p-3">Document</th><th className="p-3">Filed</th></tr>
            </thead>
            <tbody>
              {documents.map((doc) => (
                <tr className="cursor-pointer border-b border-border hover:bg-background" key={doc.document_id} onClick={() => void openDocument(doc.document_id).catch((err) => setMessage(getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.')))}>
                  <td className="p-3">
                    <p className="font-semibold text-secondary">{doc.document_name}</p>
                    <p className="text-xs text-muted"><span className="rounded bg-background px-2 py-0.5 text-[11px] font-semibold text-secondary">{doc.status}</span> · {doc.category_name}{doc.folder_name ? ` / ${doc.folder_name}` : ''}</p>
                  </td>
                  <td className="p-3 text-muted">{formatDateOnly(doc.date_received)}</td>
                </tr>
              ))}
              {documents.length === 0 && (
                <tr>
                  <td className="p-4" colSpan={2}>
                    <EmptyState
                      message="Try another search term, choose a different category, or return to the category root."
                      title="No public documents found"
                    />
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>

        <div className="rounded border border-border bg-surface p-5 shadow-sm">
          {!detail ? (
            <EmptyState
              message="Choose a document from the list to view details, attachments, export, and print actions."
              title="No document selected"
            />
          ) : (
            <div>
              <div className="mb-4 flex items-start gap-3">
                <FileText className="mt-1 text-primary" size={24} />
                <div className="min-w-0 flex-1">
                  <h2 className="text-xl font-bold text-secondary">{detail.document.document_name}</h2>
                  <p className="text-sm text-muted">{detail.document.category_name}{detail.document.folder_name ? ` / ${detail.document.folder_name}` : ''}</p>
                  <p className="mt-2 inline-flex rounded bg-background px-2 py-1 text-xs font-semibold text-secondary">{detail.document.status}</p>
                </div>
                <button className="btn btn-primary" disabled={exporting} onClick={() => void exportPdf()} type="button">
                  <Download size={16} />
                  {exporting ? 'Exporting...' : 'Export PDF'}
                </button>
              </div>
              <dl className="grid gap-3 text-sm md:grid-cols-2">
                <div><dt className="text-muted">Date received</dt><dd className="font-medium text-secondary">{formatDateOnly(detail.document.date_received)}</dd></div>
                <div><dt className="text-muted">Sender office</dt><dd className="font-medium text-secondary">{detail.document.office_name ?? 'Not specified'}</dd></div>
                <div className="md:col-span-2"><dt className="text-muted">Remarks</dt><dd className="font-medium text-secondary">{detail.document.remarks ?? 'No remarks'}</dd></div>
              </dl>
              <div className="mt-4 grid gap-3 rounded border border-border bg-background p-4 md:grid-cols-[1fr_100px_auto]">
                <label>
                  <span className="form-label">Printer</span>
                  <select aria-label="Printer" className="input" disabled={printers.length === 0} value={selectedPrinterId} onChange={(e) => setSelectedPrinterId(e.target.value)}>
                    <option value="">Select printer</option>
                    {printers.map((printer) => <option key={printer.printer_id} value={printer.printer_id}>{printer.name}{printer.is_default ? ' (Windows default)' : ''}</option>)}
                  </select>
                  {printers.length === 0 && (
                    <p className="mt-1 text-xs text-muted">Printers are not available right now. You can still view and export public documents.</p>
                  )}
                </label>
                <label>
                  <span className="form-label">Copies</span>
                  <input className="input" min={1} max={20} type="number" value={copies} onChange={(e) => setCopies(Number(e.target.value))} />
                </label>
                <button className="btn btn-primary self-end" disabled={printing || printers.length === 0 || !selectedPrinterId} onClick={() => void printPdf()} type="button">
                  <Printer size={16} />{printing ? 'Printing...' : 'Print PDF'}
                </button>
              </div>
              <div className="mt-5 border-t border-border pt-4">
                <h3 className="mb-3 font-semibold text-secondary">Attachments</h3>
                <div className="space-y-2">
                  {detail.attachments.map((file) => (
                    <div className="flex items-center justify-between gap-2 rounded border border-border p-3 text-sm" key={file.attachment_id}>
                      <button className="min-w-0 flex-1 truncate text-left font-medium text-secondary hover:text-primary" onClick={() => setPreviewAttachmentId(file.attachment_id)} type="button">{file.original_file_name}</button>
                    </div>
                  ))}
                </div>
                <div className="mt-4">
                  <AttachmentPreview
                    attachment={detail.attachments.find((file) => file.attachment_id === previewAttachmentId) ?? detail.attachments[0] ?? null}
                    onError={(error) => setMessage(error)}
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

const normalizePrinters = (value: unknown): PrinterDevice[] => (Array.isArray(value) ? value : []);

import { open, save } from '@tauri-apps/plugin-dialog';
import {
  Download,
  Edit3,
  Eye,
  EyeOff,
  MoveRight,
  Paperclip,
  Printer,
  RefreshCw,
  RotateCcw,
  Save,
  Search,
  Trash2,
  X,
} from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import type { FormEvent, KeyboardEvent } from 'react';

import { AttachmentPreview } from '../../components/AttachmentPreview';
import { ConfirmDialog, type ConfirmAction } from '../../components/ConfirmDialog';
import { EmptyState } from '../../components/EmptyState';
import { formatDateOnly } from '../../lib/dates';
import { invoke } from '@tauri-apps/api/core';
import { getUserErrorMessage } from '../../lib/errors';
import { fileNameFromPath, normalizeSelectedPaths, safeFileName } from '../../lib/helpers';
import { useSessionStore } from '../../store/sessionStore';
import type {
  CategoryItem,
  DocumentDetail,
  DocumentItem,
  DocumentListPage,
  DocumentStatus,
  FolderItem,
  OfficeItem,
  PrinterDevice,
} from '../../types';

const attachmentFilters = [
  {
    name: 'Allowed document and image files',
    extensions: ['pdf', 'doc', 'docx', 'xls', 'xlsx', 'jpg', 'jpeg', 'png', 'tif', 'tiff', 'txt'],
  },
];

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
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null);
  const clearConfirmAction = () => setConfirmAction(null);
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [totalCount, setTotalCount] = useState(0);
  const [offset, setOffset] = useState(0);
  const PAGE_SIZE = 50;
  const searchRef = useRef<HTMLInputElement>(null);

  const loadLookups = async () => {
    if (!sessionId) return;
    const [nextCategories, nextOffices, nextPrinters] = await Promise.all([
      invoke<CategoryItem[]>('list_public_categories'),
      invoke<OfficeItem[]>('list_document_offices', { sessionId }),
      invoke<PrinterDevice[]>('list_print_printers', { sessionId }),
    ]);
    setCategories(nextCategories);
    setOffices(nextOffices);
    setPrinters(nextPrinters);
    setSelectedPrinterId(
      (current) =>
        current ||
        nextPrinters.find((printer) => printer.is_default)?.printer_id ||
        nextPrinters[0]?.printer_id ||
        '',
    );
  };

  const loadFolders = async (nextCategoryId: string) => {
    setFolderId('');
    if (!nextCategoryId) {
      setFolders([]);
      return;
    }
    setFolders(
      await invoke<FolderItem[]>('list_public_folders', { categoryId: Number(nextCategoryId) }),
    );
  };

  const loadDocuments = async (resetOffset = true) => {
    if (!sessionId) return;
    const nextOffset = resetOffset ? 0 : offset;
    let rows: DocumentItem[];
    let total: number;
    if (view === 'trash') {
      rows = await invoke<DocumentItem[]>('list_trash_documents', { sessionId });
      total = rows.length;
    } else {
      const page = await invoke<DocumentListPage>('list_documents', {
        sessionId,
        search: search || null,
        categoryId: categoryId ? Number(categoryId) : null,
        folderId: folderId ? Number(folderId) : null,
        officeId: officeId ? Number(officeId) : null,
        dateFrom: dateFrom || null,
        dateTo: dateTo || null,
        status: statusFilter || null,
        limit: PAGE_SIZE,
        offset: nextOffset,
      });
      rows = resetOffset ? page.documents : [...documents, ...page.documents];
      total = page.total_count;
      setOffset(nextOffset + PAGE_SIZE);
    }
    setDocuments(rows);
    setTotalCount(total);
    if (detail && !rows.some((row) => row.document_id === detail.document.document_id)) {
      setDetail(null);
    }
    if (!detail && rows[0]) {
      await openDetail(rows[0].document_id);
    }
  };

  const openDetail = async (documentId: number) => {
    if (!sessionId) return;
    const nextDetail = await invoke<DocumentDetail>('get_document', { sessionId, documentId });
    setDetail(nextDetail);
    setStatusDraft(nextDetail.document.status);
    setMoveCategoryId(String(nextDetail.document.category_id));
    setMoveFolderId(nextDetail.document.folder_id ? String(nextDetail.document.folder_id) : '');
    setMoveFolders(
      await invoke<FolderItem[]>('list_public_folders', {
        categoryId: nextDetail.document.category_id,
      }),
    );
    setEditing(false);
    setMoving(false);
    setPreviewAttachmentId(nextDetail.attachments[0]?.attachment_id ?? null);
  };

  // ponytail: KeyboardEvent on td instead of tr due to checkbox column
  const openDetailFromKeyboard = (
    event: KeyboardEvent<HTMLTableDataCellElement>,
    documentId: number,
  ) => {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    void openDetail(documentId);
  };

  useEffect(() => {
    void loadLookups().catch((err) =>
      setMessage(
        getUserErrorMessage(err, 'Could not load document lists. Please refresh and try again.'),
      ),
    );
  }, [sessionId]);

  useEffect(() => {
    document.title = detail
      ? `${detail.document.document_name} — Documents`
      : 'Documents — OVPSPEE Filing System';
  }, [detail]);

  useEffect(() => {
    setSelectedIds(new Set());
    void loadDocuments().catch((err) =>
      setMessage(
        getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.'),
      ),
    );
  }, [sessionId, view]);

  // ponytail: debounced auto-search on filter changes; Apply button still works for explicit submit
  useEffect(() => {
    if (!sessionId) return;
    const timer = setTimeout(() => {
      void loadDocuments().catch((err) =>
        setMessage(
          getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.'),
        ),
      );
    }, 300);
    return () => clearTimeout(timer);
  }, [search, categoryId, folderId, officeId, statusFilter, dateFrom, dateTo]);

  // ponytail: Escape closes detail panel
  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      if (e.key === 'Escape' && detail) setDetail(null);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [detail]);

  // ponytail: / key focuses search input
  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      if (
        e.key === '/' &&
        !['INPUT', 'TEXTAREA', 'SELECT'].includes((e.target as HTMLElement)?.tagName)
      ) {
        e.preventDefault();
        searchRef.current?.focus();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  const submitSearch = (event: FormEvent) => {
    event.preventDefault();
    void loadDocuments().catch((err) =>
      setMessage(
        getUserErrorMessage(err, 'Could not load documents. Please refresh and try again.'),
      ),
    );
  };

  const saveEdit = async () => {
    if (!sessionId || !detail) return;
    await invoke<void>('update_document', {
      sessionId,
      documentId: detail.document.document_id,
      documentName: detail.document.document_name,
      categoryId: detail.document.category_id,
      folderId: detail.document.folder_id,
      officeId: detail.document.office_id,
      dateReceived: detail.document.date_received,
      remarks: detail.document.remarks,
      status: detail.document.status,
    });
    setMessage('Document updated.');
    await openDetail(detail.document.document_id);
    await loadDocuments();
  };

  const chooseAttachments = async () => {
    const selected = await open({
      multiple: true,
      directory: false,
      filters: attachmentFilters,
    });
    const paths = normalizeSelectedPaths(selected);
    if (paths.length) {
      setPendingAttachmentPaths((current) => Array.from(new Set([...current, ...paths])));
    }
  };

  const attach = async () => {
    if (!sessionId || !detail || pendingAttachmentPaths.length === 0) return;
    for (const [index, sourcePath] of pendingAttachmentPaths.entries()) {
      await invoke<number>('add_attachment', {
        sessionId,
        documentId: detail.document.document_id,
        sourcePath,
        sortOrder: detail.attachments.length + index + 1,
      });
    }
    setPendingAttachmentPaths([]);
    setMessage('Attachment added.');
    await openDetail(detail.document.document_id);
  };

  const remove = async (attachmentId: number) => {
    if (!sessionId || !detail) return;
    await invoke<void>('remove_attachment', { sessionId, attachmentId });
    setMessage('Attachment removed.');
    await openDetail(detail.document.document_id);
  };

  const toggleHidden = async () => {
    if (!sessionId || !detail) return;
    const nextHidden = !detail.document.is_hidden;
    await invoke<void>('set_document_hidden', {
      sessionId,
      documentId: detail.document.document_id,
      isHidden: nextHidden,
    });
    setMessage(nextHidden ? 'Document hidden.' : 'Document unhidden.');
    await openDetail(detail.document.document_id);
    await loadDocuments();
  };

  const moveToTrash = async () => {
    if (!sessionId || !detail) return;
    await invoke<void>('trash_document', { sessionId, documentId: detail.document.document_id });
    setMessage('Document moved to trash.');
    setDetail(null);
    await loadDocuments();
  };

  const restore = async () => {
    if (!sessionId || !detail) return;
    await invoke<void>('restore_document', { sessionId, documentId: detail.document.document_id });
    setMessage('Document restored.');
    setDetail(null);
    await loadDocuments();
  };

  const loadMoveFolders = async (nextCategoryId: string) => {
    setMoveCategoryId(nextCategoryId);
    setMoveFolderId('');
    setMoveFolders(
      nextCategoryId
        ? await invoke<FolderItem[]>('list_public_folders', { categoryId: Number(nextCategoryId) })
        : [],
    );
  };

  const saveMove = async () => {
    if (!sessionId || !detail || !moveCategoryId) return;
    await invoke<void>('move_document', {
      sessionId,
      documentId: detail.document.document_id,
      categoryId: Number(moveCategoryId),
      folderId: moveFolderId ? Number(moveFolderId) : null,
    });
    setMessage('Document moved.');
    await openDetail(detail.document.document_id);
    await loadDocuments();
  };

  const saveStatus = async () => {
    if (!sessionId || !detail) return;
    await invoke<void>('set_document_status', {
      sessionId,
      documentId: detail.document.document_id,
      status: statusDraft,
    });
    setMessage(
      statusDraft === 'Confidential'
        ? 'Status changed. Document is now hidden.'
        : 'Status changed.',
    );
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
        filters: [{ name: 'PDF', extensions: ['pdf'] }],
      });
      if (!outputPath) return;
      const savedPath = await invoke<string>('export_document_pdf', {
        sessionId,
        documentId: detail.document.document_id,
        outputPath,
      });
      setMessage(`Exported PDF: ${savedPath}`);
    } catch (err) {
      setMessage(
        getUserErrorMessage(
          err,
          'Could not export the PDF. Choose another save location and try again.',
        ),
      );
    } finally {
      setExporting(false);
    }
  };

  const printPdf = async () => {
    if (!sessionId || !detail || !selectedPrinterId || printing) return;
    setPrinting(true);
    setMessage('');
    try {
      const result = await invoke<{ printer_name: string }>('print_document_pdf', {
        sessionId,
        documentId: detail.document.document_id,
        printerId: selectedPrinterId,
        copies,
      });
      setMessage(`Print submitted to ${result.printer_name}.`);
    } catch (err) {
      setMessage(
        getUserErrorMessage(
          err,
          'Could not print the document. Check the selected printer and try again.',
        ),
      );
    } finally {
      setPrinting(false);
    }
  };

  const confirmMoveToTrash = () => {
    if (!detail) return;
    setConfirmAction({
      title: 'Move document to Trash?',
      body: (
        <>
          Move <strong>{detail.document.document_name}</strong> to Trash. It can be restored from
          Trash later.
        </>
      ),
      confirmLabel: 'Move to Trash',
      onConfirm: moveToTrash,
    });
  };

  const bulkTrash = () => {
    setConfirmAction({
      title: 'Trash documents',
      body: (
        <>
          Move <strong>{selectedIds.size}</strong> document(s) to Trash?
        </>
      ),
      confirmLabel: 'Trash',
      onConfirm: async () => {
        await Promise.all(
          [...selectedIds].map((id) =>
            invoke<void>('trash_document', { sessionId, documentId: id }),
          ),
        );
        setSelectedIds(new Set());
        await loadDocuments();
      },
    });
  };

  const confirmRemoveAttachment = (attachmentId: number, fileName: string) => {
    setConfirmAction({
      title: 'Remove attachment?',
      body: (
        <>
          Remove <strong>{fileName}</strong>. The document record will remain.
        </>
      ),
      confirmLabel: 'Remove Attachment',
      onConfirm: () => remove(attachmentId),
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

  const isTrashView = view === 'trash';

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
          <h1 className="text-2xl font-bold text-secondary">Documents</h1>
          <p className="mt-1 text-sm text-muted">Secretary filing list and document detail.</p>
        </div>
        <button className="btn" onClick={() => void loadDocuments(true)} type="button">
          <RefreshCw size={16} />
          Refresh
        </button>
        <span className="text-xs text-muted">
          {isTrashView ? documents.length : totalCount} document
          {(isTrashView ? documents.length : totalCount) !== 1 ? 's' : ''}
        </span>
      </div>

      <div className="inline-flex rounded border border-border bg-surface p-1 text-sm shadow-sm">
        <button
          className={view === 'active' ? 'btn btn-primary' : 'btn'}
          onClick={() => {
            setView('active');
            setDetail(null);
          }}
          type="button"
        >
          Documents
        </button>
        <button
          className={view === 'trash' ? 'btn btn-primary' : 'btn'}
          onClick={() => {
            setView('trash');
            setDetail(null);
          }}
          type="button"
        >
          Trash
        </button>
      </div>

      {!isTrashView && (
        <div className="flex flex-wrap gap-3 rounded border border-border bg-surface p-3 text-xs text-muted">
          {['Filed', 'Archived', 'Confidential', 'Other'].map((s) => (
            <span key={s} className="rounded bg-background px-2 py-1">
              {s}: {documents.filter((d) => d.status === s).length}
            </span>
          ))}
        </div>
      )}
      {selectedIds.size > 0 && (
        <div className="flex items-center gap-3 rounded border border-border bg-surface p-3 text-sm">
          <span className="font-semibold text-secondary">{selectedIds.size} selected</span>
          <button className="btn" onClick={bulkTrash} type="button">
            <Trash2 size={16} />
            Trash selected
          </button>
          <button className="btn" onClick={() => setSelectedIds(new Set())} type="button">
            <X size={16} />
            Clear
          </button>
        </div>
      )}
      {!isTrashView && (
        <form
          className="grid gap-3 rounded border border-border bg-surface p-4 shadow-sm md:grid-cols-[1fr_160px_160px_160px_150px_180px_180px_auto]"
          onSubmit={submitSearch}
        >
          <label>
            <span className="form-label">Search</span>
            <input
              ref={searchRef}
              className="input"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </label>
          <label>
            <span className="form-label">Category</span>
            <select
              className="input"
              value={categoryId}
              onChange={(e) => {
                setCategoryId(e.target.value);
                void loadFolders(e.target.value);
              }}
            >
              <option value="">All</option>
              {categories.map((category) => (
                <option key={category.category_id} value={category.category_id}>
                  {category.category_name}
                </option>
              ))}
            </select>
          </label>
          <label>
            <span className="form-label">Folder</span>
            <select
              className="input"
              value={folderId}
              onChange={(e) => setFolderId(e.target.value)}
            >
              <option value="">All</option>
              {folders.map((folder) => (
                <option key={folder.folder_id} value={folder.folder_id}>
                  {folder.folder_name}
                </option>
              ))}
            </select>
          </label>
          <label>
            <span className="form-label">Office</span>
            <select
              className="input"
              value={officeId}
              onChange={(e) => setOfficeId(e.target.value)}
            >
              <option value="">All</option>
              {offices.map((office) => (
                <option key={office.office_id} value={office.office_id}>
                  {office.office_name}
                </option>
              ))}
            </select>
          </label>
          <label>
            <span className="form-label">Status</span>
            <select
              className="input"
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value)}
            >
              <option value="">All</option>
              <option>Filed</option>
              <option>Archived</option>
              <option>Confidential</option>
              <option>Other</option>
            </select>
          </label>
          <label>
            <span className="form-label">From</span>
            <input
              className="input"
              type="date"
              value={dateFrom}
              onChange={(e) => setDateFrom(e.target.value)}
            />
          </label>
          <label>
            <span className="form-label">To</span>
            <input
              className="input"
              type="date"
              value={dateTo}
              onChange={(e) => setDateTo(e.target.value)}
            />
          </label>
          <button className="btn btn-primary self-end" type="submit">
            <Search size={16} />
            Apply
          </button>
        </form>
      )}

      {message && (
        <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">
          {message}
        </div>
      )}

      <div className="grid gap-5 xl:grid-cols-[0.9fr_1.1fr]">
        <div className="overflow-hidden rounded border border-border bg-surface shadow-sm">
          <table className="w-full text-left text-sm">
            <thead className="border-b border-border bg-background text-xs uppercase text-muted">
              <tr>
                <th className="w-10 p-3">
                  <input
                    aria-label="Select all documents"
                    checked={documents.length > 0 && selectedIds.size === documents.length}
                    className="checkbox"
                    onChange={(e) =>
                      setSelectedIds(
                        e.target.checked ? new Set(documents.map((d) => d.document_id)) : new Set(),
                      )
                    }
                    type="checkbox"
                  />
                </th>
                <th className="p-3">Document</th>
                <th className="p-3">Location</th>
                <th className="p-3">Date</th>
              </tr>
            </thead>
            <tbody>
              {documents.length === 0 ? (
                <tr>
                  <td className="p-4" colSpan={4}>
                    <EmptyState
                      message={
                        view === 'trash'
                          ? 'No documents in trash.'
                          : 'Try adjusting your search or filter criteria.'
                      }
                      title={view === 'trash' ? 'Trash is empty' : 'No documents found'}
                    />
                  </td>
                </tr>
              ) : (
                documents.map((doc) => (
                  <tr
                    aria-label={`Open document ${doc.document_name}`}
                    className="border-b border-border hover:bg-background focus-visible:outline focus-visible:outline-2 focus-visible:outline-primary"
                    key={doc.document_id}
                  >
                    <td className="w-10 p-3">
                      <input
                        aria-label={`Select ${doc.document_name}`}
                        checked={selectedIds.has(doc.document_id)}
                        className="checkbox"
                        onChange={(e) => {
                          const next = new Set(selectedIds);
                          if (e.target.checked) next.add(doc.document_id);
                          else next.delete(doc.document_id);
                          setSelectedIds(next);
                        }}
                        type="checkbox"
                      />
                    </td>
                    <td
                      className="cursor-pointer p-3"
                      onClick={() => void openDetail(doc.document_id)}
                      onKeyDown={(event) => openDetailFromKeyboard(event, doc.document_id)}
                      role="button"
                      tabIndex={0}
                    >
                      <p className="font-semibold text-secondary">{doc.document_name}</p>
                      <p className="text-xs text-muted">
                        <span className="rounded bg-background px-2 py-0.5 text-[11px] font-semibold text-secondary">
                          {doc.status}
                        </span>{' '}
                        · {doc.attachment_count} file(s){doc.is_hidden ? ' · Hidden' : ''}
                        {doc.is_trashed ? ' · Trashed' : ''}
                      </p>
                    </td>
                    <td className="p-3 text-muted">
                      {doc.category_name}
                      {doc.folder_name ? ` / ${doc.folder_name}` : ''}
                    </td>
                    <td className="p-3 text-muted">{formatDateOnly(doc.date_received)}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
          {!isTrashView && documents.length < totalCount && (
            <div className="border-t border-border p-3 text-center">
              <button
                className="btn btn-primary"
                onClick={() => void loadDocuments(false)}
                type="button"
              >
                Load More ({totalCount - documents.length} remaining)
              </button>
            </div>
          )}
        </div>

        <div className="rounded border border-border bg-surface p-5 shadow-sm">
          {!detail ? (
            <p className="text-sm text-muted">Select a document.</p>
          ) : (
            <div className="space-y-4">
              <div className="flex items-start justify-between gap-3">
                <button
                  aria-label="Close detail"
                  className="icon-btn shrink-0"
                  onClick={() => setDetail(null)}
                  title="Close (Esc)"
                  type="button"
                >
                  <X size={16} />
                </button>
                <div>
                  {editing ? (
                    <input
                      className="input text-lg font-semibold"
                      value={detail.document.document_name}
                      onChange={(e) =>
                        setDetail({
                          ...detail,
                          document: { ...detail.document, document_name: e.target.value },
                        })
                      }
                    />
                  ) : (
                    <h2 className="text-xl font-bold text-secondary">
                      {detail.document.document_name}
                    </h2>
                  )}
                  <p className="mt-1 text-sm text-muted">
                    {detail.document.category_name}
                    {detail.document.folder_name ? ` / ${detail.document.folder_name}` : ''}
                  </p>
                  <p className="mt-2 inline-flex rounded bg-background px-2 py-1 text-xs font-semibold text-secondary">
                    {detail.document.status}
                    {detail.document.is_hidden ? ' · Hidden' : ''}
                  </p>
                </div>
                <div className="flex flex-wrap justify-end gap-2">
                  {!isTrashView && (
                    <button className="btn" onClick={() => setEditing(!editing)} type="button">
                      <Edit3 size={16} />
                      Edit
                    </button>
                  )}
                  {!isTrashView && (
                    <button
                      className="btn btn-primary"
                      disabled={exporting}
                      onClick={() => void exportPdf()}
                      type="button"
                    >
                      <Download size={16} />
                      {exporting ? 'Exporting...' : 'Export PDF'}
                    </button>
                  )}
                  {!isTrashView && (
                    <button className="btn" onClick={() => setMoving(!moving)} type="button">
                      <MoveRight size={16} />
                      Move
                    </button>
                  )}
                  {!isTrashView && (
                    <button
                      className="btn"
                      onClick={() =>
                        void toggleHidden().catch((err) =>
                          setMessage(
                            getUserErrorMessage(
                              err,
                              'Could not save the document. Check the required fields and try again.',
                            ),
                          ),
                        )
                      }
                      type="button"
                    >
                      {detail.document.is_hidden ? <Eye size={16} /> : <EyeOff size={16} />}
                      {detail.document.is_hidden ? 'Unhide' : 'Hide'}
                    </button>
                  )}
                  {!isTrashView && (
                    <button className="btn" onClick={confirmMoveToTrash} type="button">
                      <Trash2 size={16} />
                      Move to Trash
                    </button>
                  )}
                  {isTrashView && (
                    <button
                      className="btn btn-primary"
                      onClick={() =>
                        void restore().catch((err) =>
                          setMessage(getUserErrorMessage(err, 'Could not restore the document.')),
                        )
                      }
                      type="button"
                    >
                      <RotateCcw size={16} />
                      Restore
                    </button>
                  )}
                </div>
              </div>

              <div className="grid gap-3 md:grid-cols-2">
                <label>
                  <span className="form-label">Date received</span>
                  <input
                    className="input"
                    disabled={!editing || isTrashView}
                    type="date"
                    value={detail.document.date_received}
                    onChange={(e) =>
                      setDetail({
                        ...detail,
                        document: { ...detail.document, date_received: e.target.value },
                      })
                    }
                  />
                </label>
                <label>
                  <span className="form-label">Status</span>
                  <input className="input" disabled value={detail.document.status} />
                </label>
                <label className="md:col-span-2">
                  <span className="form-label">Remarks</span>
                  <textarea
                    className="input min-h-24"
                    disabled={!editing || isTrashView}
                    value={detail.document.remarks ?? ''}
                    onChange={(e) =>
                      setDetail({
                        ...detail,
                        document: { ...detail.document, remarks: e.target.value },
                      })
                    }
                  />
                </label>
              </div>
              {editing && !isTrashView && (
                <button
                  className="btn btn-primary"
                  onClick={() =>
                    void saveEdit().catch((err) =>
                      setMessage(
                        getUserErrorMessage(
                          err,
                          'Could not save the document. Check the required fields and try again.',
                        ),
                      ),
                    )
                  }
                  type="button"
                >
                  <Save size={16} />
                  Save Changes
                </button>
              )}

              {!isTrashView && (
                <div className="grid gap-3 rounded border border-border bg-background p-4 md:grid-cols-[1fr_110px_auto]">
                  <label>
                    <span className="form-label">Printer</span>
                    <select
                      className="input"
                      value={selectedPrinterId}
                      onChange={(e) => setSelectedPrinterId(e.target.value)}
                    >
                      <option value="">Select printer</option>
                      {printers.map((printer) => (
                        <option key={printer.printer_id} value={printer.printer_id}>
                          {printer.name}
                          {printer.is_default ? ' (Windows default)' : ''}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label>
                    <span className="form-label">Copies</span>
                    <input
                      className="input"
                      min={1}
                      max={20}
                      type="number"
                      value={copies}
                      onChange={(e) => setCopies(Number(e.target.value))}
                    />
                  </label>
                  <button
                    className="btn btn-primary self-end"
                    disabled={printing || !selectedPrinterId}
                    onClick={() => void printPdf()}
                    type="button"
                  >
                    <Printer size={16} />
                    {printing ? 'Printing...' : 'Print PDF'}
                  </button>
                </div>
              )}

              {!isTrashView && moving && (
                <div className="grid gap-3 rounded border border-border bg-background p-4 md:grid-cols-[1fr_1fr_auto]">
                  <label>
                    <span className="form-label">Move to category</span>
                    <select
                      className="input"
                      value={moveCategoryId}
                      onChange={(e) =>
                        void loadMoveFolders(e.target.value).catch((err) =>
                          setMessage(
                            getUserErrorMessage(
                              err,
                              'Could not load documents. Please refresh and try again.',
                            ),
                          ),
                        )
                      }
                    >
                      <option value="">Select category</option>
                      {categories.map((category) => (
                        <option key={category.category_id} value={category.category_id}>
                          {category.category_name}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label>
                    <span className="form-label">Move to folder</span>
                    <select
                      className="input"
                      value={moveFolderId}
                      onChange={(e) => setMoveFolderId(e.target.value)}
                    >
                      <option value="">Category root</option>
                      {moveFolders.map((folder) => (
                        <option key={folder.folder_id} value={folder.folder_id}>
                          {folder.folder_name}
                        </option>
                      ))}
                    </select>
                  </label>
                  <button
                    className="btn btn-primary self-end"
                    onClick={() =>
                      void saveMove().catch((err) =>
                        setMessage(
                          getUserErrorMessage(
                            err,
                            'Could not save the document. Check the required fields and try again.',
                          ),
                        ),
                      )
                    }
                    type="button"
                  >
                    <MoveRight size={16} />
                    Save Move
                  </button>
                </div>
              )}

              {!isTrashView && (
                <div className="grid gap-3 rounded border border-border bg-background p-4 md:grid-cols-[1fr_auto]">
                  <label>
                    <span className="form-label">Change status</span>
                    <select
                      className="input"
                      value={statusDraft}
                      onChange={(e) => setStatusDraft(e.target.value as DocumentStatus)}
                    >
                      <option>Filed</option>
                      <option>Archived</option>
                      <option>Confidential</option>
                      <option>Other</option>
                    </select>
                  </label>
                  <button
                    className="btn btn-primary self-end"
                    onClick={() =>
                      void saveStatus().catch((err) =>
                        setMessage(
                          getUserErrorMessage(
                            err,
                            'Could not save the document. Check the required fields and try again.',
                          ),
                        ),
                      )
                    }
                    type="button"
                  >
                    <Save size={16} />
                    Save Status
                  </button>
                </div>
              )}

              <div className="border-t border-border pt-4">
                <div className="mb-3 flex items-center gap-2">
                  <Paperclip size={17} className="text-primary" />
                  <h3 className="font-semibold text-secondary">Attachments</h3>
                </div>
                <div className="space-y-2">
                  {detail.attachments.map((file) => (
                    <div
                      className="flex items-center justify-between rounded border border-border p-3 text-sm"
                      key={file.attachment_id}
                    >
                      <div>
                        <p className="font-medium text-secondary">{file.original_file_name}</p>
                        <p className="text-xs text-muted">
                          {Math.ceil(file.file_size_bytes / 1024)} KB
                        </p>
                      </div>
                      <div className="flex gap-2">
                        <button
                          aria-label={`Preview attachment ${file.original_file_name}`}
                          className="icon-btn"
                          title="Preview attachment"
                          onClick={() => setPreviewAttachmentId(file.attachment_id)}
                          type="button"
                        >
                          <Eye size={15} />
                        </button>
                        {!isTrashView && (
                          <button
                            aria-label={`Remove attachment ${file.original_file_name}`}
                            className="icon-btn"
                            title="Remove attachment"
                            onClick={() =>
                              confirmRemoveAttachment(file.attachment_id, file.original_file_name)
                            }
                            type="button"
                          >
                            <Trash2 size={15} />
                          </button>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
                {!isTrashView && (
                  <div className="mt-3 space-y-3">
                    <button
                      className="btn"
                      onClick={() =>
                        void chooseAttachments().catch((err) =>
                          setMessage(
                            getUserErrorMessage(
                              err,
                              'Could not add the attachment. Check the file type and try again.',
                            ),
                          ),
                        )
                      }
                      type="button"
                    >
                      <Paperclip size={16} />
                      Add Attachments
                    </button>
                    {pendingAttachmentPaths.length > 0 && (
                      <div className="space-y-2">
                        {pendingAttachmentPaths.map((sourcePath) => (
                          <div
                            className="flex items-center justify-between gap-3 rounded border border-border p-3 text-sm"
                            key={sourcePath}
                          >
                            <div className="min-w-0">
                              <p className="truncate font-medium text-secondary">
                                {fileNameFromPath(sourcePath)}
                              </p>
                              <p className="truncate text-xs text-muted">{sourcePath}</p>
                            </div>
                            <button
                              aria-label={`Remove selected file ${fileNameFromPath(sourcePath)}`}
                              className="icon-btn shrink-0"
                              onClick={() =>
                                setPendingAttachmentPaths((current) =>
                                  current.filter((path) => path !== sourcePath),
                                )
                              }
                              title="Remove selected file"
                              type="button"
                            >
                              <X size={15} />
                            </button>
                          </div>
                        ))}
                        <button
                          className="btn btn-primary"
                          onClick={() =>
                            void attach().catch((err) =>
                              setMessage(
                                getUserErrorMessage(
                                  err,
                                  'Could not add the attachment. Check the file type and try again.',
                                ),
                              ),
                            )
                          }
                          type="button"
                        >
                          Save Selected Attachments
                        </button>
                      </div>
                    )}
                  </div>
                )}
                <div className="mt-4">
                  <AttachmentPreview
                    attachment={
                      detail.attachments.find(
                        (file) => file.attachment_id === previewAttachmentId,
                      ) ??
                      detail.attachments[0] ??
                      null
                    }
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

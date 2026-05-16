import { open } from '@tauri-apps/plugin-dialog';
import { AlertTriangle, ChevronLeft, ChevronRight, Eye, FileScan, FileText, Link2, RefreshCw, Save, ScanLine, Trash2, Upload, X } from 'lucide-react';
import { FormEvent, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import {
  attachScanToDocument,
  fileScanAsDocument,
  importScanFiles,
  getScanIntakePreviewPage,
  getDeviceSettings,
  getScannerCapabilities,
  listDocumentOffices,
  listDocuments,
  listPublicCategories,
  listPublicFolders,
  listScanIntake,
  listScanners,
  removeScanIntake,
  scanToIntake,
  updateScanIntakeNotes
} from '../../lib/invoke';
import { useSessionStore } from '../../store/sessionStore';
import type {
  CategoryItem,
  DocumentItem,
  DocumentStatus,
  FolderItem,
  OfficeItem,
  ScanIntakeItem,
  ScanIntakePreviewPage,
  ScanOptions,
  ScannerCapabilities,
  ScannerDevice
} from '../../types';

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

interface ScanIntakePreviewProps {
  item: ScanIntakeItem | null;
  loading: boolean;
  onPageChange: (page: number) => void;
  page: number;
  preview: ScanIntakePreviewPage | null;
}

const ScanIntakePreview = ({ item, loading, onPageChange, page, preview }: ScanIntakePreviewProps) => {
  if (!item) {
    return (
      <div className="rounded border border-border bg-surface p-5 shadow-sm">
        <div className="mb-3 flex items-center gap-2">
          <Eye size={18} className="text-primary" />
          <h2 className="font-semibold text-secondary">Pending Preview</h2>
        </div>
        <p className="rounded border border-dashed border-border p-4 text-sm text-muted">Select a pending scan/import to preview before filing.</p>
      </div>
    );
  }

  const info = preview?.info;
  const maxPage = info?.page_count ?? 1;
  const canPage = info?.preview_kind === 'Pdf' && maxPage > 1;
  const kind = info?.preview_kind ?? 'Loading';

  return (
    <div className="space-y-3 rounded border border-border bg-surface p-5 shadow-sm">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <Eye size={18} className="text-primary" />
            <h2 className="font-semibold text-secondary">Pending Preview</h2>
          </div>
          <p className="mt-1 truncate text-sm font-medium text-secondary">{item.original_file_name}</p>
          <p className="text-xs text-muted">
            {kind} · {info?.extension ?? extensionFromName(item.original_file_name)} · {item.mime_type} · {sizeLabel(item.file_size_bytes)}
          </p>
        </div>
        <span className="rounded bg-background px-2 py-1 text-xs font-semibold text-secondary">{kind}</span>
      </div>

      {canPage && (
        <div className="flex items-center gap-2">
          <button className="icon-btn" disabled={loading || page <= 1} onClick={() => onPageChange(page - 1)} title="Previous page" type="button">
            <ChevronLeft size={15} />
          </button>
          <span className="text-xs font-semibold text-secondary">PAGE {page} of {maxPage}</span>
          <button className="icon-btn" disabled={loading || page >= maxPage} onClick={() => onPageChange(page + 1)} title="Next page" type="button">
            <ChevronRight size={15} />
          </button>
        </div>
      )}

      {loading && <div className="rounded border border-border bg-background p-4 text-sm text-muted">Loading preview...</div>}
      {!loading && info && !info.file_exists && (
        <div className="rounded border border-warning/30 bg-warning/10 p-4 text-sm text-warning">
          <div className="flex items-center gap-2 font-semibold"><AlertTriangle size={16} />File unavailable</div>
          <p className="mt-1">{info.message}</p>
        </div>
      )}
      {!loading && info?.file_exists && info.preview_kind === 'Image' && preview?.preview_data_url && (
        <div className="max-h-[26rem] overflow-auto rounded border border-border bg-white p-3">
          <img alt={item.original_file_name} className="mx-auto max-h-[24rem] max-w-full object-contain" src={preview.preview_data_url} />
        </div>
      )}
      {!loading && info?.file_exists && info.preview_kind === 'Pdf' && preview?.preview_data_url && (
        <iframe className="h-[28rem] w-full rounded border border-border bg-white" src={`${preview.preview_data_url}#page=${page}`} title={item.original_file_name} />
      )}
      {!loading && preview && info?.file_exists && info.preview_kind === 'Text' && (
        <div className="rounded border border-border bg-white">
          <div className="flex items-center gap-2 border-b border-border px-3 py-2 text-sm font-semibold text-secondary">
            <FileText size={16} />Text preview
          </div>
          {preview.text_content ? (
            <pre className="max-h-[24rem] overflow-auto whitespace-pre-wrap break-words p-3 text-xs leading-relaxed text-secondary">{preview.text_content}</pre>
          ) : (
            <div className="p-4 text-sm text-muted">{info.message}</div>
          )}
          {preview.text_truncated && <p className="border-t border-border px-3 py-2 text-xs text-muted">Preview capped for safety.</p>}
        </div>
      )}
      {!loading && info?.file_exists && info.preview_kind === 'Unsupported' && (
        <div className="rounded border border-border bg-background p-4 text-sm text-secondary">
          <div className="mb-2 flex items-center gap-2 font-semibold"><Eye size={16} />Preview not available for this file type</div>
          <p>{info.message}</p>
          <dl className="mt-3 grid gap-2 text-xs text-muted sm:grid-cols-3">
            <div><dt className="font-semibold text-secondary">Type</dt><dd>{info.extension.toUpperCase()}</dd></div>
            <div><dt className="font-semibold text-secondary">MIME</dt><dd>{info.mime_type}</dd></div>
            <div><dt className="font-semibold text-secondary">Size</dt><dd>{sizeLabel(info.file_size_bytes)}</dd></div>
          </dl>
        </div>
      )}
    </div>
  );
};

const fileNameFromPath = (path: string) => path.split(/[\\/]/).pop() ?? path;
const normalizeSelectedPaths = (selected: string | string[] | null) => {
  if (!selected) return [];
  return Array.isArray(selected) ? selected : [selected];
};
const sizeLabel = (bytes: number) => `${Math.ceil(bytes / 1024)} KB`;
const extensionFromName = (name: string) => name.split('.').pop()?.toLowerCase() ?? 'file';

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
  const [scanners, setScanners] = useState<ScannerDevice[]>([]);
  const [selectedScannerId, setSelectedScannerId] = useState('');
  const [scannerCapabilities, setScannerCapabilities] = useState<ScannerCapabilities | null>(null);
  const [scanOptions, setScanOptions] = useState<ScanOptions>({
    dpi: 300,
    color_mode: 'color',
    output_format: 'png',
    source: 'flatbed'
  });
  const [form, setForm] = useState(emptyForm);
  const [existingDocumentId, setExistingDocumentId] = useState('');
  const [notesDraft, setNotesDraft] = useState('');
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const [scanBusy, setScanBusy] = useState(false);
  const [preview, setPreview] = useState<ScanIntakePreviewPage | null>(null);
  const [previewPage, setPreviewPage] = useState(1);
  const [previewLoading, setPreviewLoading] = useState(false);

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

  const loadScanners = async () => {
    if (!sessionId) return;
    const [rows, settings] = await Promise.all([
      listScanners(sessionId),
      getDeviceSettings(sessionId)
    ]);
    setScanners(rows);
    const preferred = settings.default_scanner_id && rows.some((scanner) => scanner.device_id === settings.default_scanner_id)
      ? settings.default_scanner_id
      : rows[0]?.device_id ?? '';
    setSelectedScannerId(preferred);
    setScanOptions((current) => ({
      ...current,
      dpi: settings.scan_default_dpi,
      color_mode: settings.scan_default_color_mode,
      output_format: settings.scan_default_output_format === 'jpg' ? 'jpg' : 'png'
    }));
  };

  const loadIntake = async () => {
    if (!sessionId) return [];
    const rows = await listScanIntake(sessionId);
    setItems(rows);
    setSelectedIds((current) => current.filter((id) => rows.some((row) => row.scan_intake_id === id)));
    return rows;
  };

  useEffect(() => {
    void Promise.all([loadLookups(), loadIntake(), loadScanners()]).catch((err) => setMessage(String(err)));
  }, [sessionId]);

  useEffect(() => {
    if (!sessionId || !selectedScannerId) {
      setScannerCapabilities(null);
      return;
    }
    void getScannerCapabilities({ sessionId, scannerId: selectedScannerId })
      .then((capabilities) => {
        setScannerCapabilities(capabilities);
        setScanOptions((current) => ({
          dpi: capabilities.supported_dpi.includes(current.dpi) ? current.dpi : capabilities.supported_dpi[0] ?? 300,
          color_mode: capabilities.supported_color_modes.includes(current.color_mode) ? current.color_mode : capabilities.supported_color_modes[0] ?? 'color',
          output_format: capabilities.supported_output_formats.includes(current.output_format) ? current.output_format : capabilities.supported_output_formats[0] ?? 'png',
          source: current.source === 'adf' && capabilities.supports_adf ? 'adf' : 'flatbed'
        }));
      })
      .catch((err) => {
        setScannerCapabilities(null);
        setMessage(String(err));
      });
  }, [sessionId, selectedScannerId]);

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

  const loadPreview = async (scanIntakeId: number, nextPage = 1) => {
    if (!sessionId) return;
    setPreviewLoading(true);
    try {
      const next = await getScanIntakePreviewPage({ sessionId, scanIntakeId, pageNumber: nextPage });
      setPreview(next);
      setPreviewPage(next.page_number);
    } catch (err) {
      setPreview(null);
      setMessage(String(err));
    } finally {
      setPreviewLoading(false);
    }
  };

  useEffect(() => {
    const firstSelected = selectedItems[0];
    if (!firstSelected) {
      setNotesDraft('');
      setPreview(null);
      return;
    }
    setNotesDraft(firstSelected.notes ?? '');
    setPreviewPage(1);
    void loadPreview(firstSelected.scan_intake_id, 1);
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
      const importedIds = await importScanFiles({ sessionId, sourcePaths: selectedPaths });
      setSelectedPaths([]);
      setMessage('Scan file(s) imported.');
      await loadIntake();
      if (importedIds.length) {
        setSelectedIds([Math.max(...importedIds)]);
      }
    } catch (err) {
      setMessage(String(err));
    } finally {
      setBusy(false);
    }
  };

  const captureFromScanner = async () => {
    if (!sessionId || !selectedScannerId || scanBusy) return;
    setScanBusy(true);
    setMessage('Scanner capture started...');
    try {
      const item = await scanToIntake({
        sessionId,
        scannerId: selectedScannerId,
        options: scanOptions
      });
      setMessage('Scanner capture added to pending intake.');
      await loadIntake();
      setSelectedIds([item.scan_intake_id]);
    } catch (err) {
      setMessage(String(err));
    } finally {
      setScanBusy(false);
    }
  };

  const toggleSelected = (id: number) => {
    setSelectedIds((current) => current.includes(id) ? current.filter((itemId) => itemId !== id) : [id, ...current]);
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
          <p className="mt-1 text-sm text-muted">Review scanned/imported files here before filing them as official documents.</p>
        </div>
        <button className="btn" onClick={() => void Promise.all([loadLookups(), loadIntake(), loadScanners()]).catch((err) => setMessage(String(err)))} type="button">
          <RefreshCw size={16} />Refresh
        </button>
      </div>

      {message && <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div>}

      <div className="grid gap-5 xl:grid-cols-[0.9fr_1.1fr]">
        <div className="space-y-5">
          <div className="rounded border border-border bg-surface p-5 shadow-sm">
            <div className="mb-3 flex items-center gap-2">
              <ScanLine size={18} className="text-primary" />
              <h2 className="font-semibold text-secondary">Scanner Capture</h2>
            </div>
            <div className="grid gap-3">
              <div className="flex items-end gap-3">
                <label className="min-w-0 flex-1">
                  <span className="form-label">Selected scanner</span>
                  <select className="input" value={selectedScannerId} onChange={(e) => setSelectedScannerId(e.target.value)}>
                    <option value="">No scanner selected</option>
                    {scanners.map((scanner) => (
                      <option key={scanner.device_id} value={scanner.device_id}>
                        {scanner.name}{scanner.is_available ? '' : ' (Unavailable)'}
                      </option>
                    ))}
                  </select>
                </label>
                <button className="btn" onClick={() => void loadScanners().catch((err) => setMessage(String(err)))} type="button">
                  <RefreshCw size={16} />Refresh
                </button>
              </div>
              {scanners.length === 0 ? (
                <p className="rounded border border-dashed border-border p-3 text-sm text-muted">No scanner detected.</p>
              ) : (
                <div className="rounded border border-border p-3 text-sm">
                  <p className="font-medium text-secondary">
                    {scanners.find((scanner) => scanner.device_id === selectedScannerId)?.name ?? 'Scanner'}
                  </p>
                  <p className="text-xs text-muted">
                    {scannerCapabilities?.status ?? scanners.find((scanner) => scanner.device_id === selectedScannerId)?.status ?? 'Status unknown'}
                  </p>
                </div>
              )}
              <div className="grid gap-3 md:grid-cols-2">
                <label>
                  <span className="form-label">DPI</span>
                  <select className="input" value={scanOptions.dpi} onChange={(e) => setScanOptions({ ...scanOptions, dpi: Number(e.target.value) })}>
                    {(scannerCapabilities?.supported_dpi ?? [200, 300, 600]).map((dpi) => <option key={dpi} value={dpi}>{dpi}</option>)}
                  </select>
                </label>
                <label>
                  <span className="form-label">Color mode</span>
                  <select className="input" value={scanOptions.color_mode} onChange={(e) => setScanOptions({ ...scanOptions, color_mode: e.target.value as ScanOptions['color_mode'] })}>
                    {(scannerCapabilities?.supported_color_modes ?? ['color', 'grayscale', 'black_white']).map((mode) => (
                      <option key={mode} value={mode}>{mode === 'black_white' ? 'Black & white' : mode[0].toUpperCase() + mode.slice(1)}</option>
                    ))}
                  </select>
                </label>
                <label>
                  <span className="form-label">Output format</span>
                  <select className="input" value={scanOptions.output_format} onChange={(e) => setScanOptions({ ...scanOptions, output_format: e.target.value as ScanOptions['output_format'] })}>
                    {(scannerCapabilities?.supported_output_formats ?? ['png', 'jpg']).map((format) => <option key={format} value={format}>{format.toUpperCase()}</option>)}
                  </select>
                </label>
                <label>
                  <span className="form-label">Source</span>
                  <select className="input" value={scanOptions.source} onChange={(e) => setScanOptions({ ...scanOptions, source: e.target.value as ScanOptions['source'] })}>
                    <option value="flatbed">Flatbed</option>
                    {scannerCapabilities?.supports_adf && <option value="adf">ADF</option>}
                  </select>
                </label>
              </div>
              <button
                className="btn btn-primary w-full justify-center"
                disabled={scanBusy || !selectedScannerId || scannerCapabilities?.is_available === false}
                onClick={() => void captureFromScanner()}
                type="button"
              >
                <ScanLine size={16} />{scanBusy ? 'Scanning...' : 'Scan to Intake'}
              </button>
              <p className="text-xs text-muted">Flatbed single-page capture. PNG/JPG supported for MVP; PDF batching stays deferred.</p>
            </div>
          </div>

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
                    <p className="truncate text-xs text-muted">Ready to import</p>
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
                    <span className="flex min-w-0 items-center gap-2">
                      <span className="truncate font-medium text-secondary">{item.original_file_name}</span>
                      <span className="rounded bg-primary/10 px-2 py-0.5 text-[11px] font-semibold text-primary">{extensionFromName(item.original_file_name).toUpperCase()}</span>
                    </span>
                    <span className="block text-xs text-muted">{item.status} · {item.created_at} · {item.mime_type} · {sizeLabel(item.file_size_bytes)}{item.is_large ? ' · Large file' : ''}</span>
                    {item.notes && <span className="mt-1 block text-xs text-muted">{item.notes}</span>}
                  </span>
                </label>
              ))}
            </div>
          </div>
        </div>

        <div className="space-y-5">
          <ScanIntakePreview
            item={selectedItems[0] ?? null}
            loading={previewLoading}
            onPageChange={(nextPage) => selectedItems[0] && void loadPreview(selectedItems[0].scan_intake_id, nextPage)}
            page={previewPage}
            preview={preview}
          />

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

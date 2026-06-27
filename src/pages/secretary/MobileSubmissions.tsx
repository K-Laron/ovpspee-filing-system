import { convertFileSrc } from '@tauri-apps/api/core';
import { AlertTriangle, CheckCircle2, ChevronLeft, ChevronRight, Clock3, Eye, FileText, Paperclip, QrCode, RefreshCw, Search, Smartphone, XCircle } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import type { FormEvent } from 'react';

import { ConfirmDialog, type ConfirmAction } from '../../components/ConfirmDialog';
import { EmptyState } from '../../components/EmptyState';
import { formatDateOnly, formatDateTime } from '../../lib/dates';
import { getUserErrorMessage } from '../../lib/errors';
import { invoke } from '@tauri-apps/api/core';
import { sizeLabel } from '../../lib/helpers';
import { useSessionStore } from '../../store/sessionStore';
import type {
  MobileReviewStatus,
  MobileApiSetup,
  MobileSubmissionAttachmentItem,
  MobileSubmissionAttachmentPreviewPage,
  MobileSubmissionDetail,
  MobileSubmissionItem
} from '../../types';

type FilterStatus = MobileReviewStatus | '';

const reviewStatuses: FilterStatus[] = ['', 'Pending', 'Approved', 'Rejected', 'Removed'];
const rejectionTemplates = [
  'Metadata does not match the attached document.',
  'Wrong category or folder selected.',
  'Attachment is unreadable. Please recapture and resend.'
];

const statusClass = (status: MobileReviewStatus) => {
  if (status === 'Approved') return 'bg-emerald-50 text-emerald-700 border-emerald-200';
  if (status === 'Rejected') return 'bg-red-50 text-red-700 border-red-200';
  if (status === 'Removed') return 'bg-slate-100 text-slate-600 border-slate-200';
  return 'bg-amber-50 text-amber-800 border-amber-200';
};


const detailFromItem = (submission: MobileSubmissionItem): MobileSubmissionDetail => ({
  submission,
  attachments: []
});

export const MobileSubmissions = () => {
  const sessionId = useSessionStore((state) => state.sessionId);
  const [filter, setFilter] = useState<FilterStatus>('Pending');
  const [search, setSearch] = useState('');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const searchRef = useRef<HTMLInputElement>(null);
  const [submissions, setSubmissions] = useState<MobileSubmissionItem[]>([]);
  const [detail, setDetail] = useState<MobileSubmissionDetail | null>(null);
  const [setup, setSetup] = useState<MobileApiSetup | null>(null);
  const [message, setMessage] = useState('');
  const [reviewNotes, setReviewNotes] = useState('');
  const [rejectReason, setRejectReason] = useState('');
  const [rejecting, setRejecting] = useState(false);
  const [busy, setBusy] = useState(false);
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null);
  const clearConfirmAction = () => setConfirmAction(null);
  const [selectedAttachmentId, setSelectedAttachmentId] = useState<number | null>(null);
  const [preview, setPreview] = useState<MobileSubmissionAttachmentPreviewPage | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);

  const selected = detail?.submission;
  const attachments = detail?.attachments ?? [];
  const selectedId = selected?.mobile_submission_id ?? null;
  const pendingCount = useMemo(
    () => submissions.filter((submission) => submission.review_status === 'Pending').length,
    [submissions]
  );

  const openDetail = async (submission: MobileSubmissionItem) => {
    setDetail(detailFromItem(submission));
    setReviewNotes(submission.review_notes ?? '');
    setSelectedAttachmentId(null);
    setPreview(null);
    if (!sessionId) return;

    try {
      const nextDetail = await invoke<MobileSubmissionDetail>('get_mobile_submission', {
        sessionId,
        mobileSubmissionId: submission.mobile_submission_id
      });
      if (nextDetail) {
        setDetail(nextDetail);
        setReviewNotes(nextDetail.submission.review_notes ?? '');
      }
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not load mobile submission attachments. Metadata is still available.'));
    }
  };

  const loadSubmissions = async () => {
    if (!sessionId) return;
    const rows = await invoke<MobileSubmissionItem[]>('list_mobile_submissions', {
      sessionId,
      reviewStatus: filter || null,
      search: search.trim() || null,
      dateFrom: dateFrom || null,
      dateTo: dateTo || null
    });
    setSubmissions(rows);
    if (rows.length === 0) {
      setDetail(null);
      return;
    }

    const current = rows.find((row) => row.mobile_submission_id === selectedId) ?? rows[0];
    await openDetail(current);
  };

  useEffect(() => {
    void loadSubmissions().catch((err) => setMessage(getUserErrorMessage(err, 'Could not load mobile submissions. Please refresh and try again.')));
  }, [sessionId, filter]);

  // ponytail: debounced auto-search on filter changes
  useEffect(() => {
    const timer = setTimeout(() => {
      void loadSubmissions().catch((err) => setMessage(getUserErrorMessage(err, 'Could not load mobile submissions. Please refresh and try again.')));
    }, 300);
    return () => clearTimeout(timer);
  }, [search, dateFrom, dateTo]);

  // ponytail: / key focuses search input
  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      if (e.key === '/' && !['INPUT', 'TEXTAREA', 'SELECT'].includes((e.target as HTMLElement)?.tagName)) {
        e.preventDefault();
        searchRef.current?.focus();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  useEffect(() => {
    void invoke<MobileApiSetup>('get_mobile_api_setup')
      .then(setSetup)
      .catch(() => setSetup(null));
  }, []);

  const approveSelected = async () => {
    if (!sessionId || !detail || busy) return;
    setBusy(true);
    setMessage('');
    try {
      const documentId = await invoke<number>('approve_mobile_submission', {
        sessionId,
        mobileSubmissionId: detail.submission.mobile_submission_id,
        reviewNotes: reviewNotes.trim() || null
      });
      setMessage(`Mobile submission approved as document #${documentId}.`);
      await loadSubmissions();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not approve the mobile submission. Please review metadata and try again.'));
    } finally {
      setBusy(false);
      clearConfirmAction();
    }
  };

  const submitReject = async (event: FormEvent) => {
    event.preventDefault();
    if (!sessionId || !detail || busy || !rejectReason.trim()) return;
    setBusy(true);
    setMessage('');
    try {
      await invoke<void>('reject_mobile_submission', {
        sessionId,
        mobileSubmissionId: detail.submission.mobile_submission_id,
        rejectionReason: rejectReason.trim()
      });
      setMessage('Mobile submission rejected.');
      setRejecting(false);
      setRejectReason('');
      await loadSubmissions();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not reject the mobile submission. Please enter a reason and try again.'));
    } finally {
      setBusy(false);
    }
  };

  const confirmApprove = () => {
    if (!detail) return;
    setConfirmAction({
      title: 'Approve mobile submission?',
      body: <>Create an official document from <strong>{detail.submission.document_name}</strong> with its submitted metadata and attachments.</>,
      confirmLabel: 'Approve Submission',
      onConfirm: approveSelected
    });
  };

  const loadPreview = async (attachment: MobileSubmissionAttachmentItem, pageNumber = 1) => {
    if (!sessionId) return;
    setSelectedAttachmentId(attachment.mobile_submission_attachment_id);
    setPreviewLoading(true);
    try {
      setPreview(await invoke<MobileSubmissionAttachmentPreviewPage>('get_mobile_submission_attachment_preview_page', {
        sessionId,
        mobileSubmissionAttachmentId: attachment.mobile_submission_attachment_id,
        pageNumber
      }));
    } catch (err) {
      setPreview(null);
      setMessage(getUserErrorMessage(err, 'Could not preview this mobile attachment.'));
    } finally {
      setPreviewLoading(false);
    }
  };

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && rejecting) {
        setRejecting(false);
        return;
      }
      if (!selected || selected.review_status !== 'Pending' || busy) return;
      if (event.ctrlKey && event.key === 'Enter') {
        event.preventDefault();
        confirmApprove();
      }
      if (event.ctrlKey && (event.key === 'Backspace' || event.key === 'Delete')) {
        event.preventDefault();
        setRejecting(true);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [busy, rejecting, selected?.mobile_submission_id, selected?.review_status]);

  return (
    <section className="space-y-5">
      {confirmAction && (
        <ConfirmDialog
          body={confirmAction.body}
          confirmLabel={confirmAction.confirmLabel}
          onCancel={() => clearConfirmAction()}
          onConfirm={() => void confirmAction.onConfirm()}
          title={confirmAction.title}
          tone="default"
        />
      )}

      {rejecting && selected && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-secondary/40 p-6">
          <form className="w-full max-w-lg rounded border border-border bg-surface p-5 shadow-xl" onSubmit={submitReject}>
            <div className="mb-4 flex items-start gap-3">
              <XCircle className="mt-0.5 shrink-0 text-primary" size={22} />
              <div>
                <h2 className="font-semibold text-secondary">Reject mobile submission</h2>
                <p className="mt-1 text-sm text-muted">Return {selected.document_name} to the Android submitter with a clear reason.</p>
              </div>
            </div>
            <label>
              <span className="form-label">Rejection reason</span>
              <textarea
                autoFocus
                className="input min-h-28"
                onChange={(event) => setRejectReason(event.target.value)}
                required
                value={rejectReason}
              />
            </label>
            <div className="mt-3 flex flex-wrap gap-2">
              {rejectionTemplates.map((template) => (
                <button
                  className="btn text-xs"
                  key={template}
                  onClick={() => setRejectReason(template)}
                  type="button"
                >
                  {template}
                </button>
              ))}
            </div>
            <div className="mt-5 flex justify-end gap-2">
              <button className="btn" disabled={busy} onClick={() => setRejecting(false)} type="button">Cancel</button>
              <button className="btn btn-primary" disabled={busy || !rejectReason.trim()} type="submit">
                {busy ? 'Rejecting...' : 'Reject Submission'}
              </button>
            </div>
          </form>
        </div>
      )}

      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-bold text-secondary">Mobile Submissions</h1>
          <p className="mt-1 text-sm text-muted">Review Android-captured files before they become official document records.</p>
        </div>
        <button className="btn" onClick={() => void loadSubmissions().catch((err) => setMessage(getUserErrorMessage(err, 'Could not load mobile submissions. Please refresh and try again.')))} type="button">
          <RefreshCw size={16} />Refresh
        </button>
      </div>

      <div className="grid gap-3 rounded border border-border bg-surface p-4 shadow-sm lg:grid-cols-[1fr_1.2fr_auto]">
        <div className="flex items-center gap-3">
          <span className="flex h-10 w-10 items-center justify-center rounded border border-primary/20 bg-primary/10 text-primary">
            <QrCode size={19} />
          </span>
          <div>
            <h2 className="font-semibold text-secondary">Android Setup</h2>
            <p className="text-sm text-muted">
              {setup ? `${setup.local_ip} · ${setup.enabled ? 'Mobile API enabled' : 'Enable OVPSPEE_MOBILE_API_ENABLED=1'}` : 'Setup details unavailable'}
            </p>
          </div>
        </div>
        <div className="min-w-0 rounded border border-border bg-background p-3 text-xs text-muted">
          <p className="truncate font-mono text-secondary">{setup?.setup_url ?? 'ovpspee://setup unavailable'}</p>
          <p className="mt-1">{setup?.device_token_required ? 'Device token required.' : 'Device token optional for this hub.'}</p>
        </div>
        <button
          className="btn self-center"
          onClick={() => {
            if (setup?.setup_url) void navigator.clipboard?.writeText(setup.setup_url);
          }}
          type="button"
        >
          Copy setup
        </button>
      </div>

      <div className="grid gap-3 rounded border border-border bg-surface p-4 shadow-sm xl:grid-cols-[1fr_180px_220px_160px_160px_auto]">
        <div className="flex items-center gap-3">
          <span className="flex h-10 w-10 items-center justify-center rounded border border-primary/20 bg-primary/10 text-primary">
            <Smartphone size={19} />
          </span>
          <div>
            <p className="text-sm font-semibold text-secondary">{pendingCount} pending in current view</p>
            <p className="text-xs text-muted">Approve clean uploads or reject with instructions for correction.</p>
          </div>
        </div>
        <label>
          <span className="form-label">Review status</span>
          <select className="input" value={filter} onChange={(event) => setFilter(event.target.value as FilterStatus)}>
            {reviewStatuses.map((status) => (
              <option key={status || 'All'} value={status}>{status || 'All'}</option>
            ))}
          </select>
        </label>
        <label>
          <span className="form-label">Search submissions</span>
          <div className="relative">
            <Search className="pointer-events-none absolute left-3 top-3 text-muted" size={16} />
            <input
              ref={searchRef}
              aria-label="Search submissions"
              className="input pl-9"
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Name, sender, device"
              value={search}
            />
          </div>
        </label>
        <label>
          <span className="form-label">Date from</span>
          <input
            aria-label="Date from"
            className="input"
            onChange={(event) => setDateFrom(event.target.value)}
            type="date"
            value={dateFrom}
          />
        </label>
        <label>
          <span className="form-label">Date to</span>
          <input
            aria-label="Date to"
            className="input"
            onChange={(event) => setDateTo(event.target.value)}
            type="date"
            value={dateTo}
          />
        </label>
        <button className="btn btn-primary self-end" onClick={() => void loadSubmissions().catch((err) => setMessage(getUserErrorMessage(err, 'Could not load mobile submissions. Please refresh and try again.')))} type="button">
          Apply
        </button>
      </div>

      {message && <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div>}

      <div className="grid gap-5 xl:grid-cols-[0.88fr_1.12fr]">
        <div className="overflow-hidden rounded border border-border bg-surface shadow-sm">
          <div className="border-b border-border p-4">
            <h2 className="font-semibold text-secondary">Review Queue</h2>
            <p className="text-sm text-muted">{submissions.length} submission(s)</p>
          </div>
          {submissions.length === 0 ? (
            <div className="p-4">
              <EmptyState
                message="No Android submissions match this review status."
                title="Queue is clear"
              />
            </div>
          ) : (
            <div className="divide-y divide-border">
              {submissions.map((submission) => (
                <button
                  className={`block w-full p-4 text-left hover:bg-background ${selectedId === submission.mobile_submission_id ? 'bg-background' : ''}`}
                  key={submission.mobile_submission_id}
                  onClick={() => void openDetail(submission)}
                  type="button"
                >
                  <span className="flex items-start justify-between gap-3">
                    <span className="min-w-0">
                      <span className="block truncate font-semibold text-secondary">{submission.document_name}</span>
                      <span className="mt-1 block text-xs text-muted">
                        {submission.submitter_name} · {formatDateOnly(submission.date_received)} · {submission.attachment_count} file(s)
                      </span>
                    </span>
                    <span className={`shrink-0 rounded border px-2 py-0.5 text-[11px] font-semibold ${statusClass(submission.review_status)}`}>
                      {submission.review_status}
                    </span>
                  </span>
                  <span className="mt-2 block truncate text-xs text-muted">
                    {submission.category_name}{submission.folder_name ? ` / ${submission.folder_name}` : ''}{submission.office_name ? ` · ${submission.office_name}` : ''}
                  </span>
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="rounded border border-border bg-surface p-5 shadow-sm">
          {!selected ? (
            <EmptyState
              message="Choose one mobile upload to inspect its filing metadata and decide whether it should become an official document."
              title="No submission selected"
            />
          ) : (
            <div className="space-y-5">
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="mb-2 flex items-center gap-2">
                    <FileText size={18} className="text-primary" />
                    <span className={`rounded border px-2 py-0.5 text-xs font-semibold ${statusClass(selected.review_status)}`}>{selected.review_status}</span>
                  </div>
                  <h2 className="text-xl font-bold text-secondary">{selected.document_name}</h2>
                  <p className="mt-1 text-sm text-muted">
                    Submitted by {selected.submitter_name} on {formatDateTime(selected.created_at)}
                  </p>
                </div>
                <div className="flex flex-wrap gap-2">
                  <button className="btn btn-primary" disabled={busy || selected.review_status !== 'Pending'} onClick={confirmApprove} type="button">
                    <CheckCircle2 size={16} />Approve
                  </button>
                  <button className="btn" disabled={busy || selected.review_status !== 'Pending'} onClick={() => setRejecting(true)} type="button">
                    <XCircle size={16} />Reject
                  </button>
                </div>
              </div>

              <dl className="grid gap-3 md:grid-cols-2">
                <MetadataItem label="Submitted by" value={selected.submitter_name} />
                <MetadataItem label="Device" value={selected.submitted_device_name ?? selected.submitted_device_id ?? 'Not reported'} />
                <MetadataItem label="Category" value={selected.category_name} />
                <MetadataItem label="Folder" value={selected.folder_name ?? 'Category root'} />
                <MetadataItem label="Sender office" value={selected.office_name ?? 'Not specified'} />
                <MetadataItem label="Date received" value={formatDateOnly(selected.date_received)} />
                <MetadataItem label="Document status" value={selected.status} />
                <MetadataItem label="Attachment count" value={`${selected.attachment_count} file(s)`} />
                <MetadataItem label="Client submission" value={selected.client_submission_id ?? 'Not reported'} />
                <MetadataItem className="md:col-span-2" label="Remarks" value={selected.remarks ?? 'None'} />
              </dl>

              {selected.review_status === 'Rejected' && selected.rejection_reason && (
                <div className="rounded border border-red-200 bg-red-50 p-3 text-sm text-red-700">
                  <p className="font-semibold">Rejection reason</p>
                  <p className="mt-1">{selected.rejection_reason}</p>
                </div>
              )}

              {selected.resulting_document_id && (
                <div className="rounded border border-emerald-200 bg-emerald-50 p-3 text-sm text-emerald-700">
                  Approved as document #{selected.resulting_document_id}.
                </div>
              )}

              <label>
                <span className="form-label">Review notes</span>
                <textarea
                  className="input min-h-24"
                  disabled={selected.review_status !== 'Pending'}
                  onChange={(event) => setReviewNotes(event.target.value)}
                  placeholder="Optional notes saved with approval"
                  value={reviewNotes}
                />
              </label>

              <div className="border-t border-border pt-4">
                <div className="mb-3 flex items-center gap-2">
                  <Paperclip size={18} className="text-primary" />
                  <h3 className="font-semibold text-secondary">Attachments</h3>
                </div>
                <AttachmentList
                  attachments={attachments}
                  expectedCount={selected.attachment_count}
                  onPreview={(attachment) => void loadPreview(attachment)}
                  selectedAttachmentId={selectedAttachmentId}
                />
                <MobileAttachmentPreview
                  loading={previewLoading}
                  onPageChange={(pageNumber) => {
                    const attachment = attachments.find((item) => item.mobile_submission_attachment_id === selectedAttachmentId);
                    if (attachment) void loadPreview(attachment, pageNumber);
                  }}
                  preview={preview}
                />
              </div>

              <div className="rounded border border-border bg-background p-3 text-xs text-muted">
                <div className="flex items-center gap-2 font-semibold text-secondary"><Clock3 size={15} />Audit trail</div>
                <p className="mt-1">Created {formatDateTime(selected.created_at)} · Updated {formatDateTime(selected.updated_at)}</p>
                {selected.reviewed_at && <p>Reviewed {formatDateTime(selected.reviewed_at)}{selected.reviewer_name ? ` by ${selected.reviewer_name}` : ''}</p>}
              </div>
            </div>
          )}
        </div>
      </div>
    </section>
  );
};

const MetadataItem = ({ className = '', label, value }: { className?: string; label: string; value: string }) => (
  <div className={`rounded border border-border bg-background p-3 ${className}`}>
    <dt className="text-xs font-semibold uppercase text-muted">{label}</dt>
    <dd className="mt-1 break-words text-sm font-medium text-secondary">{value}</dd>
  </div>
);

const AttachmentList = ({
  attachments,
  expectedCount,
  onPreview,
  selectedAttachmentId
}: {
  attachments: MobileSubmissionAttachmentItem[];
  expectedCount: number;
  onPreview: (attachment: MobileSubmissionAttachmentItem) => void;
  selectedAttachmentId: number | null;
}) => {
  if (attachments.length === 0) {
    return (
      <EmptyState
        message={expectedCount > 0 ? `${expectedCount} file(s) submitted. Attachment names will appear when backend detail is available.` : 'No attachments are listed for this submission.'}
        title={expectedCount > 0 ? 'Attachment metadata pending' : 'No attachments'}
      />
    );
  }

  return (
    <div className="space-y-2">
      {attachments.map((attachment) => (
        <div className={`flex items-center justify-between gap-3 rounded border p-3 text-sm ${selectedAttachmentId === attachment.mobile_submission_attachment_id ? 'border-primary bg-primary/5' : 'border-border'}`} key={attachment.mobile_submission_attachment_id}>
          <div className="min-w-0">
            <p className="truncate font-medium text-secondary">{attachment.original_file_name}</p>
            <p className="truncate text-xs text-muted">{attachment.mime_type} · {sizeLabel(attachment.file_size_bytes)}</p>
          </div>
          <div className="flex shrink-0 items-center gap-2">
            <span className="rounded bg-background px-2 py-1 text-xs font-semibold text-secondary">#{attachment.sort_order}</span>
            <button className="btn" onClick={() => onPreview(attachment)} type="button">
              <Eye size={15} />Preview
            </button>
          </div>
        </div>
      ))}
    </div>
  );
};

const MobileAttachmentPreview = ({
  loading,
  onPageChange,
  preview
}: {
  loading: boolean;
  onPageChange: (pageNumber: number) => void;
  preview: MobileSubmissionAttachmentPreviewPage | null;
}) => {
  if (loading) {
    return <div className="mt-3 rounded border border-border bg-background p-4 text-sm text-muted">Loading preview...</div>;
  }
  if (!preview) {
    return <div className="mt-3 rounded border border-border bg-background p-4 text-sm text-muted">Select Preview on an attachment.</div>;
  }

  const info = preview.info;
  const maxPage = info.page_count ?? 1;
  const canPage = info.preview_kind === 'Pdf' && maxPage > 1;
  const fileUrl = preview.file_path ? convertFileSrc(preview.file_path) : null;

  return (
    <div className="mt-3 space-y-3 rounded border border-border bg-background p-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <p className="truncate text-sm font-semibold text-secondary">{info.original_file_name}</p>
          <p className="text-xs text-muted">{info.preview_kind} · {info.extension} · {sizeLabel(info.file_size_bytes)}</p>
        </div>
        {canPage ? (
          <div className="flex items-center gap-2">
            <button aria-label="Previous mobile attachment preview page" className="icon-btn" disabled={preview.page_number <= 1} onClick={() => onPageChange(preview.page_number - 1)} title="Previous page" type="button">
              <ChevronLeft size={15} />
            </button>
            <span className="text-xs font-semibold text-secondary">PAGE {preview.page_number} of {maxPage}</span>
            <button aria-label="Next mobile attachment preview page" className="icon-btn" disabled={preview.page_number >= maxPage} onClick={() => onPageChange(preview.page_number + 1)} title="Next page" type="button">
              <ChevronRight size={15} />
            </button>
          </div>
        ) : null}
      </div>

      {!info.file_exists ? (
        <div className="rounded border border-warning/30 bg-warning/10 p-4 text-sm text-warning">
          <div className="flex items-center gap-2 font-semibold"><AlertTriangle size={16} />File unavailable</div>
          <p className="mt-1">{info.message}</p>
        </div>
      ) : null}
      {info.file_exists && info.preview_kind === 'Text' ? (
        <pre className="max-h-[26rem] overflow-auto whitespace-pre-wrap break-words rounded border border-border bg-white p-3 text-xs leading-relaxed text-secondary">{preview.text_content ?? info.message}</pre>
      ) : null}
      {info.file_exists && info.preview_kind === 'Image' && fileUrl ? (
        <div className="max-h-[32rem] overflow-auto rounded border border-border bg-white p-3">
          <img alt={info.original_file_name} className="mx-auto max-h-[30rem] max-w-full object-contain" src={fileUrl} />
        </div>
      ) : null}
      {info.file_exists && info.preview_kind === 'Pdf' && fileUrl ? (
        <iframe className="h-[32rem] w-full rounded border border-border bg-white" src={`${fileUrl}#page=${preview.page_number}`} title={info.original_file_name} />
      ) : null}
      {info.file_exists && info.preview_kind === 'Unsupported' ? (
        <div className="rounded border border-border bg-surface p-4 text-sm text-secondary">{info.message}</div>
      ) : null}
    </div>
  );
};

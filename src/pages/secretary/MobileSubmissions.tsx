import { CheckCircle2, Clock3, FileText, Paperclip, RefreshCw, Smartphone, XCircle } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import type { FormEvent, ReactNode } from 'react';

import { ConfirmDialog } from '../../components/ConfirmDialog';
import { EmptyState } from '../../components/EmptyState';
import { formatDateOnly, formatDateTime } from '../../lib/dates';
import { getUserErrorMessage } from '../../lib/errors';
import {
  approveMobileSubmission,
  getMobileSubmission,
  listMobileSubmissions,
  rejectMobileSubmission
} from '../../lib/invoke';
import { useSessionStore } from '../../store/sessionStore';
import type {
  MobileReviewStatus,
  MobileSubmissionAttachmentItem,
  MobileSubmissionDetail,
  MobileSubmissionItem
} from '../../types';

type FilterStatus = MobileReviewStatus | '';

interface ConfirmAction {
  title: string;
  body: ReactNode;
  confirmLabel: string;
  onConfirm: () => Promise<void>;
}

const reviewStatuses: FilterStatus[] = ['', 'Pending', 'Approved', 'Rejected', 'Removed'];

const statusClass = (status: MobileReviewStatus) => {
  if (status === 'Approved') return 'bg-emerald-50 text-emerald-700 border-emerald-200';
  if (status === 'Rejected') return 'bg-red-50 text-red-700 border-red-200';
  if (status === 'Removed') return 'bg-slate-100 text-slate-600 border-slate-200';
  return 'bg-amber-50 text-amber-800 border-amber-200';
};

const sizeLabel = (bytes: number) => `${Math.ceil(bytes / 1024)} KB`;

const detailFromItem = (submission: MobileSubmissionItem): MobileSubmissionDetail => ({
  submission,
  attachments: []
});

export const MobileSubmissions = () => {
  const sessionId = useSessionStore((state) => state.sessionId);
  const [filter, setFilter] = useState<FilterStatus>('Pending');
  const [submissions, setSubmissions] = useState<MobileSubmissionItem[]>([]);
  const [detail, setDetail] = useState<MobileSubmissionDetail | null>(null);
  const [message, setMessage] = useState('');
  const [reviewNotes, setReviewNotes] = useState('');
  const [rejectReason, setRejectReason] = useState('');
  const [rejecting, setRejecting] = useState(false);
  const [busy, setBusy] = useState(false);
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null);

  const selectedId = detail?.submission.mobile_submission_id ?? null;
  const pendingCount = useMemo(
    () => submissions.filter((submission) => submission.review_status === 'Pending').length,
    [submissions]
  );

  const openDetail = async (submission: MobileSubmissionItem) => {
    setDetail(detailFromItem(submission));
    setReviewNotes(submission.review_notes ?? '');
    if (!sessionId) return;

    try {
      const nextDetail = await getMobileSubmission({
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
    const rows = await listMobileSubmissions({
      sessionId,
      reviewStatus: filter || null
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

  const approveSelected = async () => {
    if (!sessionId || !detail || busy) return;
    setBusy(true);
    setMessage('');
    try {
      const documentId = await approveMobileSubmission({
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
      setConfirmAction(null);
    }
  };

  const submitReject = async (event: FormEvent) => {
    event.preventDefault();
    if (!sessionId || !detail || busy || !rejectReason.trim()) return;
    setBusy(true);
    setMessage('');
    try {
      await rejectMobileSubmission({
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

  const selected = detail?.submission;
  const attachments = detail?.attachments ?? [];

  return (
    <section className="space-y-5">
      {confirmAction && (
        <ConfirmDialog
          body={confirmAction.body}
          confirmLabel={confirmAction.confirmLabel}
          onCancel={() => setConfirmAction(null)}
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

      <div className="grid gap-3 rounded border border-border bg-surface p-4 shadow-sm md:grid-cols-[1fr_180px_auto]">
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
                <MetadataItem label="Category" value={selected.category_name} />
                <MetadataItem label="Folder" value={selected.folder_name ?? 'Category root'} />
                <MetadataItem label="Sender office" value={selected.office_name ?? 'Not specified'} />
                <MetadataItem label="Date received" value={formatDateOnly(selected.date_received)} />
                <MetadataItem label="Document status" value={selected.status} />
                <MetadataItem label="Attachment count" value={`${selected.attachment_count} file(s)`} />
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
                <AttachmentList attachments={attachments} expectedCount={selected.attachment_count} />
              </div>

              <div className="rounded border border-border bg-background p-3 text-xs text-muted">
                <div className="flex items-center gap-2 font-semibold text-secondary"><Clock3 size={15} />Audit trail</div>
                <p className="mt-1">Created {formatDateTime(selected.created_at)} · Updated {formatDateTime(selected.updated_at)}</p>
                {selected.reviewed_at && <p>Reviewed {formatDateTime(selected.reviewed_at)}</p>}
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
  expectedCount
}: {
  attachments: MobileSubmissionAttachmentItem[];
  expectedCount: number;
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
        <div className="flex items-center justify-between gap-3 rounded border border-border p-3 text-sm" key={attachment.mobile_submission_attachment_id}>
          <div className="min-w-0">
            <p className="truncate font-medium text-secondary">{attachment.original_file_name}</p>
            <p className="truncate text-xs text-muted">{attachment.mime_type} · {sizeLabel(attachment.file_size_bytes)}</p>
          </div>
          <span className="rounded bg-background px-2 py-1 text-xs font-semibold text-secondary">#{attachment.sort_order}</span>
        </div>
      ))}
    </div>
  );
};

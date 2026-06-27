import { convertFileSrc } from '@tauri-apps/api/core';
import { AlertTriangle, ChevronLeft, ChevronRight, Eye, FileText, Info } from 'lucide-react';
import { useEffect, useState } from 'react';

import { getUserErrorMessage } from '../lib/errors';
import { invoke } from '@tauri-apps/api/core';
import { extensionFromName, formatBytes } from '../lib/helpers';
import type { AttachmentItem, AttachmentPreviewPage } from '../types';

interface AttachmentPreviewProps {
  attachment: AttachmentItem | null;
  sessionId?: string | null;
  onError: (message: string) => void;
}

export const AttachmentPreview = ({
  attachment,
  onError,
  sessionId = null,
}: AttachmentPreviewProps) => {
  const [preview, setPreview] = useState<AttachmentPreviewPage | null>(null);
  const [pageNumber, setPageNumber] = useState(1);
  const [loading, setLoading] = useState(false);

  const load = async (nextPage: number) => {
    if (!attachment) {
      setPreview(null);
      return;
    }
    setLoading(true);
    try {
      const next = await invoke<AttachmentPreviewPage>('get_attachment_preview_page', {
        attachmentId: attachment.attachment_id,
        sessionId,
        pageNumber: nextPage,
      });
      setPreview(next);
      setPageNumber(next.page_number);
    } catch (err) {
      setPreview(null);
      onError(getUserErrorMessage(err, 'Could not preview this attachment.'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    setPageNumber(1);
    void load(1);
  }, [attachment?.attachment_id, sessionId]);

  if (!attachment) {
    return (
      <div className="rounded border border-border bg-background p-4 text-sm text-muted">
        Select an attachment to preview.
      </div>
    );
  }

  const info = preview?.info;
  const maxPage = info?.page_count ?? 1;
  const canPage = info?.preview_kind === 'Pdf' && maxPage > 1;
  const fileUrl = preview?.file_path ? convertFileSrc(preview.file_path) : null;
  const kindLabel = info?.preview_kind ?? 'Loading';

  return (
    <div className="space-y-3 rounded border border-border bg-background p-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <p className="truncate text-sm font-semibold text-secondary">
            {attachment.original_file_name}
          </p>
          <p className="text-xs text-muted">
            {kindLabel} · {info?.extension ?? extensionFromName(attachment.original_file_name)} ·{' '}
            {info?.mime_type ?? attachment.mime_type} · {formatBytes(attachment.file_size_bytes)}
          </p>
        </div>
        <span className="rounded bg-surface px-2 py-1 text-xs font-semibold text-secondary">
          {kindLabel}
        </span>
        <div className="flex items-center gap-2">
          {canPage && (
            <>
              <button
                aria-label="Previous preview page"
                className="icon-btn"
                disabled={loading || pageNumber <= 1}
                onClick={() => void load(pageNumber - 1)}
                title="Previous page"
                type="button"
              >
                <ChevronLeft size={15} />
              </button>
              <span className="text-xs font-semibold text-secondary">
                PAGE {pageNumber} of {maxPage}
              </span>
              <button
                aria-label="Next preview page"
                className="icon-btn"
                disabled={loading || pageNumber >= maxPage}
                onClick={() => void load(pageNumber + 1)}
                title="Next page"
                type="button"
              >
                <ChevronRight size={15} />
              </button>
            </>
          )}
        </div>
      </div>

      {loading && (
        <div className="rounded border border-border bg-surface p-4 text-sm text-muted">
          Loading preview...
        </div>
      )}
      {!loading && info && !info.file_exists && <Unavailable message={info.message} />}
      {!loading && preview && info?.file_exists && info.preview_kind === 'Text' && (
        <div className="rounded border border-border bg-white">
          <div className="flex items-center gap-2 border-b border-border px-3 py-2 text-sm font-semibold text-secondary">
            <FileText size={16} />
            Text preview
          </div>
          {preview.text_content ? (
            <pre className="max-h-[28rem] overflow-auto whitespace-pre-wrap break-words p-3 text-xs leading-relaxed text-secondary">
              {preview.text_content}
            </pre>
          ) : (
            <div className="p-4 text-sm text-muted">{info.message}</div>
          )}
          {preview.text_truncated && (
            <p className="border-t border-border px-3 py-2 text-xs text-muted">
              Preview is capped for safety. Use the existing access action if more content is
              needed.
            </p>
          )}
        </div>
      )}
      {!loading && info?.file_exists && info.preview_kind === 'Image' && fileUrl && (
        <div className="max-h-[34rem] overflow-auto rounded border border-border bg-white p-3">
          <img
            alt={attachment.original_file_name}
            className="mx-auto max-h-[32rem] max-w-full object-contain"
            src={fileUrl}
          />
        </div>
      )}
      {!loading && info?.file_exists && info.preview_kind === 'Pdf' && fileUrl && (
        <iframe
          className="h-[34rem] w-full rounded border border-border bg-white"
          src={`${fileUrl}#page=${pageNumber}`}
          title={attachment.original_file_name}
        />
      )}
      {!loading && info?.file_exists && info.preview_kind === 'Unsupported' && (
        <div className="rounded border border-border bg-surface p-4 text-sm text-secondary">
          <div className="mb-2 flex items-center gap-2 font-semibold">
            <Eye size={16} />
            Preview not available for this file type
          </div>
          <p>{info.message}</p>
          <dl className="mt-3 grid gap-2 text-xs text-muted sm:grid-cols-3">
            <div>
              <dt className="font-semibold text-secondary">Type</dt>
              <dd>{info.extension.toUpperCase()}</dd>
            </div>
            <div>
              <dt className="font-semibold text-secondary">MIME</dt>
              <dd>{info.mime_type}</dd>
            </div>
            <div>
              <dt className="font-semibold text-secondary">Size</dt>
              <dd>{formatBytes(info.file_size_bytes)}</dd>
            </div>
          </dl>
          <p className="mt-3 flex items-center gap-2 text-xs text-muted">
            <Info size={14} />
            Use the existing safe access action to open the stored attachment, if permitted.
          </p>
        </div>
      )}
    </div>
  );
};

const Unavailable = ({ message }: { message: string }) => (
  <div className="rounded border border-warning/30 bg-warning/10 p-4 text-sm text-warning">
    <div className="flex items-center gap-2 font-semibold">
      <AlertTriangle size={16} />
      File unavailable
    </div>
    <p className="mt-1">{message}</p>
  </div>
);

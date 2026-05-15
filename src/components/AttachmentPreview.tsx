import { convertFileSrc } from '@tauri-apps/api/core';
import { ChevronLeft, ChevronRight, Eye } from 'lucide-react';
import { useEffect, useState } from 'react';

import { getAttachmentPreviewPage } from '../lib/invoke';
import type { AttachmentItem, AttachmentPreviewPage } from '../types';

interface AttachmentPreviewProps {
  attachment: AttachmentItem | null;
  sessionId?: string | null;
  onError: (message: string) => void;
}

export const AttachmentPreview = ({ attachment, onError, sessionId = null }: AttachmentPreviewProps) => {
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
      const next = await getAttachmentPreviewPage({
        attachmentId: attachment.attachment_id,
        sessionId,
        pageNumber: nextPage
      });
      setPreview(next);
      setPageNumber(next.page_number);
    } catch (err) {
      setPreview(null);
      onError(String(err));
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

  return (
    <div className="space-y-3 rounded border border-border bg-background p-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <p className="truncate text-sm font-semibold text-secondary">{attachment.original_file_name}</p>
          <p className="text-xs text-muted">{info?.mime_type ?? attachment.mime_type} · {Math.ceil(attachment.file_size_bytes / 1024)} KB</p>
        </div>
        <div className="flex items-center gap-2">
          {canPage && (
            <>
              <button className="icon-btn" disabled={loading || pageNumber <= 1} onClick={() => void load(pageNumber - 1)} title="Previous page" type="button">
                <ChevronLeft size={15} />
              </button>
              <span className="text-xs font-semibold text-secondary">PAGE {pageNumber} of {maxPage}</span>
              <button className="icon-btn" disabled={loading || pageNumber >= maxPage} onClick={() => void load(pageNumber + 1)} title="Next page" type="button">
                <ChevronRight size={15} />
              </button>
            </>
          )}
        </div>
      </div>

      {loading && <div className="rounded border border-border bg-surface p-4 text-sm text-muted">Loading preview...</div>}
      {!loading && info && !info.file_exists && <Unavailable message={info.message} />}
      {!loading && info?.file_exists && info.preview_kind === 'Image' && fileUrl && (
        <div className="max-h-[34rem] overflow-auto rounded border border-border bg-white p-3">
          <img alt={attachment.original_file_name} className="mx-auto max-h-[32rem] max-w-full object-contain" src={fileUrl} />
        </div>
      )}
      {!loading && info?.file_exists && info.preview_kind === 'Pdf' && fileUrl && (
        <iframe className="h-[34rem] w-full rounded border border-border bg-white" src={`${fileUrl}#page=${pageNumber}`} title={attachment.original_file_name} />
      )}
      {!loading && info?.file_exists && info.preview_kind === 'Unsupported' && (
        <div className="rounded border border-border bg-surface p-4 text-sm text-secondary">
          <div className="mb-2 flex items-center gap-2 font-semibold"><Eye size={16} />Unsupported preview</div>
          <p>{info.message}</p>
          <p className="mt-2 text-xs text-muted">Stored file remains accessible through the system attachment record.</p>
        </div>
      )}
    </div>
  );
};

const Unavailable = ({ message }: { message: string }) => (
  <div className="rounded border border-warning/30 bg-warning/10 p-4 text-sm text-warning">
    {message}
  </div>
);

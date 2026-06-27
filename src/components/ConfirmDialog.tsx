import { AlertTriangle, X } from 'lucide-react';
import { useEffect, useId, useRef, useState } from 'react';
import type { KeyboardEvent, ReactNode } from 'react';

interface ConfirmDialogProps {
  title: string;
  body: ReactNode;
  confirmLabel: string;
  cancelLabel?: string;
  requiredText?: string;
  tone?: 'default' | 'danger';
  onCancel: () => void;
  onConfirm: () => void | Promise<void>;
}

export interface ConfirmAction {
  title: string;
  body: ReactNode;
  confirmLabel: string;
  requiredText?: string;
  onConfirm: () => Promise<void>;
}

export const ConfirmDialog = ({
  title,
  body,
  confirmLabel,
  cancelLabel = 'Cancel',
  requiredText,
  tone = 'danger',
  onCancel,
  onConfirm,
}: ConfirmDialogProps) => {
  const titleId = useId();
  const inputId = useId();
  const dialogRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const cancelButtonRef = useRef<HTMLButtonElement>(null);
  const previousFocusRef = useRef<Element | null>(null);
  const [typedText, setTypedText] = useState('');
  const [confirming, setConfirming] = useState(false);
  const isConfirmationRequired = Boolean(requiredText);
  const canConfirm = !requiredText || typedText === requiredText;
  const iconClassName = tone === 'danger' ? 'text-primary' : 'text-warning';

  useEffect(() => {
    previousFocusRef.current = document.activeElement;
    const initialFocus = requiredText ? inputRef.current : cancelButtonRef.current;
    initialFocus?.focus();

    return () => {
      const previousFocus = previousFocusRef.current;
      if (previousFocus instanceof HTMLElement) {
        previousFocus.focus();
      }
    };
  }, [requiredText]);

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === 'Escape') {
      event.stopPropagation();
      if (confirming) {
        return;
      }
      onCancel();
      return;
    }

    if (event.key !== 'Tab') {
      return;
    }

    const focusableElements = getFocusableElements(dialogRef.current);
    if (focusableElements.length === 0) {
      event.preventDefault();
      return;
    }

    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];

    if (event.shiftKey && document.activeElement === firstElement) {
      event.preventDefault();
      lastElement.focus();
      return;
    }

    if (!event.shiftKey && document.activeElement === lastElement) {
      event.preventDefault();
      firstElement.focus();
    }
  };

  const handleConfirm = async () => {
    if (confirming) {
      return;
    }

    setConfirming(true);
    try {
      await onConfirm();
    } finally {
      setConfirming(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-secondary/40 p-6">
      <div
        aria-labelledby={titleId}
        aria-modal="true"
        className="w-full max-w-md rounded border border-border bg-surface p-5 shadow-xl"
        onKeyDown={handleKeyDown}
        ref={dialogRef}
        role="dialog"
      >
        <div className="mb-4 flex items-start justify-between gap-4">
          <div className="flex min-w-0 items-start gap-3">
            <AlertTriangle
              aria-hidden="true"
              className={`mt-0.5 shrink-0 ${iconClassName}`}
              size={22}
            />
            <div className="min-w-0">
              <h2 className="text-base font-semibold text-secondary" id={titleId}>
                {title}
              </h2>
              <div className="mt-2 text-sm leading-6 text-muted">{body}</div>
            </div>
          </div>
          <button
            aria-label="Close dialog"
            className="icon-btn shrink-0"
            disabled={confirming}
            onClick={onCancel}
            title="Close dialog"
            type="button"
          >
            <X size={16} />
          </button>
        </div>

        {isConfirmationRequired && (
          <div className="mb-5">
            <label className="form-label" htmlFor={inputId}>
              Type {requiredText} to confirm
            </label>
            <input
              autoComplete="off"
              className="input"
              id={inputId}
              onChange={(event) => setTypedText(event.target.value)}
              ref={inputRef}
              value={typedText}
            />
          </div>
        )}

        <div className="flex justify-end gap-2">
          <button
            className="btn"
            disabled={confirming}
            onClick={onCancel}
            ref={cancelButtonRef}
            type="button"
          >
            {cancelLabel}
          </button>
          <button
            className="btn btn-primary"
            disabled={!canConfirm || confirming}
            onClick={handleConfirm}
            type="button"
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
};

const getFocusableElements = (root: HTMLElement | null) => {
  if (!root) {
    return [];
  }

  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])',
    ),
  ).filter((element) => !element.hasAttribute('hidden') && element.tabIndex !== -1);
};

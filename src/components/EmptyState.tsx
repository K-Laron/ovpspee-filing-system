interface EmptyStateProps {
  title: string;
  message: string;
  actionLabel?: string;
  onAction?: () => void;
}

export const EmptyState = ({ actionLabel, message, onAction, title }: EmptyStateProps) => (
  <div className="rounded border border-dashed border-border bg-background p-4 text-sm">
    <p className="font-semibold text-secondary">{title}</p>
    <p className="mt-1 text-muted">{message}</p>
    {actionLabel && onAction && (
      <button className="btn mt-3" onClick={onAction} type="button">
        {actionLabel}
      </button>
    )}
  </div>
);

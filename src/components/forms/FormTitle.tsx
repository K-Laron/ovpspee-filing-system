import { X } from 'lucide-react';

export const FormTitle = ({
  editing,
  label,
  onCancel,
}: {
  editing: boolean;
  label: string;
  onCancel: () => void;
}) => (
  <div className="flex items-center justify-between gap-2">
    <h2 className="text-base font-semibold text-secondary">
      {editing ? `Edit ${label}` : `New ${label}`}
    </h2>
    {editing && (
      <button
        aria-label="Cancel edit"
        className="focus-ring inline-flex h-8 w-8 items-center justify-center rounded border border-border text-muted hover:text-secondary"
        onClick={onCancel}
        title="Cancel edit"
        type="button"
      >
        <X size={15} />
      </button>
    )}
  </div>
);

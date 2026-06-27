import type { ReactNode } from 'react';

export const IconButton = ({
  children,
  label,
  onClick,
}: {
  children: ReactNode;
  label: string;
  onClick: () => void;
}) => (
  <button
    aria-label={label}
    className="focus-ring inline-flex h-9 w-9 items-center justify-center rounded border border-border text-secondary hover:bg-background"
    onClick={onClick}
    title={label}
    type="button"
  >
    {children}
  </button>
);

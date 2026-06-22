import { useState, type ReactNode } from 'react';

export interface ConfirmAction {
  title: string;
  body: ReactNode;
  confirmLabel: string;
  requiredText?: string;
  onConfirm: () => Promise<void>;
}

export const useConfirmAction = () => {
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null);
  const clearConfirmAction = () => setConfirmAction(null);
  return { confirmAction, setConfirmAction, clearConfirmAction };
};

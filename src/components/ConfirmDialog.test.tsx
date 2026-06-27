import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { ConfirmDialog } from './ConfirmDialog';

describe('ConfirmDialog', () => {
  it('calls onConfirm for simple confirmation', async () => {
    const onConfirm = vi.fn();
    render(
      <ConfirmDialog
        title="Move document to trash?"
        body="This document can be restored from Trash."
        confirmLabel="Move to Trash"
        onCancel={vi.fn()}
        onConfirm={onConfirm}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Move to Trash' }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('requires exact typed confirmation when requiredText is set', async () => {
    const onConfirm = vi.fn();
    render(
      <ConfirmDialog
        title="Permanently purge trash?"
        body="This cannot be undone."
        confirmLabel="Purge"
        requiredText="PURGE"
        onCancel={vi.fn()}
        onConfirm={onConfirm}
      />,
    );

    expect(screen.getByRole('button', { name: 'Purge' })).toBeDisabled();
    fireEvent.change(screen.getByLabelText('Type PURGE to confirm'), {
      target: { value: 'PURGE' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Purge' }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('prevents repeat confirms while async confirmation is pending', async () => {
    let resolveConfirm: (() => void) | undefined;
    const onCancel = vi.fn();
    const onConfirm = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          resolveConfirm = resolve;
        }),
    );

    render(
      <ConfirmDialog
        title="Move document to trash?"
        body="This document can be restored from Trash."
        confirmLabel="Move to Trash"
        onCancel={onCancel}
        onConfirm={onConfirm}
      />,
    );

    const confirmButton = screen.getByRole('button', { name: 'Move to Trash' });
    const cancelButton = screen.getByRole('button', { name: 'Cancel' });
    const closeButton = screen.getByRole('button', { name: 'Close dialog' });
    fireEvent.click(confirmButton);

    await waitFor(() => expect(confirmButton).toBeDisabled());
    expect(cancelButton).toBeDisabled();
    expect(closeButton).toBeDisabled();

    fireEvent.click(confirmButton);
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });

    expect(onConfirm).toHaveBeenCalledTimes(1);
    expect(onCancel).not.toHaveBeenCalled();

    resolveConfirm?.();
    await waitFor(() => expect(confirmButton).not.toBeDisabled());
  });

  it('calls onCancel from cancel and close controls', async () => {
    const onCancel = vi.fn();
    render(
      <ConfirmDialog
        title="Discard changes?"
        body="Unsaved changes will be lost."
        confirmLabel="Discard"
        onCancel={onCancel}
        onConfirm={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    fireEvent.click(screen.getByRole('button', { name: 'Close dialog' }));
    expect(onCancel).toHaveBeenCalledTimes(2);
  });

  it('calls onCancel when Escape is pressed', async () => {
    const onCancel = vi.fn();
    render(
      <ConfirmDialog
        title="Discard changes?"
        body="Unsaved changes will be lost."
        confirmLabel="Discard"
        onCancel={onCancel}
        onConfirm={vi.fn()}
      />,
    );

    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it('focuses the typed confirmation input when confirmation text is required', () => {
    render(
      <ConfirmDialog
        title="Permanently purge trash?"
        body="This cannot be undone."
        confirmLabel="Purge"
        requiredText="PURGE"
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />,
    );

    expect(screen.getByLabelText('Type PURGE to confirm')).toHaveFocus();
  });

  it('wraps Tab from the last focusable control to the first control', () => {
    render(
      <>
        <button type="button">Background action</button>
        <ConfirmDialog
          title="Discard changes?"
          body="Unsaved changes will be lost."
          confirmLabel="Discard"
          onCancel={vi.fn()}
          onConfirm={vi.fn()}
        />
      </>,
    );

    const backgroundButton = screen.getByRole('button', { name: 'Background action' });
    const closeButton = screen.getByRole('button', { name: 'Close dialog' });
    const confirmButton = screen.getByRole('button', { name: 'Discard' });

    confirmButton.focus();
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Tab' });
    expect(closeButton).toHaveFocus();
    expect(backgroundButton).not.toHaveFocus();
  });

  it('restores focus to the trigger after cancel unmounts the dialog', async () => {
    render(<DialogHarness />);

    const triggerButton = screen.getByRole('button', { name: 'Open dialog' });
    triggerButton.focus();
    fireEvent.click(triggerButton);
    expect(screen.getByRole('button', { name: 'Cancel' })).toHaveFocus();

    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(triggerButton).toHaveFocus();
  });

  it('reverse-wraps Shift+Tab from first focusable control to last focusable control', () => {
    render(
      <>
        <button type="button">Background action</button>
        <ConfirmDialog
          title="Discard changes?"
          body="Unsaved changes will be lost."
          confirmLabel="Discard"
          onCancel={vi.fn()}
          onConfirm={vi.fn()}
        />
      </>,
    );

    const backgroundButton = screen.getByRole('button', { name: 'Background action' });
    const closeButton = screen.getByRole('button', { name: 'Close dialog' });
    const cancelButton = screen.getByRole('button', { name: 'Cancel' });
    const confirmButton = screen.getByRole('button', { name: 'Discard' });

    closeButton.focus();
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Tab', shiftKey: true });

    expect(confirmButton).toHaveFocus();
    expect(cancelButton).not.toHaveFocus();
    expect(backgroundButton).not.toHaveFocus();
  });
});

const DialogHarness = () => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      <button onClick={() => setIsOpen(true)} type="button">
        Open dialog
      </button>
      {isOpen && (
        <ConfirmDialog
          title="Discard changes?"
          body="Unsaved changes will be lost."
          confirmLabel="Discard"
          onCancel={() => setIsOpen(false)}
          onConfirm={vi.fn()}
        />
      )}
    </>
  );
};

import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { EmptyState } from './EmptyState';

describe('EmptyState', () => {
  it('renders message and optional action', async () => {
    const onAction = vi.fn();
    render(
      <EmptyState
        title="No scanner detected"
        message="Connect a scanner, turn it on, then refresh devices."
        actionLabel="Refresh Devices"
        onAction={onAction}
      />
    );

    expect(screen.getByText('No scanner detected')).toBeInTheDocument();
    expect(screen.getByText('Connect a scanner, turn it on, then refresh devices.')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Refresh Devices' }));
    expect(onAction).toHaveBeenCalledTimes(1);
  });

  it('does not render an action button without both action props', () => {
    const { rerender } = render(
      <EmptyState title="No documents" message="Create a document to get started." actionLabel="Create Document" />
    );

    expect(screen.queryByRole('button', { name: 'Create Document' })).not.toBeInTheDocument();

    rerender(<EmptyState title="No documents" message="Create a document to get started." onAction={vi.fn()} />);

    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});

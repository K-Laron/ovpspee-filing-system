import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../../lib/invoke', () => ({
  createMobileDevice: vi.fn(),
  listMobileDevices: vi.fn(),
  revokeMobileDevice: vi.fn()
}));

vi.mock('../../store/sessionStore', () => ({
  useSessionStore: (selector: (state: { sessionId: string }) => string) =>
    selector({ sessionId: 'session-1' })
}));

import { listMobileDevices, revokeMobileDevice } from '../../lib/invoke';
import type { MobileDeviceItem } from '../../types';

const device: MobileDeviceItem = {
  mobile_device_id: 1,
  device_id: 'device-1',
  device_name: 'Records Android',
  is_active: true,
  last_seen_at: '2026-05-21T09:00:00Z',
  created_by: 1,
  created_at: '2026-05-21T08:00:00Z',
  updated_at: '2026-05-21T09:00:00Z'
};

describe('MobileDevices', () => {
  beforeEach(() => {
    vi.mocked(listMobileDevices).mockResolvedValue([device]);
    vi.mocked(revokeMobileDevice).mockResolvedValue(undefined);
  });

  it('shows device list, create token, and revoke controls', async () => {
    const { MobileDevices } = await import('./MobileDevices');

    render(<MobileDevices />);

    expect(await screen.findByText('Mobile Devices')).toBeInTheDocument();
    expect(await screen.findByText('Records Android')).toBeInTheDocument();
    expect(screen.getByText('device-1')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /create token/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /revoke/i })).toBeInTheDocument();
  });

  it('requires confirmation before revoking a mobile device', async () => {
    const user = userEvent.setup();
    const { MobileDevices } = await import('./MobileDevices');

    render(<MobileDevices />);

    await user.click(await screen.findByRole('button', { name: /revoke/i }));

    expect(revokeMobileDevice).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog', { name: /revoke mobile device/i })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /revoke device/i }));

    await waitFor(() =>
      expect(revokeMobileDevice).toHaveBeenCalledWith({
        sessionId: 'session-1',
        deviceId: 'device-1'
      })
    );
  });
});

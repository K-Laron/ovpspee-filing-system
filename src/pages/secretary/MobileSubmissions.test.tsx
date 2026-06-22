import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../../lib/invoke', () => ({
  cmd: vi.fn()
}));

vi.mock('../../store/sessionStore', () => ({
  useSessionStore: () => 'session-1'
}));

import { cmd } from '../../lib/invoke';
import type { MobileApiSetup, MobileSubmissionDetail, MobileSubmissionItem } from '../../types';

const pendingSubmission: MobileSubmissionItem = {
  mobile_submission_id: 18,
  submitted_by: 2,
  submitter_name: 'Sec User',
  document_name: 'Mobile BAC memo',
  category_id: 1,
  category_name: 'BAC',
  folder_id: null,
  folder_name: null,
  office_id: 3,
  office_name: 'OVP Records',
  date_received: '2026-05-20',
  remarks: 'Captured on Android',
  status: 'Filed',
  review_status: 'Pending',
  rejection_reason: null,
  review_notes: null,
  reviewed_by: null,
  reviewed_at: null,
  resulting_document_id: null,
  client_submission_id: 'mobile-client-18',
  submitted_device_id: 'device-1',
  submitted_device_name: 'Records phone',
  reviewer_name: null,
  attachment_count: 2,
  created_at: '2026-05-20T08:00:00Z',
  updated_at: '2026-05-20T08:00:00Z'
};

const pendingDetail: MobileSubmissionDetail = {
  submission: pendingSubmission,
  attachments: [
    {
      mobile_submission_attachment_id: 9,
      mobile_submission_id: 18,
      original_file_name: 'mobile-bac-memo.pdf',
      mime_type: 'application/pdf',
      file_size_bytes: 4096,
      sort_order: 1,
      created_at: '2026-05-20T08:00:00Z'
    }
  ]
};

describe('MobileSubmissions', () => {
  beforeEach(() => {
    vi.mocked(cmd).mockImplementation((name: string) => {
      if (name === 'list_mobile_submissions') return Promise.resolve([pendingSubmission]);
      if (name === 'get_mobile_submission') return Promise.resolve(pendingDetail);
      if (name === 'get_mobile_api_setup') return Promise.resolve({
        enabled: true,
        bind_addr: '0.0.0.0:1421',
        local_ip: '192.168.1.50',
        setup_url: 'ovpspee://setup?hub=http%3A%2F%2F192.168.1.50%3A1421',
        device_token_required: true
      });
      return Promise.reject(new Error(`unexpected cmd: ${name}`));
    });
  });

  it('shows setup, filters, and pending mobile submissions for review', async () => {
    const { MobileSubmissions } = await import('./MobileSubmissions');

    render(<MobileSubmissions />);

    expect(await screen.findByText('Android Setup')).toBeInTheDocument();
    expect(screen.getAllByText(/192\.168\.1\.50/).length).toBeGreaterThan(0);
    expect(screen.getByLabelText('Search submissions')).toBeInTheDocument();
    expect(screen.getByLabelText('Date from')).toBeInTheDocument();
    expect(screen.getByLabelText('Date to')).toBeInTheDocument();
    expect(await screen.findAllByText('Mobile BAC memo')).toHaveLength(2);
    expect(screen.getByText('Sec User')).toBeInTheDocument();
    expect(screen.getByText('Records phone')).toBeInTheDocument();
    expect(screen.getAllByText('Pending').length).toBeGreaterThan(0);
    expect(screen.getByText('Captured on Android')).toBeInTheDocument();
    expect(screen.getByText('2 file(s)')).toBeInTheDocument();
    expect(await screen.findByRole('button', { name: /preview/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /approve/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /reject/i })).toBeInTheDocument();
  });
});

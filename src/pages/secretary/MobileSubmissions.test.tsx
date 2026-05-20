import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../../lib/invoke', () => ({
  approveMobileSubmission: vi.fn(),
  getMobileSubmission: vi.fn(),
  listMobileSubmissions: vi.fn(),
  rejectMobileSubmission: vi.fn()
}));

vi.mock('../../store/sessionStore', () => ({
  useSessionStore: () => 'session-1'
}));

import { listMobileSubmissions } from '../../lib/invoke';
import type { MobileSubmissionItem } from '../../types';

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
  attachment_count: 2,
  created_at: '2026-05-20T08:00:00Z',
  updated_at: '2026-05-20T08:00:00Z'
};

describe('MobileSubmissions', () => {
  beforeEach(() => {
    vi.mocked(listMobileSubmissions).mockResolvedValue([pendingSubmission]);
  });

  it('shows pending mobile submissions for review', async () => {
    const { MobileSubmissions } = await import('./MobileSubmissions');

    render(<MobileSubmissions />);

    expect(await screen.findAllByText('Mobile BAC memo')).toHaveLength(2);
    expect(screen.getByText('Sec User')).toBeInTheDocument();
    expect(screen.getAllByText('Pending').length).toBeGreaterThan(0);
    expect(screen.getByText('Captured on Android')).toBeInTheDocument();
    expect(screen.getByText('2 file(s)')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /approve/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /reject/i })).toBeInTheDocument();
  });
});

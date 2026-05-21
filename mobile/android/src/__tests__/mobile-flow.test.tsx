import React from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react-native';

import { AppRoot } from '../AppRoot';
import type { LookupsPayload } from '../types';

jest.mock('../native/capture', () => ({
  pickFile: jest.fn(async () => ({
    uri: 'content://office/mobile-picked.pdf',
    name: 'mobile-picked.pdf',
    type: 'application/pdf',
    sizeBytes: 4096
  })),
  capturePhoto: jest.fn(async () => ({
    uri: 'file:///cache/mobile-photo.jpg',
    name: 'mobile-photo.jpg',
    type: 'image/jpeg',
    sizeBytes: 2048
  }))
}));

jest.mock('../storage/drafts', () => ({
  clearQueuedSubmissions: jest.fn(),
  loadDeviceProfile: jest.fn(async () => ({ deviceId: 'device-1', deviceName: 'Records phone', deviceToken: '' })),
  saveDraft: jest.fn(),
  loadDraft: jest.fn(async () => null),
  loadQueuedSubmissions: jest.fn(async () => []),
  clearDraft: jest.fn(),
  markQueuedSubmissionAttempt: jest.fn(),
  newClientSubmissionId: jest.fn(() => 'mobile-client-test'),
  removeQueuedSubmission: jest.fn(),
  saveDeviceProfile: jest.fn(),
  saveHubUrl: jest.fn(),
  loadHubUrl: jest.fn(async () => null),
  saveQueuedSubmission: jest.fn()
}));

const lookups: LookupsPayload = {
  categories: [{ category_id: 1, category_name: 'BAC' }],
  folders: [{ folder_id: 2, folder_name: 'Procurement' }],
  offices: [{ office_id: 3, office_name: 'Accounting' }]
};

jest.setTimeout(20000);

describe('mobile capture flow', () => {
  it('logs in, requires complete metadata, submits an attachment, and shows history', async () => {
    const api = {
      health: jest.fn(async () => ({ status: 'ok' as const })),
      login: jest.fn(async () => ({
        session_id: 'session-1',
        user_id: 7,
        role: 'Secretary' as const,
        display_name: 'Sec User',
        profile_pic_path: null
      })),
      getLookups: jest.fn(async () => lookups),
      createSubmission: jest.fn(async () => ({ mobile_submission_id: 42 })),
      listSubmissions: jest.fn(async () => [
        {
          mobile_submission_id: 42,
          document_name: 'Mobile BAC memo',
          review_status: 'Pending' as const,
          rejection_reason: null,
          created_at: '2026-05-20T08:00:00Z'
        }
      ])
    };

    render(<AppRoot api={api} initialHubUrl="http://10.0.0.5:1421" />);

    fireEvent.changeText(screen.getByLabelText('Username'), 'sec1');
    fireEvent.changeText(screen.getByLabelText('Password'), 'Secret123!');
    fireEvent.press(screen.getByText('Login'));

    expect(await screen.findByText('Start with the document, then complete the filing details.')).toBeTruthy();
    fireEvent.press(screen.getByText('Add file'));
    expect(await screen.findByText('mobile-picked.pdf')).toBeTruthy();
    fireEvent.press(screen.getByText('Camera capture'));
    expect(await screen.findByText('mobile-photo.jpg')).toBeTruthy();
    fireEvent.press(screen.getByText('Next'));
    expect(await screen.findByText('Full Add Document metadata')).toBeTruthy();
    fireEvent.press(screen.getByText('Next'));
    expect(await screen.findByText('Complete all required metadata before review.')).toBeTruthy();

    fireEvent.changeText(screen.getByLabelText('Document title'), 'Mobile BAC memo');
    fireEvent.changeText(screen.getByLabelText('Category ID'), '1');
    fireEvent.changeText(screen.getByLabelText('Folder ID'), '2');
    fireEvent.changeText(screen.getByLabelText('Sender office ID'), '3');
    fireEvent.changeText(screen.getByLabelText('Date received'), '2026-05-20');
    fireEvent.changeText(screen.getByLabelText('Remarks'), 'Captured on Android');
    fireEvent.press(screen.getByText('Next'));

    expect(await screen.findByText('Review Attachments')).toBeTruthy();
    fireEvent.press(screen.getByText('Submit to office PC'));

    await waitFor(() => expect(api.createSubmission).toHaveBeenCalled());
    expect(await screen.findByText('Pending review #42')).toBeTruthy();
    fireEvent.press(screen.getByText('History'));

    expect(await screen.findByText('Mobile BAC memo')).toBeTruthy();
    expect(screen.getByText('Pending')).toBeTruthy();
  });
});

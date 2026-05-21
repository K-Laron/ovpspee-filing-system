import {
  clearQueuedSubmissions,
  loadQueuedSubmissions,
  markQueuedSubmissionAttempt,
  saveQueuedSubmission
} from '../storage/drafts';
import type { MobileSubmissionDraft } from '../types';

const draft: MobileSubmissionDraft = {
  clientSubmissionId: 'queue-1',
  documentName: 'Offline memo',
  categoryId: 1,
  folderId: 2,
  officeId: 3,
  dateReceived: '2026-05-20',
  remarks: 'Queued while offline',
  status: 'Filed',
  attachments: [{ uri: 'file:///scan.pdf', name: 'scan.pdf', type: 'application/pdf', sizeBytes: 2048 }]
};

describe('submission queue storage', () => {
  beforeEach(async () => {
    await clearQueuedSubmissions();
  });

  it('persists queued submissions and attempt metadata', async () => {
    await saveQueuedSubmission({
      clientSubmissionId: 'queue-1',
      draft,
      attempts: 0,
      lastError: 'Office PC hub is not reachable.',
      queuedAt: '2026-05-21T08:00:00Z'
    });

    await markQueuedSubmissionAttempt('queue-1', 'Retry failed.');

    expect(await loadQueuedSubmissions()).toEqual([
      expect.objectContaining({
        clientSubmissionId: 'queue-1',
        attempts: 1,
        lastError: 'Retry failed.',
        draft: expect.objectContaining({ documentName: 'Offline memo' })
      })
    ]);
  });
});

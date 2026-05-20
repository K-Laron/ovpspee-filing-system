import { ApiClient } from '../api/client';
import type { MobileSubmissionDraft } from '../types';

describe('ApiClient', () => {
  beforeEach(() => {
    globalThis.fetch = jest.fn(async () => ({
      ok: true,
      json: async () => ({
        session_id: 'session-1',
        user_id: 7,
        role: 'Secretary',
        display_name: 'Sec User',
        profile_pic_path: null
      })
    })) as jest.Mock;
  });

  it('posts login to the mobile endpoint without retaining the password', async () => {
    const client = new ApiClient('http://10.0.0.5:1421');

    await client.login('sec1', 'Secret123!');

    expect(globalThis.fetch).toHaveBeenCalledWith(
      'http://10.0.0.5:1421/api/mobile/login',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: 'sec1', password: 'Secret123!' })
      })
    );
    expect(JSON.stringify(client)).not.toContain('Secret123!');
  });

  it('submits metadata and files with bearer authorization', async () => {
    globalThis.fetch = jest.fn(async () => ({
      ok: true,
      json: async () => ({ mobile_submission_id: 42 })
    })) as jest.Mock;
    const draft: MobileSubmissionDraft = {
      documentName: 'Mobile BAC memo',
      categoryId: 1,
      folderId: 2,
      officeId: 3,
      dateReceived: '2026-05-20',
      remarks: 'Captured on Android',
      status: 'Filed',
      attachments: [{ uri: 'file:///scan.pdf', name: 'scan.pdf', type: 'application/pdf' }]
    };

    const result = await new ApiClient('http://hub.local').createSubmission('session-1', draft);

    expect(result.mobile_submission_id).toBe(42);
    expect(globalThis.fetch).toHaveBeenCalledWith(
      'http://hub.local/api/mobile/submissions',
      expect.objectContaining({
        method: 'POST',
        headers: { Authorization: 'Bearer session-1' },
        body: expect.any(FormData)
      })
    );
  });
});

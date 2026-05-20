import type {
  LookupsPayload,
  MobileApi,
  MobileSubmissionDraft,
  SessionPayload,
  SubmissionHistoryItem
} from '../types';

export class ApiClient implements MobileApi {
  private readonly baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl.replace(/\/+$/, '');
  }

  async health(): Promise<{ status: 'ok' }> {
    const response = await fetch(`${this.baseUrl}/api/mobile/health`);
    return this.parse(response, 'Office PC hub is not reachable.');
  }

  async login(username: string, password: string): Promise<SessionPayload> {
    const response = await fetch(`${this.baseUrl}/api/mobile/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    });
    return this.parse(response, 'Login failed. Check your account and office Wi-Fi.');
  }

  async getLookups(sessionId: string): Promise<LookupsPayload> {
    const response = await fetch(`${this.baseUrl}/api/mobile/lookups`, {
      headers: { Authorization: `Bearer ${sessionId}` }
    });
    return this.parse(response, 'Could not load metadata lists from the office PC.');
  }

  async listSubmissions(sessionId: string): Promise<SubmissionHistoryItem[]> {
    const response = await fetch(`${this.baseUrl}/api/mobile/submissions`, {
      headers: { Authorization: `Bearer ${sessionId}` }
    });
    return this.parse(response, 'Could not load mobile submission history.');
  }

  async createSubmission(
    sessionId: string,
    draft: MobileSubmissionDraft
  ): Promise<{ mobile_submission_id: number }> {
    const data = new FormData();
    data.append(
      'metadata',
      JSON.stringify({
        document_name: draft.documentName,
        category_id: draft.categoryId,
        folder_id: draft.folderId,
        office_id: draft.officeId,
        date_received: draft.dateReceived,
        remarks: draft.remarks || null,
        status: draft.status
      })
    );
    draft.attachments.forEach((file) => {
      data.append('files', {
        uri: file.uri,
        name: file.name,
        type: file.type
      } as unknown as Blob);
    });

    const response = await fetch(`${this.baseUrl}/api/mobile/submissions`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${sessionId}` },
      body: data
    });
    return this.parse(response, 'Could not submit. Check the office PC connection and try again.');
  }

  private async parse<T>(response: Response, fallback: string): Promise<T> {
    if (!response.ok) {
      throw new Error(fallback);
    }
    return response.json() as Promise<T>;
  }
}

import type {
  DeviceProfile,
  LookupsPayload,
  MobileApi,
  MobileSubmissionDraft,
  SessionPayload,
  SubmissionHistoryItem
} from '../types';

const MAX_UPLOAD_ATTEMPTS = 3;

export const buildSubmissionMetadata = (
  draft: MobileSubmissionDraft,
  deviceProfile?: DeviceProfile
) => ({
  client_submission_id: draft.clientSubmissionId,
  device_id: deviceProfile?.deviceId ?? null,
  device_name: deviceProfile?.deviceName ?? null,
  document_name: draft.documentName,
  category_id: draft.categoryId,
  folder_id: draft.folderId,
  office_id: draft.officeId,
  date_received: draft.dateReceived,
  remarks: draft.remarks || null,
  status: draft.status
});

export class ApiClient implements MobileApi {
  private readonly baseUrl: string;
  private readonly deviceProfile?: DeviceProfile;

  constructor(baseUrl: string, deviceProfile?: DeviceProfile) {
    this.baseUrl = baseUrl.replace(/\/+$/, '');
    this.deviceProfile = deviceProfile;
  }

  async health(): Promise<{ status: 'ok' }> {
    const response = await fetch(`${this.baseUrl}/api/mobile/health`, {
      headers: this.deviceHeaders()
    });
    return this.parse(response, 'Office PC hub is not reachable.');
  }

  async login(username: string, password: string, deviceProfile: DeviceProfile): Promise<SessionPayload> {
    const response = await fetch(`${this.baseUrl}/api/mobile/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...this.deviceHeaders() },
      body: JSON.stringify({
        username,
        password,
        device_id: deviceProfile.deviceId,
        device_name: deviceProfile.deviceName
      })
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
      JSON.stringify(buildSubmissionMetadata(draft, this.deviceProfile))
    );
    draft.attachments.forEach((file) => {
      data.append('files', {
        uri: file.uri,
        name: file.name,
        type: file.type
      } as unknown as Blob);
    });

    const response = await this.fetchWithRetry(`${this.baseUrl}/api/mobile/submissions`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${sessionId}`, ...this.deviceHeaders() },
      body: data
    });
    return this.parse(response, 'Could not submit. Check the office PC connection and try again.');
  }

  private deviceHeaders(): Record<string, string> {
    const token = this.deviceProfile?.deviceToken.trim();
    return token ? { 'X-OVPSPEE-Device-Token': token } : {};
  }

  private async fetchWithRetry(input: RequestInfo, init: RequestInit): Promise<Response> {
    let lastError: unknown;
    for (let attempt = 1; attempt <= MAX_UPLOAD_ATTEMPTS; attempt += 1) {
      try {
        return await fetch(input, init);
      } catch (err) {
        lastError = err;
        if (attempt === MAX_UPLOAD_ATTEMPTS) break;
      }
    }
    throw lastError instanceof Error ? lastError : new Error('Network request failed.');
  }

  private async parse<T>(response: Response, fallback: string): Promise<T> {
    if (!response.ok) {
      throw new Error(fallback);
    }
    return response.json() as Promise<T>;
  }
}

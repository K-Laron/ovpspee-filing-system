export type DocumentStatus = 'Filed' | 'Archived' | 'Confidential' | 'Other';
export type ReviewStatus = 'Pending' | 'Approved' | 'Rejected' | 'Removed';

export interface SessionPayload {
  session_id: string;
  user_id: number;
  role: 'Secretary';
  display_name: string;
  profile_pic_path: string | null;
}

export interface LookupItem {
  category_id?: number;
  category_name?: string;
  folder_id?: number;
  folder_name?: string;
  office_id?: number;
  office_name?: string;
}

export interface LookupsPayload {
  categories: Array<Required<Pick<LookupItem, 'category_id' | 'category_name'>>>;
  folders: Array<Required<Pick<LookupItem, 'folder_id' | 'folder_name'>>>;
  offices: Array<Required<Pick<LookupItem, 'office_id' | 'office_name'>>>;
}

export interface MobileAttachmentDraft {
  uri: string;
  name: string;
  type: string;
}

export interface MobileSubmissionDraft {
  documentName: string;
  categoryId: number | null;
  folderId: number | null;
  officeId: number | null;
  dateReceived: string;
  remarks: string;
  status: DocumentStatus;
  attachments: MobileAttachmentDraft[];
}

export interface SubmissionHistoryItem {
  mobile_submission_id: number;
  document_name: string;
  review_status: ReviewStatus;
  rejection_reason: string | null;
  created_at: string;
}

export interface MobileApi {
  health(): Promise<{ status: 'ok' }>;
  login(username: string, password: string): Promise<SessionPayload>;
  getLookups(sessionId: string): Promise<LookupsPayload>;
  createSubmission(
    sessionId: string,
    draft: MobileSubmissionDraft
  ): Promise<{ mobile_submission_id: number }>;
  listSubmissions(sessionId: string): Promise<SubmissionHistoryItem[]>;
}

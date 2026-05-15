export type Role = 'Admin' | 'Secretary';

export interface SessionPayload {
  session_id: string;
  user_id: number;
  role: Role;
  display_name: string;
  profile_pic_path: string | null;
}

export interface CategoryItem {
  category_id: number;
  category_name: string;
  description: string | null;
  color_code: string;
  icon: string | null;
  is_system: boolean;
  is_active: boolean;
  document_count: number;
}

export interface FolderItem {
  folder_id: number;
  category_id: number;
  category_name: string;
  folder_name: string;
  description: string | null;
  folder_color: string;
  is_active: boolean;
  document_count: number;
}

export interface OfficeItem {
  office_id: number;
  office_name: string;
  description: string | null;
  is_active: boolean;
}

export interface UserItem {
  user_id: number;
  role: Role;
  first_name: string;
  middle_name: string | null;
  last_name: string;
  username: string;
  email: string | null;
  contact_number: string | null;
  address: string | null;
  profile_pic_path: string | null;
  is_active: boolean;
  last_login_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface ProfileItem {
  user_id: number;
  role: Role;
  first_name: string;
  middle_name: string | null;
  last_name: string;
  username: string;
  email: string | null;
  contact_number: string | null;
  address: string | null;
  profile_pic_path: string | null;
}

export type DocumentStatus = 'Filed' | 'Archived' | 'Confidential' | 'Other';

export interface DocumentItem {
  document_id: number;
  document_name: string;
  category_id: number;
  category_name: string;
  folder_id: number | null;
  folder_name: string | null;
  office_id: number | null;
  office_name: string | null;
  date_received: string;
  date_added: string;
  remarks: string | null;
  status: DocumentStatus;
  is_hidden: boolean;
  is_trashed: boolean;
  attachment_count: number;
  created_by: number;
  created_by_name: string;
  updated_at: string;
}

export interface AttachmentItem {
  attachment_id: number;
  document_id: number;
  original_file_name: string;
  stored_relative_path: string;
  mime_type: string;
  file_size_bytes: number;
  sort_order: number;
  created_at: string;
}

export interface DocumentDetail {
  document: DocumentItem;
  attachments: AttachmentItem[];
}

export interface ScanIntakeItem {
  scan_intake_id: number;
  original_file_name: string;
  stored_relative_path: string;
  mime_type: string;
  file_size_bytes: number;
  status: 'Pending' | 'Filed' | 'Removed';
  notes: string | null;
  is_deleted: boolean;
  is_large: boolean;
  created_by: number;
  created_at: string;
  updated_at: string;
  filed_document_id: number | null;
}

export interface AuditLogEntry {
  id: number;
  action: string;
  actor_user_id: number | null;
  actor_username: string | null;
  actor_display_name: string | null;
  actor_role: Role | null;
  entity_type: string | null;
  entity_id: number | null;
  summary: string;
  created_at: string;
  ip_address: string | null;
}

export interface AuditLogPage {
  entries: AuditLogEntry[];
  limit: number;
  offset: number;
}

export interface AuditRetentionSettings {
  retention_months: number;
  min_months: number;
  max_months: number;
  cleanup_deferred: boolean;
}

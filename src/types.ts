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
export type MobileReviewStatus = 'Pending' | 'Approved' | 'Rejected' | 'Removed';

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

export interface MobileSubmissionItem {
  mobile_submission_id: number;
  submitted_by: number;
  submitter_name: string;
  document_name: string;
  category_id: number;
  category_name: string;
  folder_id: number | null;
  folder_name: string | null;
  office_id: number | null;
  office_name: string | null;
  date_received: string;
  remarks: string | null;
  status: DocumentStatus;
  review_status: MobileReviewStatus;
  rejection_reason: string | null;
  review_notes: string | null;
  reviewed_by: number | null;
  reviewer_name: string | null;
  reviewed_at: string | null;
  resulting_document_id: number | null;
  client_submission_id: string | null;
  submitted_device_id: string | null;
  submitted_device_name: string | null;
  attachment_count: number;
  created_at: string;
  updated_at: string;
}

export interface MobileApiSetup {
  enabled: boolean;
  bind_addr: string;
  local_ip: string;
  setup_url: string;
  device_token_required: boolean;
}

export interface MobileSubmissionAttachmentItem {
  mobile_submission_attachment_id: number;
  mobile_submission_id: number;
  original_file_name: string;
  mime_type: string;
  file_size_bytes: number;
  sort_order: number;
  created_at: string;
}

export interface MobileSubmissionDetail {
  submission: MobileSubmissionItem;
  attachments: MobileSubmissionAttachmentItem[];
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

export interface ScanIntakePreviewInfo {
  scan_intake_id: number;
  original_file_name: string;
  extension: string;
  mime_type: string;
  file_size_bytes: number;
  preview_kind: 'Pdf' | 'Image' | 'Text' | 'Unsupported';
  page_count: number | null;
  file_exists: boolean;
  supported: boolean;
  message: string;
}

export interface ScanIntakePreviewPage {
  info: ScanIntakePreviewInfo;
  page_number: number;
  preview_data_url: string | null;
  text_content: string | null;
  text_truncated: boolean;
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

export interface AttachmentPreviewInfo {
  attachment_id: number;
  document_id: number;
  original_file_name: string;
  extension: string;
  mime_type: string;
  file_size_bytes: number;
  preview_kind: 'Pdf' | 'Image' | 'Text' | 'Unsupported';
  page_count: number | null;
  file_exists: boolean;
  supported: boolean;
  message: string;
}

export interface AttachmentPreviewPage {
  info: AttachmentPreviewInfo;
  page_number: number;
  file_path: string | null;
  text_content: string | null;
  text_truncated: boolean;
}

export interface BackupSettings {
  destination_path: string;
  is_local_app_data: boolean;
  schedule_enabled: boolean;
  schedule_time: string;
  retention_count: number;
}

export interface BackupSummary {
  backup_name: string;
  backup_path: string;
  manifest_path: string;
  database_path: string;
  storage_path: string;
  created_at: string;
  total_bytes: number;
  file_count: number;
  is_valid: boolean;
}

export interface BackupValidation {
  is_valid: boolean;
  backup_name: string;
  created_at: string;
  app_version: string;
  schema_version: string;
  file_count: number;
  total_bytes: number;
  message: string;
}

export interface RestoreResult {
  restored_backup_name: string;
  pre_restore_backup_name: string;
  restart_required: boolean;
  message: string;
}

export interface ScannerDevice {
  device_id: string;
  name: string;
  manufacturer: string | null;
  connection_type: string | null;
  is_available: boolean;
  status: string | null;
}

export interface PrinterDevice {
  printer_id: string;
  name: string;
  is_default: boolean;
  status: string;
  is_available: boolean;
  is_network: boolean;
}

export interface PrintResult {
  document_id: number;
  printer_name: string;
  copies: number;
  status: string;
}

export interface DeviceSettings {
  default_scanner_id: string | null;
  default_printer_id: string | null;
  scan_default_dpi: number;
  scan_default_color_mode: 'color' | 'grayscale' | 'black_white';
  scan_default_output_format: 'png' | 'jpg';
  device_detection_last_checked_at: string | null;
}

export interface ScannerCapabilities {
  scanner_id: string;
  is_available: boolean;
  status: string;
  supports_flatbed: boolean;
  supports_adf: boolean;
  supported_dpi: number[];
  supported_color_modes: Array<'color' | 'grayscale' | 'black_white'>;
  supported_output_formats: Array<'png' | 'jpg'>;
}

export interface ScanOptions {
  dpi: number;
  color_mode: 'color' | 'grayscale' | 'black_white';
  output_format: 'png' | 'jpg';
  source: 'flatbed' | 'adf';
}

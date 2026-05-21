import { invoke } from '@tauri-apps/api/core';
import type {
  CategoryItem,
  AuditLogPage,
  AuditRetentionSettings,
  AttachmentPreviewInfo,
  AttachmentPreviewPage,
  BackupSettings,
  BackupSummary,
  BackupValidation,
  DeviceSettings,
  DocumentDetail,
  DocumentItem,
  DocumentStatus,
  FolderItem,
  CreatedMobileDevice,
  MobileReviewStatus,
  MobileApiSetup,
  MobileDeviceItem,
  MobileSubmissionDetail,
  MobileSubmissionItem,
  OfficeItem,
  PrinterDevice,
  PrintResult,
  ProfileItem,
  Role,
  ScanOptions,
  ScannerCapabilities,
  ScannerDevice,
  ScanIntakeItem,
  ScanIntakePreviewPage,
  SessionPayload,
  RestoreResult,
  UserItem
} from '../types';

export const firstRunCheck = (): Promise<boolean> => invoke('first_run_check');

export const firstRunSetup = (params: {
  firstName: string;
  lastName: string;
  username: string;
  password: string;
}): Promise<void> => invoke('first_run_setup', params);

export const login = (username: string, password: string): Promise<SessionPayload> =>
  invoke('login', { username, password });

export const logout = (sessionId: string): Promise<void> =>
  invoke('logout', { sessionId });

export const validateSession = (sessionId: string): Promise<SessionPayload> =>
  invoke('validate_session', { sessionId });

export const listCategories = (
  sessionId: string,
  includeInactive = true
): Promise<CategoryItem[]> => invoke('list_categories', { sessionId, includeInactive });

export const createCategory = (params: {
  sessionId: string;
  categoryName: string;
  description: string | null;
  colorCode: string;
  icon: string | null;
}): Promise<number> => invoke('create_category', params);

export const updateCategory = (params: {
  sessionId: string;
  categoryId: number;
  categoryName: string;
  description: string | null;
  colorCode: string;
  icon: string | null;
  isActive: boolean;
}): Promise<void> => invoke('update_category', params);

export const listFolders = (
  sessionId: string,
  categoryId: number | null = null,
  includeInactive = true
): Promise<FolderItem[]> => invoke('list_folders', { sessionId, categoryId, includeInactive });

export const createFolder = (params: {
  sessionId: string;
  categoryId: number;
  folderName: string;
  description: string | null;
  folderColor: string;
}): Promise<number> => invoke('create_folder', params);

export const updateFolder = (params: {
  sessionId: string;
  folderId: number;
  categoryId: number;
  folderName: string;
  description: string | null;
  folderColor: string;
  isActive: boolean;
}): Promise<void> => invoke('update_folder', params);

export const listOffices = (
  sessionId: string,
  includeInactive = true
): Promise<OfficeItem[]> => invoke('list_offices', { sessionId, includeInactive });

export const createOffice = (params: {
  sessionId: string;
  officeName: string;
  description: string | null;
}): Promise<number> => invoke('create_office', params);

export const updateOffice = (params: {
  sessionId: string;
  officeId: number;
  officeName: string;
  description: string | null;
  isActive: boolean;
}): Promise<void> => invoke('update_office', params);

export const listUsers = (sessionId: string, search: string | null = null): Promise<UserItem[]> =>
  invoke('list_users', { sessionId, search });

export const createUser = (params: {
  sessionId: string;
  role: Role;
  firstName: string;
  middleName: string | null;
  lastName: string;
  username: string;
  email: string | null;
  contactNumber: string | null;
  address: string | null;
  password: string;
}): Promise<number> => invoke('create_user', params);

export const updateUser = (params: {
  sessionId: string;
  userId: number;
  role: Role;
  firstName: string;
  middleName: string | null;
  lastName: string;
  username: string;
  email: string | null;
  contactNumber: string | null;
  address: string | null;
  isActive: boolean;
}): Promise<void> => invoke('update_user', params);

export const adminResetPassword = (params: {
  sessionId: string;
  userId: number;
  newPassword: string;
}): Promise<void> => invoke('admin_reset_password', params);

export const getMyProfile = (sessionId: string): Promise<ProfileItem> =>
  invoke('get_my_profile', { sessionId });

export const updateMyProfile = (params: {
  sessionId: string;
  firstName: string;
  middleName: string | null;
  lastName: string;
  email: string | null;
  contactNumber: string | null;
  address: string | null;
}): Promise<void> => invoke('update_my_profile', params);

export const changeMyPassword = (params: {
  sessionId: string;
  currentPassword: string;
  newPassword: string;
}): Promise<void> => invoke('change_my_password', params);

export const createDocument = (params: {
  sessionId: string;
  documentName: string;
  categoryId: number;
  folderId: number | null;
  officeId: number | null;
  dateReceived: string;
  remarks: string | null;
  status: DocumentStatus;
}): Promise<number> => invoke('create_document', params);

export const updateDocument = (params: {
  sessionId: string;
  documentId: number;
  documentName: string;
  categoryId: number;
  folderId: number | null;
  officeId: number | null;
  dateReceived: string;
  remarks: string | null;
  status: DocumentStatus;
}): Promise<void> => invoke('update_document', params);

export const moveDocument = (params: {
  sessionId: string;
  documentId: number;
  categoryId: number;
  folderId: number | null;
}): Promise<void> => invoke('move_document', params);

export const setDocumentStatus = (params: {
  sessionId: string;
  documentId: number;
  status: DocumentStatus;
}): Promise<void> => invoke('set_document_status', params);

export const setDocumentHidden = (params: {
  sessionId: string;
  documentId: number;
  isHidden: boolean;
}): Promise<void> => invoke('set_document_hidden', params);

export const trashDocument = (params: {
  sessionId: string;
  documentId: number;
}): Promise<void> => invoke('trash_document', params);

export const restoreDocument = (params: {
  sessionId: string;
  documentId: number;
}): Promise<void> => invoke('restore_document', params);

export const listTrashDocuments = (sessionId: string): Promise<DocumentItem[]> =>
  invoke('list_trash_documents', { sessionId });

export const purgeDocument = (params: {
  sessionId: string;
  documentId: number;
}): Promise<void> => invoke('purge_document', params);

export const emptyTrash = (sessionId: string): Promise<number> =>
  invoke('empty_trash', { sessionId });

export const listDocuments = (params: {
  sessionId: string;
  search?: string | null;
  categoryId?: number | null;
  folderId?: number | null;
  officeId?: number | null;
  dateFrom?: string | null;
  dateTo?: string | null;
}): Promise<DocumentItem[]> => invoke('list_documents', params);

export const getDocument = (sessionId: string, documentId: number): Promise<DocumentDetail> =>
  invoke('get_document', { sessionId, documentId });

export const addAttachment = (params: {
  sessionId: string;
  documentId: number;
  sourcePath: string;
  sortOrder?: number | null;
}): Promise<number> => invoke('add_attachment', params);

export const removeAttachment = (params: {
  sessionId: string;
  attachmentId: number;
}): Promise<void> => invoke('remove_attachment', params);

export const reorderAttachments = (params: {
  sessionId: string;
  documentId: number;
  attachmentIds: number[];
}): Promise<void> => invoke('reorder_attachments', params);

export const getAttachmentFilePath = (
  attachmentId: number,
  sessionId: string | null = null
): Promise<string> => invoke('get_attachment_file_path', { attachmentId, sessionId });

export const getAttachmentPreviewInfo = (
  attachmentId: number,
  sessionId: string | null = null
): Promise<AttachmentPreviewInfo> =>
  invoke('get_attachment_preview_info', { attachmentId, sessionId });

export const getAttachmentPreviewPage = (params: {
  attachmentId: number;
  sessionId?: string | null;
  pageNumber?: number | null;
}): Promise<AttachmentPreviewPage> => invoke('get_attachment_preview_page', params);

export const exportDocumentPdf = (params: {
  documentId: number;
  outputPath: string;
  sessionId?: string | null;
}): Promise<string> => invoke('export_document_pdf', params);

export const listPublicCategories = (): Promise<CategoryItem[]> => invoke('list_public_categories');

export const listPublicFolders = (categoryId: number): Promise<FolderItem[]> =>
  invoke('list_public_folders', { categoryId });

export const listPublicDocuments = (params: {
  search?: string | null;
  categoryId?: number | null;
  folderId?: number | null;
  officeId?: number | null;
  dateFrom?: string | null;
  dateTo?: string | null;
} = {}): Promise<DocumentItem[]> => invoke('list_public_documents', params);

export const getPublicDocument = (documentId: number): Promise<DocumentDetail> =>
  invoke('get_public_document', { documentId });

export const listDocumentOffices = (sessionId: string): Promise<OfficeItem[]> =>
  invoke('list_document_offices', { sessionId });

export const importScanFiles = (params: {
  sessionId: string;
  sourcePaths: string[];
}): Promise<number[]> => invoke('import_scan_files', params);

export const listScanIntake = (sessionId: string): Promise<ScanIntakeItem[]> =>
  invoke('list_scan_intake', { sessionId });

export const getScanIntake = (
  sessionId: string,
  scanIntakeId: number
): Promise<ScanIntakeItem> => invoke('get_scan_intake', { sessionId, scanIntakeId });

export const getScanIntakePreviewPage = (params: {
  sessionId: string;
  scanIntakeId: number;
  pageNumber?: number | null;
}): Promise<ScanIntakePreviewPage> => invoke('get_scan_intake_preview_page', params);

export const updateScanIntakeNotes = (params: {
  sessionId: string;
  scanIntakeId: number;
  notes: string | null;
}): Promise<void> => invoke('update_scan_intake_notes', params);

export const removeScanIntake = (params: {
  sessionId: string;
  scanIntakeId: number;
}): Promise<void> => invoke('remove_scan_intake', params);

export const fileScanAsDocument = (params: {
  sessionId: string;
  scanIntakeIds: number[];
  documentName: string;
  categoryId: number;
  folderId: number | null;
  officeId: number | null;
  dateReceived: string;
  remarks: string | null;
  status: DocumentStatus;
}): Promise<number> => invoke('file_scan_as_document', params);

export const attachScanToDocument = (params: {
  sessionId: string;
  scanIntakeIds: number[];
  documentId: number;
}): Promise<number[]> => invoke('attach_scan_to_document', params);

export const listAuditLogs = (params: {
  sessionId: string;
  search?: string | null;
  actorUserId?: number | null;
  actorSearch?: string | null;
  action?: string | null;
  entityType?: string | null;
  dateFrom?: string | null;
  dateTo?: string | null;
  limit?: number | null;
  offset?: number | null;
}): Promise<AuditLogPage> => invoke('list_audit_logs', params);

export const listMyActivity = (params: {
  sessionId: string;
  search?: string | null;
  action?: string | null;
  entityType?: string | null;
  dateFrom?: string | null;
  dateTo?: string | null;
  limit?: number | null;
  offset?: number | null;
}): Promise<AuditLogPage> => invoke('list_my_activity', params);

export const listAuditEventTypes = (sessionId: string): Promise<string[]> =>
  invoke('list_audit_event_types', { sessionId });

export const listMyActivityEventTypes = (sessionId: string): Promise<string[]> =>
  invoke('list_my_activity_event_types', { sessionId });

export const getAuditRetentionSettings = (sessionId: string): Promise<AuditRetentionSettings> =>
  invoke('get_audit_retention_settings', { sessionId });

export const updateAuditRetentionSettings = (params: {
  sessionId: string;
  retentionMonths: number;
}): Promise<AuditRetentionSettings> => invoke('update_audit_retention_settings', params);

export const getBackupSettings = (sessionId: string): Promise<BackupSettings> =>
  invoke('get_backup_settings', { sessionId });

export const updateBackupSettings = (params: {
  sessionId: string;
  destinationPath: string | null;
  scheduleEnabled: boolean;
  scheduleTime: string;
  retentionCount: number;
}): Promise<BackupSettings> => invoke('update_backup_settings', params);

export const createBackup = (sessionId: string): Promise<BackupSummary> =>
  invoke('create_backup', { sessionId });

export const listBackupHistory = (sessionId: string): Promise<BackupSummary[]> =>
  invoke('list_backup_history', { sessionId });

export const exportBackupArchive = (params: {
  sessionId: string;
  backupName: string;
  outputPath: string;
}): Promise<string> => invoke('export_backup_archive', params);

export const validateBackupArchive = (params: {
  sessionId: string;
  archivePath: string;
}): Promise<BackupValidation> => invoke('validate_backup_archive', params);

export const importBackupArchive = (params: {
  sessionId: string;
  archivePath: string;
}): Promise<BackupSummary> => invoke('import_backup_archive', params);

export const restoreFromBackup = (params: {
  sessionId: string;
  backupName: string;
}): Promise<RestoreResult> => invoke('restore_from_backup', params);

export const restoreFromBackupFolder = (params: {
  sessionId: string;
  folderPath: string;
}): Promise<RestoreResult> => invoke('restore_from_backup_folder', params);

export const runScheduledBackupCheck = (sessionId: string): Promise<BackupSummary | null> =>
  invoke('run_scheduled_backup_check', { sessionId });

export const listScanners = (sessionId: string): Promise<ScannerDevice[]> =>
  invoke('list_scanners', { sessionId });

export const listPrinters = (sessionId: string): Promise<PrinterDevice[]> =>
  invoke('list_printers', { sessionId });

export const getDefaultPrinter = (sessionId: string): Promise<PrinterDevice | null> =>
  invoke('get_default_printer', { sessionId });

export const getDeviceSettings = (sessionId: string): Promise<DeviceSettings> =>
  invoke('get_device_settings', { sessionId });

export const updateDeviceSettings = (params: {
  sessionId: string;
  defaultScannerId: string | null;
  defaultPrinterId: string | null;
  scanDefaultDpi: number;
  scanDefaultColorMode: DeviceSettings['scan_default_color_mode'];
  scanDefaultOutputFormat: DeviceSettings['scan_default_output_format'];
}): Promise<DeviceSettings> => invoke('update_device_settings', params);

export const listPrintPrinters = (sessionId: string | null): Promise<PrinterDevice[]> =>
  invoke('list_print_printers', { sessionId });

export const printDocumentPdf = (params: {
  sessionId: string | null;
  documentId: number;
  printerId: string;
  copies: number;
}): Promise<PrintResult> => invoke('print_document_pdf', params);

export const getScannerCapabilities = (params: {
  sessionId: string;
  scannerId: string;
}): Promise<ScannerCapabilities> => invoke('get_scanner_capabilities', params);

export const scanToIntake = (params: {
  sessionId: string;
  scannerId: string;
  options: ScanOptions;
}): Promise<ScanIntakeItem> => invoke('scan_to_intake', params);

export const listMobileSubmissions = (params: {
  sessionId: string;
  reviewStatus?: MobileReviewStatus | null;
  search?: string | null;
  dateFrom?: string | null;
  dateTo?: string | null;
}): Promise<MobileSubmissionItem[]> => invoke('list_mobile_submissions', params);

export const getMobileApiSetup = (): Promise<MobileApiSetup> => invoke('get_mobile_api_setup');

export const createMobileDevice = (params: {
  sessionId: string;
  deviceName: string;
}): Promise<CreatedMobileDevice> => invoke('create_mobile_device', params);

export const listMobileDevices = (sessionId: string): Promise<MobileDeviceItem[]> =>
  invoke('list_mobile_devices', { sessionId });

export const revokeMobileDevice = (params: {
  sessionId: string;
  deviceId: string;
}): Promise<void> => invoke('revoke_mobile_device', params);

export const getMobileSubmission = (params: {
  sessionId: string;
  mobileSubmissionId: number;
}): Promise<MobileSubmissionDetail> => invoke('get_mobile_submission', params);

export const approveMobileSubmission = (params: {
  sessionId: string;
  mobileSubmissionId: number;
  reviewNotes?: string | null;
}): Promise<number> => invoke('approve_mobile_submission', params);

export const rejectMobileSubmission = (params: {
  sessionId: string;
  mobileSubmissionId: number;
  rejectionReason: string;
}): Promise<void> => invoke('reject_mobile_submission', params);

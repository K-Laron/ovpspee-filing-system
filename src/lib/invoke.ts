import { invoke } from '@tauri-apps/api/core';
import type {
  CategoryItem,
  DocumentDetail,
  DocumentItem,
  DocumentStatus,
  FolderItem,
  OfficeItem,
  ProfileItem,
  Role,
  ScanIntakeItem,
  SessionPayload,
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

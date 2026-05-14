import { invoke } from '@tauri-apps/api/core';
import type { CategoryItem, FolderItem, OfficeItem, ProfileItem, Role, SessionPayload, UserItem } from '../types';

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

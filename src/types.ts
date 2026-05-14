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

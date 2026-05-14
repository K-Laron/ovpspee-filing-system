import { create } from 'zustand';
import type { Role, SessionPayload } from '../types';

interface SessionState {
  sessionId: string | null;
  userId: number | null;
  role: Role | null;
  displayName: string;
  profilePicPath: string | null;
  setSession: (payload: SessionPayload) => void;
  clearSession: () => void;
}

export const useSessionStore = create<SessionState>((set) => ({
  sessionId: null,
  userId: null,
  role: null,
  displayName: '',
  profilePicPath: null,
  setSession: (payload) =>
    set({
      sessionId: payload.session_id,
      userId: payload.user_id,
      role: payload.role,
      displayName: payload.display_name,
      profilePicPath: payload.profile_pic_path
    }),
  clearSession: () =>
    set({
      sessionId: null,
      userId: null,
      role: null,
      displayName: '',
      profilePicPath: null
    })
}));

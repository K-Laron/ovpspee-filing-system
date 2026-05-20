import type { MobileSubmissionDraft } from '../types';

const DRAFT_KEY = 'ovpspee.mobileSubmissionDraft.v1';
const HUB_URL_KEY = 'ovpspee.mobileHubUrl.v1';
const memoryStore = new Map<string, string>();

const setItem = async (key: string, value: string): Promise<void> => {
  memoryStore.set(key, value);
};

const getItem = async (key: string): Promise<string | null> => {
  return memoryStore.get(key) ?? null;
};

const removeItem = async (key: string): Promise<void> => {
  memoryStore.delete(key);
};

export const saveDraft = async (draft: MobileSubmissionDraft): Promise<void> => {
  await setItem(DRAFT_KEY, JSON.stringify(draft));
};

export const loadDraft = async (): Promise<MobileSubmissionDraft | null> => {
  const value = await getItem(DRAFT_KEY);
  if (!value) return null;
  return JSON.parse(value) as MobileSubmissionDraft;
};

export const clearDraft = async (): Promise<void> => {
  await removeItem(DRAFT_KEY);
};

export const saveHubUrl = async (hubUrl: string): Promise<void> => {
  await setItem(HUB_URL_KEY, hubUrl);
};

export const loadHubUrl = async (): Promise<string | null> => {
  return getItem(HUB_URL_KEY);
};

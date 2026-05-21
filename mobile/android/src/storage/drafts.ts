import { NativeModules, Platform } from 'react-native';

import type { DeviceProfile, MobileSubmissionDraft, QueuedSubmission } from '../types';

const DRAFT_KEY = 'ovpspee.mobileSubmissionDraft.v1';
const HUB_URL_KEY = 'ovpspee.mobileHubUrl.v1';
const DEVICE_PROFILE_KEY = 'ovpspee.mobileDeviceProfile.v1';
const QUEUE_KEY = 'ovpspee.mobileSubmissionQueue.v1';
const memoryStore = new Map<string, string>();

type NativeStorageModule = {
  setItem(key: string, value: string): Promise<void>;
  getItem(key: string): Promise<string | null>;
  removeItem(key: string): Promise<void>;
};

const nativeStorage = NativeModules.OvpspeeStorage as NativeStorageModule | undefined;

const setItem = async (key: string, value: string): Promise<void> => {
  if (nativeStorage) {
    await nativeStorage.setItem(key, value);
    return;
  }
  memoryStore.set(key, value);
};

const getItem = async (key: string): Promise<string | null> => {
  if (nativeStorage) {
    return nativeStorage.getItem(key);
  }
  return memoryStore.get(key) ?? null;
};

const removeItem = async (key: string): Promise<void> => {
  if (nativeStorage) {
    await nativeStorage.removeItem(key);
    return;
  }
  memoryStore.delete(key);
};

const parseJson = <T>(value: string | null, fallback: T): T => {
  if (!value) return fallback;
  try {
    return JSON.parse(value) as T;
  } catch {
    return fallback;
  }
};

const randomId = (): string =>
  `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;

export const newClientSubmissionId = (): string => `mobile-${randomId()}`;

export const defaultDeviceName = (): string =>
  Platform.OS === 'android' ? 'Android office phone' : 'Office mobile device';

export const saveDraft = async (draft: MobileSubmissionDraft): Promise<void> => {
  await setItem(DRAFT_KEY, JSON.stringify(draft));
};

export const loadDraft = async (): Promise<MobileSubmissionDraft | null> => {
  const value = await getItem(DRAFT_KEY);
  if (!value) return null;
  return parseJson<MobileSubmissionDraft | null>(value, null);
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

export const loadDeviceProfile = async (): Promise<DeviceProfile> => {
  const stored = parseJson<Partial<DeviceProfile>>(await getItem(DEVICE_PROFILE_KEY), {});
  const profile: DeviceProfile = {
    deviceId: stored.deviceId?.trim() || `device-${randomId()}`,
    deviceName: stored.deviceName?.trim() || defaultDeviceName(),
    deviceToken: stored.deviceToken ?? ''
  };
  await saveDeviceProfile(profile);
  return profile;
};

export const saveDeviceProfile = async (profile: DeviceProfile): Promise<void> => {
  await setItem(DEVICE_PROFILE_KEY, JSON.stringify(profile));
};

export const loadQueuedSubmissions = async (): Promise<QueuedSubmission[]> => {
  return parseJson<QueuedSubmission[]>(await getItem(QUEUE_KEY), []);
};

export const saveQueuedSubmission = async (submission: QueuedSubmission): Promise<void> => {
  const queue = await loadQueuedSubmissions();
  const next = [
    submission,
    ...queue.filter((item) => item.clientSubmissionId !== submission.clientSubmissionId)
  ];
  await setItem(QUEUE_KEY, JSON.stringify(next));
};

export const removeQueuedSubmission = async (clientSubmissionId: string): Promise<void> => {
  const queue = await loadQueuedSubmissions();
  await setItem(
    QUEUE_KEY,
    JSON.stringify(queue.filter((item) => item.clientSubmissionId !== clientSubmissionId))
  );
};

export const markQueuedSubmissionAttempt = async (
  clientSubmissionId: string,
  lastError: string
): Promise<void> => {
  const queue = await loadQueuedSubmissions();
  const now = new Date().toISOString();
  await setItem(
    QUEUE_KEY,
    JSON.stringify(
      queue.map((item) =>
        item.clientSubmissionId === clientSubmissionId
          ? { ...item, attempts: item.attempts + 1, lastError, lastAttemptAt: now }
          : item
      )
    )
  );
};

export const clearQueuedSubmissions = async (): Promise<void> => {
  await removeItem(QUEUE_KEY);
};

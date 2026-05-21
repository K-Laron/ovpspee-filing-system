import { NativeModules } from 'react-native';

import type { MobileAttachmentDraft } from '../types';

interface NativePickedFile {
  uri: string;
  name: string;
  mimeType: string;
  sizeBytes?: number;
}

interface CaptureModule {
  pickFile(): Promise<NativePickedFile>;
  capturePhoto(): Promise<NativePickedFile>;
}

const captureModule = NativeModules.OvpspeeCapture as CaptureModule | undefined;

export const normalizePickedFile = (file: NativePickedFile): MobileAttachmentDraft => ({
  uri: file.uri,
  name: file.name,
  type: file.mimeType,
  sizeBytes: file.sizeBytes
});

export const pickFile = async (): Promise<MobileAttachmentDraft> => {
  if (!captureModule) {
    throw new Error('Android file picker is unavailable.');
  }
  return normalizePickedFile(await captureModule.pickFile());
};

export const capturePhoto = async (): Promise<MobileAttachmentDraft> => {
  if (!captureModule) {
    throw new Error('Android camera is unavailable.');
  }
  return normalizePickedFile(await captureModule.capturePhoto());
};

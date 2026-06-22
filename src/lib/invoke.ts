import { invoke } from '@tauri-apps/api/core';

export const cmd = <T>(name: string, args?: Record<string, unknown>): Promise<T> => invoke(name, args);

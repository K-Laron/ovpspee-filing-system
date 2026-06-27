const technicalPatterns = [
  /\b[A-Z]:[\\/][^\s]+/i,
  /src-tauri/i,
  /\bSQLITE_/i,
  /\bUNIQUE constraint failed\b/i,
  /\bFOREIGN KEY constraint failed\b/i,
  /\bdatabase (?:is )?locked\b/i,
  /\bdatabase disk image is malformed\b/i,
  /\bno such table\b/i,
  /\binvoke\b/i,
  /\bcommand\b/i,
  /\bthread\b/i,
  /\bpanic\b/i,
  /\bstack\b/i,
  /\.rs:\d+/i,
  /\.tsx?:\d+/i,
  /Error:.*failed at/i,
];

const safePrefixes = [
  'Password',
  'Username',
  'Login',
  'File',
  'Folder',
  'Category',
  'Office',
  'Document',
  'Backup',
  'Scanner',
  'Printer',
  'Required',
  'Invalid',
  'Could not',
];

const safeBackendPrefixPattern = /^ERR_(?:VALIDATION|DUPLICATE):\s*/;

export const getErrorMessage = (error: unknown, fallback: string): string => {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  return fallback;
};

export const getUserErrorMessage = (error: unknown, fallback: string): string => {
  const raw = getErrorMessage(error, fallback).trim();
  if (!raw) return fallback;
  const normalized = raw.replace(safeBackendPrefixPattern, '').trim();
  if (technicalPatterns.some((pattern) => pattern.test(normalized))) return fallback;
  if (safePrefixes.some((prefix) => normalized.startsWith(prefix))) return normalized;
  return fallback;
};

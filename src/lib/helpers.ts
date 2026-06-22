export const nullable = (value: string) => {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
};

export const fileNameFromPath = (path: string) => path.split(/[\\/]/).pop() ?? path;

export const normalizeSelectedPaths = (selected: string | string[] | null) => {
  if (!selected) return [];
  if (Array.isArray(selected)) return selected;
  return selected.split('|');
};

export const safeFileName = (value: string) => value.replace(/[<>:"/\\|?*]+/g, '-').slice(0, 80) || 'document';

// ponytail: KB-only, fine for scan previews where files are small
export const sizeLabel = (bytes: number) => `${Math.ceil(bytes / 1024)} KB`;

export const extensionFromName = (name: string) => name.split('.').pop()?.toLowerCase() ?? 'unknown';

export const formatBytes = (value: number) => {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
};

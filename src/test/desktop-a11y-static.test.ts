import { readFileSync } from 'node:fs';

import { describe, expect, it } from 'vitest';

const read = (path: string) => readFileSync(path, 'utf8');

const buttonTags = (source: string) => source.match(/<button[\s\S]*?>/g) ?? [];
const clickableRowBlocks = (source: string) =>
  (source.match(/<tr[\s\S]*?<\/tr>/g) ?? []).filter((block) => block.includes('cursor-pointer'));

describe('desktop accessibility source checks', () => {
  it('does not use the obsolete label utility class', () => {
    expect(read('src/pages/admin/BackupRestore.tsx')).not.toContain('className="label"');
  });

  it('gives icon-only buttons accessible names', () => {
    const files = [
      'src/components/AttachmentPreview.tsx',
      'src/pages/secretary/AddDocument.tsx',
      'src/pages/secretary/Documents.tsx',
      'src/pages/secretary/MobileSubmissions.tsx',
      'src/pages/secretary/ScanIntake.tsx'
    ];

    const unnamedIconButtons = files.flatMap((file) =>
      buttonTags(read(file))
        .filter((tag) => tag.includes('className="icon-btn'))
        .filter((tag) => !tag.includes('aria-label='))
        .map((tag) => `${file}: ${tag}`)
    );

    expect(unnamedIconButtons).toEqual([]);
  });

  it('keeps clickable document table rows keyboard accessible', () => {
    const files = ['src/pages/GuestLanding.tsx', 'src/pages/secretary/Documents.tsx'];

    const inaccessibleRows = files.flatMap((file) =>
      clickableRowBlocks(read(file))
        .filter((block) => !block.includes('role="button"') || !block.includes('tabIndex={0}') || !block.includes('onKeyDown='))
        .map((block) => `${file}: ${block}`)
    );

    expect(inaccessibleRows).toEqual([]);
  });
});

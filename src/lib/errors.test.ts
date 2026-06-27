import { describe, expect, it } from 'vitest';

import { getErrorMessage, getUserErrorMessage } from './errors';

describe('getUserErrorMessage', () => {
  it('keeps known plain validation messages readable', () => {
    expect(getUserErrorMessage('Password must be at least 8 characters.', 'Fallback.')).toBe(
      'Password must be at least 8 characters.',
    );
  });

  it('keeps safe ERR_VALIDATION messages readable', () => {
    expect(
      getUserErrorMessage('ERR_VALIDATION: Password must be at least 8 characters.', 'Fallback.'),
    ).toBe('Password must be at least 8 characters.');
  });

  it('keeps safe ERR_DUPLICATE messages readable', () => {
    expect(getUserErrorMessage('ERR_DUPLICATE: Category already exists.', 'Fallback.')).toBe(
      'Category already exists.',
    );
  });

  it('hides unsafe ERR_DB messages', () => {
    expect(getUserErrorMessage('ERR_DB: database locked', 'Fallback.')).toBe('Fallback.');
  });

  it('hides unsafe ERR_IO messages', () => {
    expect(getUserErrorMessage('ERR_IO: Could not read C:\\secret', 'Fallback.')).toBe('Fallback.');
  });

  it('trims safe messages', () => {
    expect(getUserErrorMessage('  Password must be at least 8 characters.  ', 'Fallback.')).toBe(
      'Password must be at least 8 characters.',
    );
  });

  it('hides unsafe nontechnical plain strings', () => {
    expect(getUserErrorMessage('Something went sideways', 'Fallback.')).toBe('Fallback.');
  });

  it('hides technical invoke and stack-like messages', () => {
    expect(
      getUserErrorMessage(
        'Error: command scan_to_intake failed at src-tauri\\src\\main.rs:44',
        'Could not scan.',
      ),
    ).toBe('Could not scan.');
  });

  it('hides Windows path leaks', () => {
    expect(
      getUserErrorMessage(
        'Could not save C:\\Users\\Kenneth\\Desktop\\file.pdf',
        'Could not save.',
      ),
    ).toBe('Could not save.');
  });

  it('hides SQLite constraint leaks', () => {
    expect(
      getUserErrorMessage(
        'Could not save. UNIQUE constraint failed: documents.path',
        'Could not save.',
      ),
    ).toBe('Could not save.');
  });

  it('hides database locked leaks', () => {
    expect(getUserErrorMessage('Could not save: database locked', 'Could not save.')).toBe(
      'Could not save.',
    );
  });

  it('hides unknown object errors', () => {
    expect(getUserErrorMessage({ code: 'SQLITE_BUSY' }, 'Could not save.')).toBe('Could not save.');
  });

  it('keeps legacy getErrorMessage behavior for compatibility', () => {
    expect(getErrorMessage(new Error('Login failed.'), 'Fallback.')).toBe('Login failed.');
  });
});

import { describe, expect, it } from 'vitest';

import { formatDateInputValue, formatDateOnly, formatDateTime } from './dates';

describe('formatDateOnly', () => {
  it('formats yyyy-mm-dd values for non-technical users', () => {
    expect(formatDateOnly('2026-05-17')).toBe('May 17, 2026');
  });

  it('returns Unknown for empty values', () => {
    expect(formatDateOnly('')).toBe('Unknown');
    expect(formatDateOnly(null)).toBe('Unknown');
    expect(formatDateOnly(undefined)).toBe('Unknown');
  });

  it('returns invalid values unchanged so bad source data stays visible', () => {
    expect(formatDateOnly('2026-02-30')).toBe('2026-02-30');
    expect(formatDateOnly('not-a-date')).toBe('not-a-date');
  });
});

describe('formatDateTime', () => {
  it('formats timestamps without seconds', () => {
    expect(formatDateTime('2026-05-17T12:30:45Z')).toBe('May 17, 2026, 8:30 PM');
  });

  it('returns Unknown for empty values', () => {
    expect(formatDateTime('')).toBe('Unknown');
    expect(formatDateTime(null)).toBe('Unknown');
    expect(formatDateTime(undefined)).toBe('Unknown');
  });

  it('returns invalid values unchanged so bad source data stays visible', () => {
    expect(formatDateTime('not-a-timestamp')).toBe('not-a-timestamp');
  });
});

describe('formatDateInputValue', () => {
  it('formats local Manila date values for date inputs', () => {
    expect(formatDateInputValue(new Date('2026-05-17T16:30:00.000Z'))).toBe('2026-05-18');
  });
});

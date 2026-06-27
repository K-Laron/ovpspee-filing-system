import { describe, expect, it } from 'vitest';

import { validatePasswordPair } from './passwords';

describe('validatePasswordPair', () => {
  it('requires at least 8 characters', () => {
    expect(validatePasswordPair('Pass1!', 'Pass1!')).toBe(
      'Password must be at least 8 characters.',
    );
  });

  it('counts surrogate pairs as single password characters', () => {
    expect(validatePasswordPair('Aa1!😀😀😀', 'Aa1!😀😀😀')).toBe(
      'Password must be at least 8 characters.',
    );
  });

  it('requires at least 1 number', () => {
    expect(validatePasswordPair('Password!', 'Password!')).toBe(
      'Password must include at least 1 number.',
    );
  });

  it('requires at least 1 special character', () => {
    expect(validatePasswordPair('Password1', 'Password1')).toBe(
      'Password must include at least 1 special character.',
    );
  });

  it('does not treat whitespace as a special character', () => {
    expect(validatePasswordPair('Password1 ', 'Password1 ')).toBe(
      'Password must include at least 1 special character.',
    );
  });

  it('requires matching passwords', () => {
    expect(validatePasswordPair('Password1!', 'Password2!')).toBe('Passwords do not match.');
  });

  it('accepts a valid matching pair', () => {
    expect(validatePasswordPair('Password1!', 'Password1!')).toBeNull();
  });
});

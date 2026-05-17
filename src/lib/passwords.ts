export const passwordRulesText =
  'Minimum 8 characters with at least 1 number and 1 special character.';

export const validatePasswordPair = (
  password: string,
  confirmPassword: string
): string | null => {
  if (Array.from(password).length < 8) return 'Password must be at least 8 characters.';
  if (!/\d/.test(password)) return 'Password must include at least 1 number.';
  if (!/[!@#$%^&*()\-_=+\[\]{}|;:,.<>?]/.test(password)) {
    return 'Password must include at least 1 special character.';
  }
  if (password !== confirmPassword) return 'Passwords do not match.';
  return null;
};

import { FormEvent, useState } from 'react';
import { ShieldCheck } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

import { firstRunSetup, login } from '../lib/invoke';
import { getUserErrorMessage } from '../lib/errors';
import { passwordRulesText, validatePasswordPair } from '../lib/passwords';
import { useSessionStore } from '../store/sessionStore';

export const FirstRunSetup = () => {
  const navigate = useNavigate();
  const setSession = useSessionStore((state) => state.setSession);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError(null);
    setSubmitting(true);
    const data = new FormData(event.currentTarget);
    const username = String(data.get('username') ?? '');
    const password = String(data.get('password') ?? '');
    const confirmPassword = String(data.get('confirmPassword') ?? '');
    const validationError = validatePasswordPair(password, confirmPassword);
    if (validationError) {
      setError(validationError);
      setSubmitting(false);
      return;
    }

    try {
      await firstRunSetup({
        firstName: String(data.get('firstName') ?? ''),
        lastName: String(data.get('lastName') ?? ''),
        username,
        password
      });
      const session = await login(username, password);
      setSession(session);
      navigate('/a', { replace: true });
    } catch (err) {
      setError(getUserErrorMessage(err, 'First-run setup failed.'));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="flex h-screen items-center justify-center bg-background p-6">
      <form className="w-full max-w-md rounded border border-border bg-surface p-6 shadow-sm" onSubmit={handleSubmit}>
        <div className="mb-6 flex items-center gap-3">
          <div className="flex h-11 w-11 items-center justify-center rounded bg-primary text-white">
            <ShieldCheck size={24} />
          </div>
          <div>
            <h1 className="text-xl font-bold text-secondary">First-Run Setup</h1>
            <p className="text-sm text-muted">Create initial Admin account for IT Staff.</p>
          </div>
        </div>
        <div className="space-y-4">
          <Field label="First name" name="firstName" autoComplete="given-name" />
          <Field label="Last name" name="lastName" autoComplete="family-name" />
          <Field label="Username" name="username" autoComplete="username" />
          <Field label="Password" name="password" type="password" autoComplete="new-password" />
          <Field label="Confirm password" name="confirmPassword" type="password" autoComplete="new-password" />
        </div>
        <p className="mt-3 text-xs text-muted">{passwordRulesText}</p>
        {error && <p className="mt-4 rounded border border-red-200 bg-red-50 p-3 text-sm text-red-700">{error}</p>}
        <button
          className="focus-ring mt-6 w-full rounded bg-primary px-4 py-2 text-sm font-semibold text-white hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-60"
          disabled={submitting}
          type="submit"
        >
          {submitting ? 'Creating Admin...' : 'Create Admin Account'}
        </button>
      </form>
    </div>
  );
};

interface FieldProps {
  label: string;
  name: string;
  type?: string;
  autoComplete: string;
}

const Field = ({ label, name, type = 'text', autoComplete }: FieldProps) => (
  <label className="block">
    <span className="mb-1 block text-sm font-medium text-secondary">{label}</span>
    <input
      autoComplete={autoComplete}
      className="focus-ring w-full rounded border border-border bg-white px-3 py-2 text-sm"
      name={name}
      required
      type={type}
    />
  </label>
);

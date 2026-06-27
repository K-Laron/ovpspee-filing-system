import { FormEvent, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';

import { getUserErrorMessage } from '../lib/errors';
import { invoke } from '@tauri-apps/api/core';
import type { SessionPayload } from '../types';
import { useSessionStore } from '../store/sessionStore';

export const Login = () => {
  const navigate = useNavigate();
  const setSession = useSessionStore((state) => state.setSession);
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError(null);
    setSubmitting(true);
    const data = new FormData(event.currentTarget);

    try {
      const session = await invoke<SessionPayload>('login', {
        username: String(data.get('username') ?? ''),
        password: String(data.get('password') ?? ''),
      });
      setSession(session);
      navigate(session.role === 'Admin' ? '/a' : '/s', { replace: true });
    } catch (err) {
      setError(getUserErrorMessage(err, 'Login failed.'));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="flex h-screen items-center justify-center bg-background p-6">
      <form
        className="w-full max-w-sm rounded border border-border bg-surface p-6 shadow-sm"
        onSubmit={handleSubmit}
      >
        <div className="mb-6">
          <img
            alt="UEP logo"
            className="mb-3 h-14 w-14 object-contain drop-shadow-sm"
            src="/uep-logo.png"
          />
          <h1 className="text-xl font-bold text-secondary">Login</h1>
          <p className="text-sm text-muted">Admin and Secretary accounts only.</p>
        </div>
        <label className="block">
          <span className="mb-1 block text-sm font-medium text-secondary">Username</span>
          <input
            autoComplete="username"
            className="focus-ring w-full rounded border border-border bg-white px-3 py-2 text-sm"
            name="username"
            required
          />
        </label>
        <label className="mt-4 block">
          <span className="mb-1 block text-sm font-medium text-secondary">Password</span>
          <input
            autoComplete="current-password"
            className="focus-ring w-full rounded border border-border bg-white px-3 py-2 text-sm"
            name="password"
            required
            type="password"
          />
        </label>
        {error && (
          <p className="mt-4 rounded border border-red-200 bg-red-50 p-3 text-sm text-red-700">
            {error}
          </p>
        )}
        <button className="btn btn-primary mt-6 w-full" disabled={submitting} type="submit">
          {submitting ? 'Signing in...' : 'Sign in'}
        </button>
        <Link className="mt-4 block text-center text-sm text-muted hover:text-primary" to="/">
          Continue as Staff/Head Viewer
        </Link>
      </form>
    </div>
  );
};

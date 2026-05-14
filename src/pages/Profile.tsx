import { KeyRound, RefreshCw, Save } from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';

import { changeMyPassword, getMyProfile, updateMyProfile } from '../lib/invoke';
import { getErrorMessage } from '../lib/errors';
import { useSessionStore } from '../store/sessionStore';
import type { ProfileItem } from '../types';

const emptyProfile = {
  firstName: '',
  middleName: '',
  lastName: '',
  email: '',
  contactNumber: '',
  address: ''
};

export const Profile = () => {
  const { sessionId, setSession } = useSessionStore();
  const [profile, setProfile] = useState<ProfileItem | null>(null);
  const [form, setForm] = useState(emptyProfile);
  const [passwords, setPasswords] = useState({ currentPassword: '', newPassword: '' });
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');

  const reload = async () => {
    if (!sessionId) return;
    setLoading(true);
    setError('');
    try {
      const next = await getMyProfile(sessionId);
      setProfile(next);
      setForm({
        firstName: next.first_name,
        middleName: next.middle_name ?? '',
        lastName: next.last_name,
        email: next.email ?? '',
        contactNumber: next.contact_number ?? '',
        address: next.address ?? ''
      });
    } catch (err) {
      setError(getErrorMessage(err, 'Could not load profile.'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void reload();
  }, [sessionId]);

  const submitProfile = async (event: FormEvent) => {
    event.preventDefault();
    if (!sessionId || saving) return;
    setSaving(true);
    setError('');
    setNotice('');
    try {
      await updateMyProfile({
        sessionId,
        firstName: form.firstName,
        middleName: nullable(form.middleName),
        lastName: form.lastName,
        email: nullable(form.email),
        contactNumber: nullable(form.contactNumber),
        address: nullable(form.address)
      });
      const next = await getMyProfile(sessionId);
      setProfile(next);
      setSession({
        session_id: sessionId,
        user_id: next.user_id,
        role: next.role,
        display_name: `${next.first_name} ${next.last_name}`,
        profile_pic_path: next.profile_pic_path
      });
      setNotice('Profile updated.');
    } catch (err) {
      setError(getErrorMessage(err, 'Profile update failed.'));
    } finally {
      setSaving(false);
    }
  };

  const submitPassword = async (event: FormEvent) => {
    event.preventDefault();
    if (!sessionId || saving) return;
    setSaving(true);
    setError('');
    setNotice('');
    try {
      await changeMyPassword({
        sessionId,
        currentPassword: passwords.currentPassword,
        newPassword: passwords.newPassword
      });
      setPasswords({ currentPassword: '', newPassword: '' });
      setNotice('Password changed.');
    } catch (err) {
      setError(getErrorMessage(err, 'Password change failed.'));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="mx-auto max-w-5xl space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold text-secondary">Profile</h1>
          <p className="text-sm text-muted">{profile ? `${profile.role} account: ${profile.username}` : 'Account details'}</p>
        </div>
        <button className="focus-ring inline-flex h-10 items-center gap-2 rounded border border-border bg-surface px-3 text-sm font-medium text-secondary hover:bg-background" disabled={loading} onClick={() => void reload()} type="button">
          <RefreshCw size={16} />
          Refresh
        </button>
      </div>

      {error && <div className="rounded border border-primary/30 bg-primary/10 px-4 py-3 text-sm text-primary">{error}</div>}
      {notice && <div className="rounded border border-success/30 bg-success/10 px-4 py-3 text-sm text-success">{notice}</div>}

      <section className="grid gap-5 lg:grid-cols-[1fr_20rem]">
        <form className="space-y-4 rounded border border-border bg-surface p-5" onSubmit={submitProfile}>
          <h2 className="text-base font-semibold text-secondary">Account Details</h2>
          <div className="grid gap-4 md:grid-cols-2">
            <TextField label="First name" value={form.firstName} onChange={(value) => setForm({ ...form, firstName: value })} required />
            <TextField label="Middle name" value={form.middleName} onChange={(value) => setForm({ ...form, middleName: value })} />
            <TextField label="Last name" value={form.lastName} onChange={(value) => setForm({ ...form, lastName: value })} required />
            <TextField label="Email" type="email" value={form.email} onChange={(value) => setForm({ ...form, email: value })} />
            <TextField label="Contact number" value={form.contactNumber} onChange={(value) => setForm({ ...form, contactNumber: value })} />
            <TextField label="Address" value={form.address} onChange={(value) => setForm({ ...form, address: value })} />
          </div>
          <button className="focus-ring inline-flex h-10 items-center gap-2 rounded bg-primary px-4 text-sm font-semibold text-white hover:bg-secondary disabled:opacity-60" disabled={saving || loading} type="submit">
            <Save size={16} />
            Save Profile
          </button>
        </form>

        <div className="space-y-4">
          <div className="rounded border border-border bg-surface p-5">
            <div className="mb-3 flex h-16 w-16 items-center justify-center rounded bg-secondary text-xl font-bold text-white">
              {profile ? `${profile.first_name[0] ?? ''}${profile.last_name[0] ?? ''}` : 'O'}
            </div>
            <p className="font-semibold text-secondary">{profile ? `${profile.first_name} ${profile.last_name}` : 'Loading...'}</p>
            <p className="text-sm text-muted">{profile?.email || 'No email'}</p>
            <p className="mt-3 text-xs text-muted">Profile picture upload deferred until safe app-data file validation is added.</p>
          </div>

          <form className="space-y-3 rounded border border-border bg-surface p-5" onSubmit={submitPassword}>
            <h2 className="text-base font-semibold text-secondary">Change Password</h2>
            <TextField label="Current password" type="password" value={passwords.currentPassword} onChange={(value) => setPasswords({ ...passwords, currentPassword: value })} required />
            <TextField label="New password" type="password" value={passwords.newPassword} onChange={(value) => setPasswords({ ...passwords, newPassword: value })} required />
            <button className="focus-ring inline-flex h-10 w-full items-center justify-center gap-2 rounded bg-secondary px-3 text-sm font-semibold text-white disabled:opacity-60" disabled={saving} type="submit">
              <KeyRound size={16} />
              Change Password
            </button>
          </form>
        </div>
      </section>
    </div>
  );
};

const nullable = (value: string) => {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
};

const TextField = ({ label, onChange, required = false, type = 'text', value }: { label: string; onChange: (value: string) => void; required?: boolean; type?: string; value: string }) => (
  <label className="block text-sm font-medium text-secondary">
    {label}
    <input className="focus-ring mt-1 h-10 w-full rounded border border-border bg-white px-3 text-sm" maxLength={120} onChange={(event) => onChange(event.target.value)} required={required} type={type} value={value} />
  </label>
);

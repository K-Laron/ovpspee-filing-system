import { Edit2, KeyRound, Plus, RefreshCw, Search, X } from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';
import type { ReactNode } from 'react';

import { cmd } from '../../lib/invoke';
import { getUserErrorMessage } from '../../lib/errors';
import { nullable } from '../../lib/helpers';
import { passwordRulesText, validatePasswordPair } from '../../lib/passwords';
import { useSessionStore } from '../../store/sessionStore';
import type { Role, UserItem } from '../../types';

const emptyForm = {
  role: 'Secretary' as Role,
  firstName: '',
  middleName: '',
  lastName: '',
  username: '',
  email: '',
  contactNumber: '',
  address: '',
  password: '',
  confirmPassword: ''
};

export const Users = () => {
  const { sessionId } = useSessionStore();
  const [users, setUsers] = useState<UserItem[]>([]);
  const [search, setSearch] = useState('');
  const [form, setForm] = useState(emptyForm);
  const [editing, setEditing] = useState<UserItem | null>(null);
  const [isActive, setIsActive] = useState(true);
  const [resetTarget, setResetTarget] = useState<UserItem | null>(null);
  const [resetPassword, setResetPassword] = useState('');
  const [resetConfirmPassword, setResetConfirmPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState('');
  const [error, setError] = useState('');

  const reload = async () => {
    if (!sessionId) return;
    setLoading(true);
    setError('');
    try {
      setUsers(await cmd<UserItem[]>('list_users', { sessionId, search: search.trim() || null }));
    } catch (err) {
      setError(getUserErrorMessage(err, 'Could not load users.'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void reload();
  }, [sessionId]);

  const submitUser = async (event: FormEvent) => {
    event.preventDefault();
    if (!sessionId || saving) return;
    setSaving(true);
    setError('');
    setNotice('');
    try {
      const payload = {
        sessionId,
        role: form.role,
        firstName: form.firstName,
        middleName: nullable(form.middleName),
        lastName: form.lastName,
        username: form.username,
        email: nullable(form.email),
        contactNumber: nullable(form.contactNumber),
        address: nullable(form.address)
      };
      if (editing) {
        await cmd<UserItem>('update_user', { ...payload, userId: editing.user_id, isActive });
        setNotice('User updated.');
      } else {
        const validationError = validatePasswordPair(form.password, form.confirmPassword);
        if (validationError) {
          setError(validationError);
          setSaving(false);
          return;
        }
        await cmd<UserItem>('create_user', { ...payload, password: form.password });
        setNotice('User created.');
      }
      cancelEdit();
      await reload();
    } catch (err) {
      setError(getUserErrorMessage(err, 'User save failed.'));
    } finally {
      setSaving(false);
    }
  };

  const submitReset = async (event: FormEvent) => {
    event.preventDefault();
    if (!sessionId || !resetTarget || saving) return;
    setSaving(true);
    setError('');
    setNotice('');
    try {
      const validationError = validatePasswordPair(resetPassword, resetConfirmPassword);
      if (validationError) {
        setError(validationError);
        setSaving(false);
        return;
      }
      await cmd<void>('admin_reset_password', {
        sessionId,
        userId: resetTarget.user_id,
        newPassword: resetPassword
      });
      setNotice('Password reset.');
      setResetTarget(null);
      setResetPassword('');
      setResetConfirmPassword('');
    } catch (err) {
      setError(getUserErrorMessage(err, 'Password reset failed.'));
    } finally {
      setSaving(false);
    }
  };

  const editUser = (user: UserItem) => {
    setEditing(user);
    setIsActive(user.is_active);
    setForm({
      role: user.role,
      firstName: user.first_name,
      middleName: user.middle_name ?? '',
      lastName: user.last_name,
      username: user.username,
      email: user.email ?? '',
      contactNumber: user.contact_number ?? '',
      address: user.address ?? '',
      password: '',
      confirmPassword: ''
    });
  };

  const cancelEdit = () => {
    setEditing(null);
    setIsActive(true);
    setForm(emptyForm);
  };

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold text-secondary">Users</h1>
          <p className="text-sm text-muted">Admin and Secretary accounts</p>
        </div>
        <button className="focus-ring inline-flex h-10 items-center gap-2 rounded border border-border bg-surface px-3 text-sm font-medium text-secondary hover:bg-background" disabled={loading} onClick={() => void reload()} type="button">
          <RefreshCw size={16} />
          Refresh
        </button>
      </div>

      <form className="flex max-w-xl gap-2" onSubmit={(event) => { event.preventDefault(); void reload(); }}>
        <div className="relative flex-1">
          <Search className="absolute left-3 top-2.5 text-muted" size={16} />
          <input className="focus-ring h-10 w-full rounded border border-border bg-white pl-9 pr-3 text-sm" onChange={(event) => setSearch(event.target.value)} placeholder="Search username, name, email" value={search} />
        </div>
        <button className="focus-ring rounded bg-secondary px-4 text-sm font-semibold text-white" type="submit">Search</button>
      </form>

      {error && <div className="rounded border border-primary/30 bg-primary/10 px-4 py-3 text-sm text-primary">{error}</div>}
      {notice && <div className="rounded border border-success/30 bg-success/10 px-4 py-3 text-sm text-success">{notice}</div>}

      <section className="grid gap-5 lg:grid-cols-[1fr_22rem]">
        <div className="overflow-hidden rounded border border-border bg-surface">
          <table className="w-full table-fixed text-left text-sm">
            <thead className="bg-background text-xs uppercase text-muted">
              <tr>
                <th className="px-4 py-3">Name</th>
                <th className="px-4 py-3">Role</th>
                <th className="px-4 py-3">Username</th>
                <th className="px-4 py-3">Status</th>
                <th className="px-4 py-3">Action</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {loading && <tr><td className="px-4 py-6 text-center text-muted" colSpan={5}>Loading...</td></tr>}
              {!loading && users.length === 0 && (
                <tr>
                  <td className="px-4 py-6 text-center text-muted" colSpan={5}>
                    {search.trim() ? 'No users match the current search. Clear the search or try another name, username, or email.' : 'No users yet. Use the form on the right to create an Admin or Secretary account.'}
                  </td>
                </tr>
              )}
              {!loading && users.map((user) => (
                <tr key={user.user_id}>
                  <td className="px-4 py-3">
                    <div className="truncate font-medium text-secondary">{user.first_name} {user.last_name}</div>
                    <div className="truncate text-xs text-muted">{user.email || 'No email'}</div>
                  </td>
                  <td className="px-4 py-3">{user.role}</td>
                  <td className="px-4 py-3 font-mono text-xs">{user.username}</td>
                  <td className="px-4 py-3"><Status active={user.is_active} /></td>
                  <td className="px-4 py-3">
                    <div className="flex gap-2">
                      <IconButton label="Edit user" onClick={() => editUser(user)}><Edit2 size={15} /></IconButton>
                      <IconButton label="Reset password" onClick={() => { setResetTarget(user); setResetPassword(''); setResetConfirmPassword(''); }}><KeyRound size={15} /></IconButton>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="space-y-4">
          <form className="space-y-3 rounded border border-border bg-surface p-4" onSubmit={submitUser}>
            <FormTitle editing={Boolean(editing)} label="User" onCancel={cancelEdit} />
            <label className="block text-sm font-medium text-secondary">
              Role
              <select className="focus-ring mt-1 h-10 w-full rounded border border-border bg-white px-3 text-sm" onChange={(event) => setForm({ ...form, role: event.target.value as Role })} value={form.role}>
                <option value="Secretary">Secretary</option>
                <option value="Admin">Admin</option>
              </select>
            </label>
            <TextField label="First name" value={form.firstName} onChange={(value) => setForm({ ...form, firstName: value })} required />
            <TextField label="Middle name" value={form.middleName} onChange={(value) => setForm({ ...form, middleName: value })} />
            <TextField label="Last name" value={form.lastName} onChange={(value) => setForm({ ...form, lastName: value })} required />
            <TextField label="Username" value={form.username} onChange={(value) => setForm({ ...form, username: value })} required />
            <TextField label="Email" type="email" value={form.email} onChange={(value) => setForm({ ...form, email: value })} />
            <TextField label="Contact number" value={form.contactNumber} onChange={(value) => setForm({ ...form, contactNumber: value })} />
            <TextField label="Address" value={form.address} onChange={(value) => setForm({ ...form, address: value })} />
            {!editing && (
              <div className="space-y-2">
                <TextField label="Password" type="password" value={form.password} onChange={(value) => setForm({ ...form, password: value })} required />
                <TextField label="Confirm password" type="password" value={form.confirmPassword} onChange={(value) => setForm({ ...form, confirmPassword: value })} required />
                <p className="text-xs text-muted">{passwordRulesText}</p>
              </div>
            )}
            {editing && <label className="flex items-center gap-2 text-sm font-medium text-secondary"><input checked={isActive} className="h-4 w-4" onChange={(event) => setIsActive(event.target.checked)} type="checkbox" />Active</label>}
            <button className="focus-ring inline-flex h-10 w-full items-center justify-center gap-2 rounded bg-primary px-3 text-sm font-semibold text-white hover:bg-secondary disabled:opacity-60" disabled={saving} type="submit">
              <Plus size={16} />
              {editing ? 'Save' : 'Create'}
            </button>
          </form>

          {resetTarget && (
            <form className="space-y-3 rounded border border-border bg-surface p-4" onSubmit={submitReset}>
              <FormTitle editing label={`Reset ${resetTarget.username}`} onCancel={() => { setResetTarget(null); setResetPassword(''); setResetConfirmPassword(''); }} />
              <TextField label="New password" type="password" value={resetPassword} onChange={setResetPassword} required />
              <TextField label="Confirm new password" type="password" value={resetConfirmPassword} onChange={setResetConfirmPassword} required />
              <p className="text-xs text-muted">{passwordRulesText}</p>
              <button className="focus-ring inline-flex h-10 w-full items-center justify-center gap-2 rounded bg-secondary px-3 text-sm font-semibold text-white disabled:opacity-60" disabled={saving} type="submit">
                <KeyRound size={16} />
                Reset Password
              </button>
            </form>
          )}
        </div>
      </section>
    </div>
  );
};

const Status = ({ active }: { active: boolean }) => (
  <span className={['inline-flex h-7 items-center rounded px-2 text-xs font-medium', active ? 'bg-success/10 text-success' : 'bg-warning/10 text-warning'].join(' ')}>
    {active ? 'Active' : 'Inactive'}
  </span>
);

const IconButton = ({ children, label, onClick }: { children: ReactNode; label: string; onClick: () => void }) => (
  <button aria-label={label} className="focus-ring inline-flex h-9 w-9 items-center justify-center rounded border border-border text-secondary hover:bg-background" onClick={onClick} title={label} type="button">
    {children}
  </button>
);

const FormTitle = ({ editing, label, onCancel }: { editing: boolean; label: string; onCancel: () => void }) => (
  <div className="flex items-center justify-between gap-2">
    <h2 className="text-base font-semibold text-secondary">{editing ? `Edit ${label}` : `New ${label}`}</h2>
    {editing && <button aria-label="Cancel edit" className="focus-ring inline-flex h-8 w-8 items-center justify-center rounded border border-border text-muted hover:text-secondary" onClick={onCancel} title="Cancel edit" type="button"><X size={15} /></button>}
  </div>
);

const TextField = ({ label, onChange, required = false, type = 'text', value }: { label: string; onChange: (value: string) => void; required?: boolean; type?: string; value: string }) => (
  <label className="block text-sm font-medium text-secondary">
    {label}
    <input className="focus-ring mt-1 h-10 w-full rounded border border-border bg-white px-3 text-sm" maxLength={120} onChange={(event) => onChange(event.target.value)} required={required} type={type} value={value} />
  </label>
);

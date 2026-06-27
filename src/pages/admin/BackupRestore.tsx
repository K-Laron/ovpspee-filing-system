import { open, save } from '@tauri-apps/plugin-dialog';
import { Archive, DatabaseBackup, FolderOpen, RefreshCw, RotateCcw, Save, ShieldAlert } from 'lucide-react';
import { useEffect, useState } from 'react';

import { ConfirmDialog, type ConfirmAction } from '../../components/ConfirmDialog';
import { EmptyState } from '../../components/EmptyState';
import { formatDateTime } from '../../lib/dates';
import { invoke } from '@tauri-apps/api/core';
import { getUserErrorMessage } from '../../lib/errors';
import { formatBytes } from '../../lib/helpers';
import { useSessionStore } from '../../store/sessionStore';
import type { BackupSettings, BackupSummary, BackupValidation, RestoreResult } from '../../types';

export const BackupRestore = () => {
  const sessionId = useSessionStore((state) => state.sessionId);
  const [settings, setSettings] = useState<BackupSettings | null>(null);
  const [history, setHistory] = useState<BackupSummary[]>([]);
  const [destination, setDestination] = useState('');
  const [scheduleEnabled, setScheduleEnabled] = useState(false);
  const [scheduleTime, setScheduleTime] = useState('02:00');
  const [retentionCount, setRetentionCount] = useState(10);
  const [selected, setSelected] = useState('');
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null);
  const clearConfirmAction = () => setConfirmAction(null);

  const load = async () => {
    if (!sessionId) return;
    const [nextSettings, nextHistory] = await Promise.all([
      invoke<BackupSettings>('get_backup_settings', { sessionId }),
      invoke<BackupSummary[]>('list_backup_history', { sessionId })
    ]);
    setSettings(nextSettings);
    setDestination(nextSettings.destination_path);
    setScheduleEnabled(nextSettings.schedule_enabled);
    setScheduleTime(nextSettings.schedule_time);
    setRetentionCount(nextSettings.retention_count);
    setHistory(nextHistory);
    setSelected((current) => current || nextHistory[0]?.backup_name || '');
  };

  useEffect(() => {
    void load().catch((err) => setMessage(getUserErrorMessage(err, 'Could not load backup settings.')));
  }, [sessionId]);

  const chooseDestination = async () => {
    const path = await open({ directory: true, multiple: false });
    if (typeof path === 'string') setDestination(path);
  };

  const saveSettings = async () => {
    if (!sessionId) return;
    setBusy(true);
    try {
      const updated = await invoke<BackupSettings>('update_backup_settings', {
        sessionId,
        destinationPath: destination || null,
        scheduleEnabled,
        scheduleTime,
        retentionCount
      });
      setSettings(updated);
      setMessage('Backup settings saved.');
      await load();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not save backup settings.'));
    } finally {
      setBusy(false);
    }
  };

  const createNow = async () => {
    if (!sessionId || busy) return;
    setBusy(true);
    try {
      const backup = await invoke<BackupSummary>('create_backup', { sessionId });
      setMessage(`Backup created: ${backup.backup_name}`);
      await load();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not create backup.'));
    } finally {
      setBusy(false);
    }
  };

  const exportArchive = async () => {
    if (!sessionId || !selected) return;
    const outputPath = await save({
      defaultPath: `${selected}.ovpspee-backup`,
      filters: [{ name: 'OVPSPEE Backup', extensions: ['ovpspee-backup'] }]
    });
    if (!outputPath) return;
    setBusy(true);
    try {
      const path = await invoke<string>('export_backup_archive', { sessionId, backupName: selected, outputPath });
      setMessage(`Portable backup exported: ${path}`);
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not export backup.'));
    } finally {
      setBusy(false);
    }
  };

  const importArchive = async () => {
    if (!sessionId) return;
    const path = await open({
      multiple: false,
      filters: [{ name: 'OVPSPEE Backup', extensions: ['ovpspee-backup'] }]
    });
    if (typeof path !== 'string') return;
    setBusy(true);
    try {
      const validation = await invoke<BackupValidation>('validate_backup_archive', { sessionId, archivePath: path });
      const imported = await invoke<BackupSummary>('import_backup_archive', { sessionId, archivePath: path });
      setSelected(imported.backup_name);
      setMessage(`Imported valid backup: ${validation.backup_name}`);
      await load();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not restore backup.'));
    } finally {
      setBusy(false);
    }
  };

  const restore = async (backupName: string) => {
    if (!sessionId || busy) return;
    setBusy(true);
    try {
      const result = await invoke<RestoreResult>('restore_from_backup', { sessionId, backupName });
      setMessage(`${result.message} Safety backup: ${result.pre_restore_backup_name}`);
      await load();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not restore backup.'));
    } finally {
      setBusy(false);
    }
  };

  const confirmRestore = (backupName: string) => {
    setConfirmAction({
      title: 'Restore backup?',
      body: <>Restore <strong>{backupName}</strong>. Current data will be replaced, and a safety backup will be created first.</>,
      confirmLabel: 'Restore Backup',
      requiredText: backupName,
      onConfirm: () => restore(backupName)
    });
  };

  const handleConfirmAction = async () => {
    if (!confirmAction) return;
    try {
      await confirmAction.onConfirm();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not complete confirmation action.'));
    } finally {
      clearConfirmAction();
    }
  };

  return (
    <section className="space-y-5">
      {confirmAction && (
        <ConfirmDialog
          body={confirmAction.body}
          confirmLabel={confirmAction.confirmLabel}
          onCancel={() => clearConfirmAction()}
          onConfirm={() => handleConfirmAction()}
          requiredText={confirmAction.requiredText}
          title={confirmAction.title}
        />
      )}

      <div className="flex items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-bold text-secondary">Backup & Restore</h1>
          <p className="mt-1 text-sm text-muted">Admin-only data protection and portability.</p>
        </div>
        <button className="btn" onClick={() => void load().catch((err) => setMessage(getUserErrorMessage(err, 'Could not load backup settings.')))} type="button">
          <RefreshCw size={16} />
          Refresh
        </button>
      </div>

      {settings?.is_local_app_data && (
        <div className="flex gap-2 rounded border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">
          <ShieldAlert className="shrink-0" size={18} />
          Local-only backups do not protect against device loss or drive failure. Copy backups to external, network, or removable storage.
        </div>
      )}

      {message && <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div>}

      <div className="grid gap-4 lg:grid-cols-[1fr_1fr]">
        <div className="space-y-4 rounded border border-border bg-surface p-4 shadow-sm">
          <h2 className="text-lg font-semibold text-secondary">Backup Settings</h2>
          <label className="block text-sm">
            <span className="form-label">Destination</span>
            <div className="flex gap-2">
              <input className="input" value={destination} onChange={(event) => setDestination(event.target.value)} />
              <button className="btn" onClick={() => void chooseDestination()} type="button">
                <FolderOpen size={16} />
                Choose
              </button>
            </div>
          </label>
          <div className="grid gap-3 sm:grid-cols-2">
            <label className="flex items-center gap-2 text-sm text-secondary">
              <input checked={scheduleEnabled} onChange={(event) => setScheduleEnabled(event.target.checked)} type="checkbox" />
              Scheduled backups
            </label>
            <label className="block text-sm">
              <span className="form-label">Daily time</span>
              <input className="input" onChange={(event) => setScheduleTime(event.target.value)} type="time" value={scheduleTime} />
            </label>
            <label className="block text-sm">
              <span className="form-label">Keep last</span>
              <input className="input" min={1} max={100} onChange={(event) => setRetentionCount(Number(event.target.value))} type="number" value={retentionCount} />
            </label>
          </div>
          <div className="flex flex-wrap gap-2">
            <button className="btn btn-primary" disabled={busy} onClick={() => void saveSettings()} type="button">
              <Save size={16} />
              Save Settings
            </button>
            <button className="btn" disabled={busy} onClick={() => void createNow()} type="button">
              <DatabaseBackup size={16} />
              Create Backup Now
            </button>
          </div>
        </div>

        <div className="space-y-4 rounded border border-border bg-surface p-4 shadow-sm">
          <h2 className="text-lg font-semibold text-secondary">Portable Backup</h2>
          <label className="block text-sm">
            <span className="form-label">Selected backup</span>
            <select className="input" value={selected} onChange={(event) => setSelected(event.target.value)}>
              <option value="">Select backup</option>
              {history.map((backup) => <option key={backup.backup_name} value={backup.backup_name}>{backup.backup_name}</option>)}
            </select>
          </label>
          <div className="flex flex-wrap gap-2">
            <button className="btn" disabled={busy || !selected} onClick={() => void exportArchive()} type="button">
              <Archive size={16} />
              Export .ovpspee-backup
            </button>
            <button className="btn" disabled={busy} onClick={() => void importArchive()} type="button">
              <FolderOpen size={16} />
              Import Archive
            </button>
            <button className="btn btn-primary" disabled={busy || !selected} onClick={() => confirmRestore(selected)} type="button">
              <RotateCcw size={16} />
              Restore Selected
            </button>
          </div>
        </div>
      </div>

      <div className="overflow-hidden rounded border border-border bg-surface shadow-sm">
        <table className="w-full text-left text-sm">
          <thead className="border-b border-border bg-background text-xs uppercase text-muted">
            <tr><th className="p-3">Backup</th><th className="p-3">Created</th><th className="p-3">Size</th><th className="p-3">Status</th><th className="p-3">Action</th></tr>
          </thead>
          <tbody>
            {history.map((backup) => (
              <tr className="border-b border-border" key={backup.backup_name}>
                <td className="p-3">
                  <p className="font-semibold text-secondary">{backup.backup_name}</p>
                  <p className="text-xs text-muted">{backup.backup_path}</p>
                </td>
                <td className="p-3 text-muted">{formatDateTime(backup.created_at)}</td>
                <td className="p-3 text-muted">{formatBytes(backup.total_bytes)}</td>
                <td className="p-3">{backup.is_valid ? 'Valid' : 'Invalid'} · {backup.file_count} file(s)</td>
                <td className="p-3">
                  <button className="btn" disabled={busy} onClick={() => confirmRestore(backup.backup_name)} type="button">
                    <RotateCcw size={16} />
                    Restore
                  </button>
                </td>
              </tr>
            ))}
            {history.length === 0 && (
              <tr>
                <td className="p-4" colSpan={5}>
                  <EmptyState
                    actionLabel="Create Backup Now"
                    message="Create a backup before making major changes or testing restore workflows."
                    onAction={() => void createNow()}
                    title="No backups found"
                  />
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
};




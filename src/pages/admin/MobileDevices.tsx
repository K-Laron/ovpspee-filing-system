import { Ban, KeyRound, RefreshCw, Smartphone } from 'lucide-react';
import { useEffect, useState } from 'react';

import { ConfirmDialog, type ConfirmAction } from '../../components/ConfirmDialog';
import { invoke } from '@tauri-apps/api/core';
import { getUserErrorMessage } from '../../lib/errors';
import { useSessionStore } from '../../store/sessionStore';
import type { CreatedMobileDevice, MobileDeviceItem } from '../../types';

export const MobileDevices = () => {
  const sessionId = useSessionStore((state) => state.sessionId);
  const [devices, setDevices] = useState<MobileDeviceItem[]>([]);
  const [deviceName, setDeviceName] = useState('');
  const [created, setCreated] = useState<CreatedMobileDevice | null>(null);
  const [message, setMessage] = useState('');
  const [loading, setLoading] = useState(false);
  const [creating, setCreating] = useState(false);
  const [revokingId, setRevokingId] = useState<string | null>(null);
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null);
  const clearConfirmAction = () => setConfirmAction(null);

  const load = async () => {
    if (!sessionId) return;
    setLoading(true);
    setMessage('');
    try {
      setDevices(await invoke<MobileDeviceItem[]>('list_mobile_devices', { sessionId }));
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not load mobile devices.'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, [sessionId]);

  const createToken = async () => {
    if (!sessionId) return;
    setCreating(true);
    setMessage('');
    setCreated(null);
    try {
      const next = await invoke<CreatedMobileDevice>('create_mobile_device', {
        sessionId,
        deviceName: deviceName.trim()
      });
      setCreated(next);
      setDeviceName('');
      setMessage('Mobile device token created.');
      await load();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not create mobile device token.'));
    } finally {
      setCreating(false);
    }
  };

  const revoke = async (deviceId: string) => {
    if (!sessionId) return;
    setRevokingId(deviceId);
    setMessage('');
    try {
      await invoke<void>('revoke_mobile_device', { sessionId, deviceId });
      setMessage('Mobile device revoked.');
      await load();
    } catch (err) {
      setMessage(getUserErrorMessage(err, 'Could not revoke mobile device.'));
    } finally {
      setRevokingId(null);
    }
  };

  const confirmRevoke = (device: MobileDeviceItem) => {
    setConfirmAction({
      title: 'Revoke mobile device?',
      body: <>Revoke <strong>{device.device_name}</strong>. This Android phone will no longer be allowed to upload documents to this office PC.</>,
      confirmLabel: 'Revoke Device',
      onConfirm: () => revoke(device.device_id)
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
          title={confirmAction.title}
        />
      )}

      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-bold text-secondary">Mobile Devices</h1>
          <p className="mt-1 text-sm text-muted">Create Android device tokens and revoke lost or retired phones.</p>
        </div>
        <button className="btn" disabled={loading} onClick={() => void load()} type="button">
          <RefreshCw size={16} />{loading ? 'Refreshing...' : 'Refresh'}
        </button>
      </div>

      {message ? <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div> : null}

      <div className="grid gap-5 xl:grid-cols-[360px_1fr]">
        <div className="rounded border border-border bg-surface p-5 shadow-sm">
          <div className="mb-4 flex items-center gap-2">
            <KeyRound size={18} className="text-primary" />
            <h2 className="text-lg font-semibold text-secondary">Create token</h2>
          </div>
          <label>
            <span className="form-label">Device name</span>
            <input
              className="input"
              onChange={(event) => setDeviceName(event.target.value)}
              placeholder="Records Android"
              value={deviceName}
            />
          </label>
          <button
            className="btn btn-primary mt-3 w-full"
            disabled={creating || deviceName.trim().length < 2}
            onClick={() => void createToken()}
            type="button"
          >
            <KeyRound size={16} />{creating ? 'Creating...' : 'Create token'}
          </button>

          {created ? (
            <div className="mt-4 rounded border border-amber-200 bg-amber-50 p-3 text-sm text-amber-950">
              <p className="font-semibold">Show this once on the Android phone.</p>
              <dl className="mt-2 space-y-2 break-all">
                <div>
                  <dt className="text-xs font-semibold uppercase text-amber-700">Device ID</dt>
                  <dd>{created.device_id}</dd>
                </div>
                <div>
                  <dt className="text-xs font-semibold uppercase text-amber-700">Device token</dt>
                  <dd>{created.device_token}</dd>
                </div>
              </dl>
            </div>
          ) : null}
        </div>

        <div className="rounded border border-border bg-surface p-5 shadow-sm">
          <div className="mb-4 flex items-center gap-2">
            <Smartphone size={18} className="text-primary" />
            <h2 className="text-lg font-semibold text-secondary">Approved Android phones</h2>
          </div>
          {devices.length === 0 ? (
            <p className="rounded border border-border bg-background p-4 text-sm text-muted">No approved phones yet.</p>
          ) : (
            <div className="overflow-x-auto">
              <table className="min-w-full text-left text-sm">
                <thead className="border-b border-border text-xs uppercase text-muted">
                  <tr>
                    <th className="py-2 pr-4">Device</th>
                    <th className="py-2 pr-4">Status</th>
                    <th className="py-2 pr-4">Last seen</th>
                    <th className="py-2 text-right">Action</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {devices.map((device) => (
                    <tr key={device.device_id}>
                      <td className="py-3 pr-4">
                        <div className="font-semibold text-secondary">{device.device_name}</div>
                        <div className="mt-1 break-all text-xs text-muted">{device.device_id}</div>
                      </td>
                      <td className="py-3 pr-4">
                        <span className={`rounded px-2 py-1 text-xs font-semibold ${device.is_active ? 'bg-green-50 text-green-700' : 'bg-slate-100 text-muted'}`}>
                          {device.is_active ? 'Active' : 'Revoked'}
                        </span>
                      </td>
                      <td className="py-3 pr-4 text-muted">{device.last_seen_at ?? 'Never'}</td>
                      <td className="py-3 text-right">
                        <button
                          className="btn"
                          disabled={!device.is_active || revokingId === device.device_id}
                          onClick={() => confirmRevoke(device)}
                          type="button"
                        >
                          <Ban size={16} />{revokingId === device.device_id ? 'Revoking...' : 'Revoke'}
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </section>
  );
};

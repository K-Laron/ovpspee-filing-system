import { RefreshCw, Save, ScanLine, Printer } from 'lucide-react';
import { useEffect, useState } from 'react';

import {
  getDefaultPrinter,
  getDeviceSettings,
  listPrinters,
  listScanners,
  updateDeviceSettings
} from '../../lib/invoke';
import { useSessionStore } from '../../store/sessionStore';
import type { DeviceSettings, PrinterDevice, ScannerDevice } from '../../types';

const getErrorMessage = (error: unknown, fallback: string) =>
  error instanceof Error ? error.message : typeof error === 'string' ? error : fallback;

const defaultSettings: DeviceSettings = {
  default_scanner_id: null,
  default_printer_id: null,
  scan_default_dpi: 300,
  scan_default_color_mode: 'color',
  scan_default_output_format: 'pdf',
  device_detection_last_checked_at: null
};

export const DeviceSettingsPage = () => {
  const sessionId = useSessionStore((state) => state.sessionId);
  const role = useSessionStore((state) => state.role);
  const readOnly = role !== 'Admin';
  const [scanners, setScanners] = useState<ScannerDevice[]>([]);
  const [printers, setPrinters] = useState<PrinterDevice[]>([]);
  const [settings, setSettings] = useState<DeviceSettings>(defaultSettings);
  const [message, setMessage] = useState('');
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);

  const load = async () => {
    if (!sessionId) return;
    setLoading(true);
    setMessage('');
    try {
      const [nextSettings, nextScanners, nextPrinters] = await Promise.all([
        getDeviceSettings(sessionId),
        listScanners(sessionId),
        listPrinters(sessionId),
        getDefaultPrinter(sessionId)
      ]);
      setSettings(nextSettings);
      setScanners(nextScanners);
      setPrinters(nextPrinters);
      setMessage('Device detection refreshed.');
    } catch (err) {
      setMessage(getErrorMessage(err, 'Could not detect devices.'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, [sessionId]);

  const saveSettings = async () => {
    if (!sessionId || readOnly) return;
    setSaving(true);
    setMessage('');
    try {
      const updated = await updateDeviceSettings({
        sessionId,
        defaultScannerId: settings.default_scanner_id,
        defaultPrinterId: settings.default_printer_id,
        scanDefaultDpi: settings.scan_default_dpi,
        scanDefaultColorMode: settings.scan_default_color_mode,
        scanDefaultOutputFormat: settings.scan_default_output_format
      });
      setSettings(updated);
      setMessage('Device defaults saved.');
    } catch (err) {
      setMessage(getErrorMessage(err, 'Could not save device defaults.'));
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="space-y-5">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-bold text-secondary">Devices</h1>
          <p className="mt-1 text-sm text-muted">
            Detect scanners and printers, then choose defaults for later scan and print workflows.
          </p>
        </div>
        <div className="flex gap-2">
          <button className="btn" disabled={loading} onClick={() => void load()} type="button">
            <RefreshCw size={16} />{loading ? 'Refreshing...' : 'Refresh Devices'}
          </button>
          {!readOnly && (
            <button className="btn btn-primary" disabled={saving} onClick={() => void saveSettings()} type="button">
              <Save size={16} />{saving ? 'Saving...' : 'Save Defaults'}
            </button>
          )}
        </div>
      </div>

      {readOnly && (
        <div className="rounded border border-border bg-surface p-3 text-sm text-muted">
          Device defaults are read-only for Secretary accounts.
        </div>
      )}
      {message && <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div>}

      <div className="grid gap-5 xl:grid-cols-2">
        <div className="rounded border border-border bg-surface p-5 shadow-sm">
          <div className="mb-4 flex items-center gap-2">
            <ScanLine size={18} className="text-primary" />
            <h2 className="text-lg font-semibold text-secondary">Scanner Detection</h2>
          </div>
          {scanners.length === 0 ? (
            <p className="rounded border border-dashed border-border p-4 text-sm text-muted">No scanner detected.</p>
          ) : (
            <div className="space-y-3">
              {scanners.map((scanner) => (
                <label className="flex cursor-pointer items-start gap-3 rounded border border-border p-3 text-sm" key={scanner.device_id}>
                  <input
                    checked={settings.default_scanner_id === scanner.device_id}
                    className="mt-1"
                    disabled={readOnly}
                    name="default-scanner"
                    onChange={() => setSettings((current) => ({ ...current, default_scanner_id: scanner.device_id }))}
                    type="radio"
                  />
                  <span className="min-w-0 flex-1">
                    <span className="block font-semibold text-secondary">{scanner.name}</span>
                    <span className="block text-xs text-muted">
                      {scanner.manufacturer ?? 'Unknown maker'} · {scanner.connection_type ?? 'Unknown connection'}
                    </span>
                  </span>
                  <StatusBadge ok={scanner.is_available} text={scanner.status ?? (scanner.is_available ? 'Available' : 'Unavailable')} />
                </label>
              ))}
            </div>
          )}

          <div className="mt-5 grid gap-3 md:grid-cols-3">
            <label>
              <span className="form-label">DPI default</span>
              <select
                className="input"
                disabled={readOnly}
                value={settings.scan_default_dpi}
                onChange={(event) => setSettings((current) => ({ ...current, scan_default_dpi: Number(event.target.value) }))}
              >
                <option value={200}>200</option>
                <option value={300}>300</option>
                <option value={600}>600</option>
              </select>
            </label>
            <label>
              <span className="form-label">Color mode</span>
              <select
                className="input"
                disabled={readOnly}
                value={settings.scan_default_color_mode}
                onChange={(event) => setSettings((current) => ({ ...current, scan_default_color_mode: event.target.value as DeviceSettings['scan_default_color_mode'] }))}
              >
                <option value="color">Color</option>
                <option value="grayscale">Grayscale</option>
                <option value="black_white">Black & white</option>
              </select>
            </label>
            <label>
              <span className="form-label">Output format</span>
              <select
                className="input"
                disabled={readOnly}
                value={settings.scan_default_output_format}
                onChange={(event) => setSettings((current) => ({ ...current, scan_default_output_format: event.target.value as DeviceSettings['scan_default_output_format'] }))}
              >
                <option value="pdf">PDF</option>
                <option value="png">PNG</option>
                <option value="jpg">JPG</option>
              </select>
            </label>
          </div>
        </div>

        <div className="rounded border border-border bg-surface p-5 shadow-sm">
          <div className="mb-4 flex items-center gap-2">
            <Printer size={18} className="text-primary" />
            <h2 className="text-lg font-semibold text-secondary">Printer Detection</h2>
          </div>
          {printers.length === 0 ? (
            <p className="rounded border border-dashed border-border p-4 text-sm text-muted">No printer detected.</p>
          ) : (
            <div className="space-y-3">
              {printers.map((printer) => (
                <label className="flex cursor-pointer items-start gap-3 rounded border border-border p-3 text-sm" key={printer.printer_id}>
                  <input
                    checked={settings.default_printer_id === printer.printer_id}
                    className="mt-1"
                    disabled={readOnly}
                    name="default-printer"
                    onChange={() => setSettings((current) => ({ ...current, default_printer_id: printer.printer_id }))}
                    type="radio"
                  />
                  <span className="min-w-0 flex-1">
                    <span className="block font-semibold text-secondary">{printer.name}</span>
                    <span className="block text-xs text-muted">
                      {printer.is_network ? 'Network printer' : 'Local printer'}{printer.is_default ? ' · Windows default' : ''}
                    </span>
                  </span>
                  <StatusBadge ok={printer.is_available} text={printer.status} />
                </label>
              ))}
            </div>
          )}
        </div>
      </div>
    </section>
  );
};

const StatusBadge = ({ ok, text }: { ok: boolean; text: string }) => (
  <span className={`shrink-0 rounded px-2 py-1 text-xs font-semibold ${ok ? 'bg-green-50 text-green-700' : 'bg-slate-100 text-muted'}`}>
    {text}
  </span>
);

import React, { useEffect, useMemo, useState } from 'react';
import { Pressable, StatusBar, StyleSheet, Text, View } from 'react-native';

import { ApiClient } from './api/client';
import { capturePhoto, pickFile } from './native/capture';
import { AttachmentReviewScreen } from './screens/AttachmentReviewScreen';
import { CaptureHomeScreen } from './screens/CaptureHomeScreen';
import { LoginScreen } from './screens/LoginScreen';
import { MetadataWizardScreen } from './screens/MetadataWizardScreen';
import { SettingsScreen } from './screens/SettingsScreen';
import { SubmissionHistoryScreen } from './screens/SubmissionHistoryScreen';
import {
  clearDraft,
  loadDeviceProfile,
  loadDraft,
  loadHubUrl,
  loadQueuedSubmissions,
  markQueuedSubmissionAttempt,
  markQueuedSubmissionRetrying,
  newClientSubmissionId,
  removeQueuedSubmission,
  saveDeviceProfile,
  saveDraft,
  saveHubUrl
} from './storage/drafts';
import type {
  DeviceProfile,
  MobileApi,
  MobileSubmissionDraft,
  QueuedSubmission,
  SessionPayload,
  SubmissionHistoryItem
} from './types';

type Screen = 'login' | 'capture' | 'metadata' | 'review' | 'history' | 'settings';

const APP_VERSION = '0.1.1';
const SESSION_IDLE_MS = 15 * 60 * 1000;
const today = () => new Date().toISOString().slice(0, 10);

const emptyDraft = (): MobileSubmissionDraft => ({
  clientSubmissionId: newClientSubmissionId(),
  documentName: '',
  categoryId: null,
  folderId: null,
  officeId: null,
  dateReceived: today(),
  remarks: '',
  status: 'Filed',
  attachments: []
});

const defaultDeviceProfile = (): DeviceProfile => ({
  deviceId: 'device-pending',
  deviceName: 'Android office phone',
  deviceToken: ''
});

interface AppRootProps {
  api?: MobileApi;
  initialHubUrl?: string;
}

export function AppRoot({ api: injectedApi, initialHubUrl = 'http://192.168.1.10:1421' }: AppRootProps) {
  const [hubUrl, setHubUrl] = useState(initialHubUrl);
  const [deviceProfile, setDeviceProfileState] = useState<DeviceProfile>(defaultDeviceProfile());
  const api = useMemo(() => injectedApi ?? new ApiClient(hubUrl, deviceProfile), [hubUrl, injectedApi, deviceProfile]);
  const [screen, setScreen] = useState<Screen>('login');
  const [session, setSession] = useState<SessionPayload | null>(null);
  const [draft, setDraft] = useState<MobileSubmissionDraft>(emptyDraft());
  const [queue, setQueue] = useState<QueuedSubmission[]>([]);
  const [history, setHistory] = useState<SubmissionHistoryItem[]>([]);
  const [lastSubmissionId, setLastSubmissionId] = useState<number | null>(null);
  const [captureError, setCaptureError] = useState('');
  const [locked, setLocked] = useState(false);
  const [lastActiveAt, setLastActiveAt] = useState(() => Date.now());

  useEffect(() => {
    let mounted = true;
    Promise.all([loadHubUrl(), loadDraft(), loadDeviceProfile(), loadQueuedSubmissions()]).then(
      ([storedHubUrl, storedDraft, storedDeviceProfile, storedQueue]) => {
        if (!mounted) return;
        if (storedHubUrl) setHubUrl(storedHubUrl);
        if (storedDraft) setDraft(storedDraft);
        setDeviceProfileState(storedDeviceProfile);
        setQueue(storedQueue);
      }
    );
    return () => {
      mounted = false;
    };
  }, []);

  useEffect(() => {
    if (!session) return;
    const timer = setInterval(() => {
      if (Date.now() - lastActiveAt >= SESSION_IDLE_MS) {
        setLocked(true);
      }
    }, 30_000);
    return () => clearInterval(timer);
  }, [lastActiveAt, session]);

  const touch = () => setLastActiveAt(Date.now());

  const updateDraft = (next: MobileSubmissionDraft | ((current: MobileSubmissionDraft) => MobileSubmissionDraft)) => {
    setDraft((current) => {
      const value = typeof next === 'function' ? next(current) : next;
      void saveDraft(value);
      return value;
    });
    touch();
  };

  const updateHubUrl = (value: string) => {
    setHubUrl(value);
    void saveHubUrl(value);
  };

  const updateDeviceProfile = (profile: DeviceProfile) => {
    setDeviceProfileState(profile);
    void saveDeviceProfile(profile);
  };

  const reloadQueue = async () => {
    setQueue(await loadQueuedSubmissions());
  };

  const syncQueue = async () => {
    if (!session) return;
    touch();
    for (const item of queue) {
      try {
        await markQueuedSubmissionRetrying(item.clientSubmissionId);
        const result = await api.createSubmission(session.session_id, item.draft);
        await removeQueuedSubmission(item.clientSubmissionId);
        setLastSubmissionId(result.mobile_submission_id);
      } catch {
        await markQueuedSubmissionAttempt(item.clientSubmissionId, 'Retry failed. Check office Wi-Fi and desktop hub.');
      }
    }
    await reloadQueue();
  };

  const stageAttachment = async (loader: () => Promise<MobileSubmissionDraft['attachments'][number]>, message: string) => {
    touch();
    setCaptureError('');
    try {
      const file = await loader();
      updateDraft((current) => ({ ...current, attachments: [...current.attachments, file] }));
    } catch {
      setCaptureError(message);
    }
  };

  const nav = session ? (
    <View style={styles.nav}>
      {(['capture', 'history', 'settings'] as Screen[]).map((item) => (
        <Pressable
          accessibilityRole="button"
          key={item}
          onPress={() => setScreen(item)}
          style={[styles.navButton, screen === item && styles.navButtonActive]}
        >
          <Text style={[styles.navText, screen === item && styles.navTextActive]}>
            {item === 'capture' ? 'Capture' : item === 'history' ? 'History' : 'Settings'}
          </Text>
        </Pressable>
      ))}
    </View>
  ) : null;

  return (
    <View style={styles.safe}>
      <StatusBar barStyle="light-content" backgroundColor="#12312b" />
      <View style={styles.shell}>
        <View style={styles.header}>
          <Text style={styles.kicker}>OVPSPEE Secretary</Text>
          <Text style={styles.title}>Mobile Intake</Text>
          {session ? <Text style={styles.subtitle}>{session.display_name}</Text> : null}
        </View>
        {screen === 'login' ? (
          <LoginScreen
            api={api}
            deviceProfile={deviceProfile}
            hubUrl={hubUrl}
            onDeviceProfileChange={updateDeviceProfile}
            onHubUrlChange={updateHubUrl}
            onLoggedIn={(nextSession) => {
              setSession(nextSession);
              setLocked(false);
              touch();
              setScreen('capture');
            }}
          />
        ) : null}
        {locked && session ? (
          <View style={styles.lockOverlay}>
            <Text style={styles.lockTitle}>Session locked</Text>
            <Text style={styles.lockText}>Re-open with Secretary login after 15 minutes idle.</Text>
            <Pressable
              accessibilityRole="button"
              onPress={() => {
                setSession(null);
                setLocked(false);
                setScreen('login');
              }}
              style={styles.lockButton}
            >
              <Text style={styles.lockButtonText}>Return to login</Text>
            </Pressable>
          </View>
        ) : null}
        {screen === 'capture' && session ? (
          <CaptureHomeScreen
            captureError={captureError}
            draft={draft}
            lastSubmissionId={lastSubmissionId}
            pendingQueueCount={queue.length}
            onAddFile={() => void stageAttachment(pickFile, 'Could not open the Android file picker.')}
            onCapture={() => void stageAttachment(capturePhoto, 'Could not capture a camera photo.')}
            onNext={() => {
              touch();
              setScreen('metadata');
            }}
            onSyncQueue={() => void syncQueue()}
          />
        ) : null}
        {screen === 'metadata' && session ? (
          <MetadataWizardScreen
            api={api}
            sessionId={session.session_id}
            draft={draft}
            onBack={() => setScreen('capture')}
            onChange={updateDraft}
            onNext={() => {
              touch();
              setScreen('review');
            }}
          />
        ) : null}
        {screen === 'review' && session ? (
          <AttachmentReviewScreen
            api={api}
            sessionId={session.session_id}
            draft={draft}
            onBack={() => setScreen('metadata')}
            onRemoveAttachment={(index) =>
              updateDraft((current) => ({
                ...current,
                attachments: current.attachments.filter((_, itemIndex) => itemIndex !== index)
              }))
            }
            onQueued={(submission) => {
              setQueue((current) => [
                submission,
                ...current.filter((item) => item.clientSubmissionId !== submission.clientSubmissionId)
              ]);
            }}
            onSubmitted={(id) => {
              setLastSubmissionId(id);
              const nextDraft = emptyDraft();
              setDraft(nextDraft);
              void clearDraft();
              setScreen('capture');
            }}
          />
        ) : null}
        {screen === 'history' && session ? (
          <SubmissionHistoryScreen
            api={api}
            sessionId={session.session_id}
            history={history}
            queue={queue}
            onHistoryLoaded={setHistory}
            onSyncQueue={() => void syncQueue()}
          />
        ) : null}
        {screen === 'settings' && session ? (
          <SettingsScreen
            api={api}
            appVersion={APP_VERSION}
            deviceProfile={deviceProfile}
            hubUrl={hubUrl}
            pendingQueueCount={queue.length}
            onDeviceProfileChange={updateDeviceProfile}
            onHubUrlChange={updateHubUrl}
            onLock={() => setLocked(true)}
            onLogout={() => {
              setSession(null);
              setLocked(false);
              setScreen('login');
            }}
          />
        ) : null}
        {nav}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: '#12312b' },
  shell: { flex: 1, backgroundColor: '#f6f2ea' },
  header: {
    backgroundColor: '#12312b',
    paddingHorizontal: 22,
    paddingBottom: 22,
    paddingTop: 18
  },
  kicker: { color: '#f4c86a', fontSize: 12, fontWeight: '700', letterSpacing: 0 },
  title: { color: '#fffaf0', fontSize: 34, fontWeight: '800', letterSpacing: 0, marginTop: 4 },
  subtitle: { color: '#cfe7df', fontSize: 14, marginTop: 6 },
  nav: {
    flexDirection: 'row',
    gap: 10,
    padding: 14,
    backgroundColor: '#fffaf0',
    borderTopColor: '#ded4c1',
    borderTopWidth: 1
  },
  navButton: {
    flex: 1,
    borderRadius: 8,
    paddingVertical: 12,
    alignItems: 'center',
    backgroundColor: '#ece3d2'
  },
  navButtonActive: { backgroundColor: '#12312b' },
  navText: { color: '#3a3328', fontWeight: '700' },
  navTextActive: { color: '#fffaf0' },
  lockOverlay: {
    alignItems: 'center',
    backgroundColor: '#12312b',
    bottom: 0,
    justifyContent: 'center',
    left: 0,
    padding: 28,
    position: 'absolute',
    right: 0,
    top: 0,
    zIndex: 10
  },
  lockTitle: { color: '#fffaf0', fontSize: 26, fontWeight: '900' },
  lockText: { color: '#cfe7df', marginTop: 8, textAlign: 'center' },
  lockButton: { backgroundColor: '#f4c86a', borderRadius: 8, marginTop: 18, paddingHorizontal: 18, paddingVertical: 14 },
  lockButtonText: { color: '#33230f', fontWeight: '900' }
});

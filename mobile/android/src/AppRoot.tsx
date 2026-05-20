import React, { useMemo, useState } from 'react';
import { StatusBar, StyleSheet, Text, TouchableOpacity, View } from 'react-native';

import { ApiClient } from './api/client';
import { AttachmentReviewScreen } from './screens/AttachmentReviewScreen';
import { CaptureHomeScreen } from './screens/CaptureHomeScreen';
import { LoginScreen } from './screens/LoginScreen';
import { MetadataWizardScreen } from './screens/MetadataWizardScreen';
import { SettingsScreen } from './screens/SettingsScreen';
import { SubmissionHistoryScreen } from './screens/SubmissionHistoryScreen';
import type { MobileApi, MobileSubmissionDraft, SessionPayload, SubmissionHistoryItem } from './types';

type Screen = 'login' | 'capture' | 'metadata' | 'review' | 'history' | 'settings';

const today = '2026-05-20';

const emptyDraft = (): MobileSubmissionDraft => ({
  documentName: '',
  categoryId: null,
  folderId: null,
  officeId: null,
  dateReceived: today,
  remarks: '',
  status: 'Filed',
  attachments: []
});

interface AppRootProps {
  api?: MobileApi;
  initialHubUrl?: string;
}

export function AppRoot({ api: injectedApi, initialHubUrl = 'http://192.168.1.10:1421' }: AppRootProps) {
  const [hubUrl, setHubUrl] = useState(initialHubUrl);
  const api = useMemo(() => injectedApi ?? new ApiClient(hubUrl), [hubUrl, injectedApi]);
  const [screen, setScreen] = useState<Screen>('login');
  const [session, setSession] = useState<SessionPayload | null>(null);
  const [draft, setDraft] = useState<MobileSubmissionDraft>(emptyDraft());
  const [history, setHistory] = useState<SubmissionHistoryItem[]>([]);
  const [lastSubmissionId, setLastSubmissionId] = useState<number | null>(null);

  const nav = session ? (
    <View style={styles.nav}>
      {(['capture', 'history', 'settings'] as Screen[]).map((item) => (
        <TouchableOpacity
          accessibilityRole="button"
          key={item}
          onPress={() => setScreen(item)}
          style={[styles.navButton, screen === item && styles.navButtonActive]}
        >
          <Text style={[styles.navText, screen === item && styles.navTextActive]}>
            {item === 'capture' ? 'Capture' : item === 'history' ? 'History' : 'Settings'}
          </Text>
        </TouchableOpacity>
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
            hubUrl={hubUrl}
            onHubUrlChange={setHubUrl}
            onLoggedIn={(nextSession) => {
              setSession(nextSession);
              setScreen('capture');
            }}
          />
        ) : null}
        {screen === 'capture' && session ? (
          <CaptureHomeScreen
            draft={draft}
            lastSubmissionId={lastSubmissionId}
            onAddFile={() =>
              setDraft((current) => ({
                ...current,
                attachments: [
                  ...current.attachments,
                  { uri: 'file:///mobile-capture.pdf', name: 'mobile-capture.pdf', type: 'application/pdf' }
                ]
              }))
            }
            onCapture={() =>
              setDraft((current) => ({
                ...current,
                attachments: [
                  ...current.attachments,
                  { uri: 'file:///camera-capture.jpg', name: 'camera-capture.jpg', type: 'image/jpeg' }
                ]
              }))
            }
            onNext={() => setScreen('metadata')}
          />
        ) : null}
        {screen === 'metadata' && session ? (
          <MetadataWizardScreen
            api={api}
            sessionId={session.session_id}
            draft={draft}
            onBack={() => setScreen('capture')}
            onChange={setDraft}
            onNext={() => setScreen('review')}
          />
        ) : null}
        {screen === 'review' && session ? (
          <AttachmentReviewScreen
            api={api}
            sessionId={session.session_id}
            draft={draft}
            onBack={() => setScreen('metadata')}
            onRemoveAttachment={(index) =>
              setDraft((current) => ({
                ...current,
                attachments: current.attachments.filter((_, itemIndex) => itemIndex !== index)
              }))
            }
            onSubmitted={(id) => {
              setLastSubmissionId(id);
              setDraft(emptyDraft());
              setScreen('capture');
            }}
          />
        ) : null}
        {screen === 'history' && session ? (
          <SubmissionHistoryScreen
            api={api}
            sessionId={session.session_id}
            history={history}
            onHistoryLoaded={setHistory}
          />
        ) : null}
        {screen === 'settings' && session ? (
          <SettingsScreen
            api={api}
            hubUrl={hubUrl}
            onHubUrlChange={setHubUrl}
            onLogout={() => {
              setSession(null);
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
  navTextActive: { color: '#fffaf0' }
});

import React from 'react';
import { Pressable, StyleSheet, Text, View } from 'react-native';

import type { MobileSubmissionDraft } from '../types';

interface CaptureHomeScreenProps {
  captureError?: string;
  draft: MobileSubmissionDraft;
  lastSubmissionId: number | null;
  pendingQueueCount: number;
  onAddFile(): void;
  onCapture(): void;
  onNext(): void;
  onSyncQueue(): void;
}

export function CaptureHomeScreen({
  captureError,
  draft,
  lastSubmissionId,
  pendingQueueCount,
  onAddFile,
  onCapture,
  onNext,
  onSyncQueue
}: CaptureHomeScreenProps) {
  const hasAttachments = draft.attachments.length > 0;
  return (
    <View style={styles.container}>
      <View style={styles.hero}>
        <Text style={styles.heroLabel}>Capture</Text>
        <Text style={styles.heroTitle}>Start with the document, then complete the filing details.</Text>
        <Text style={styles.heroMeta}>{draft.attachments.length} attachment(s) staged</Text>
      </View>
      {lastSubmissionId ? <Text style={styles.success}>Pending review #{lastSubmissionId}</Text> : null}
      {pendingQueueCount > 0 ? (
        <Pressable accessibilityRole="button" onPress={onSyncQueue} style={styles.queueCard}>
          <Text style={styles.queueTitle}>{pendingQueueCount} pending sync item(s)</Text>
          <Text style={styles.queueText}>Tap to retry uploads when office Wi-Fi is available.</Text>
        </Pressable>
      ) : null}
      <View style={styles.actions}>
        <Pressable accessibilityRole="button" onPress={onCapture} style={styles.captureButton}>
          <Text style={styles.captureText}>Camera capture</Text>
        </Pressable>
        <Pressable accessibilityRole="button" onPress={onAddFile} style={styles.secondaryButton}>
          <Text style={styles.secondaryText}>Add file</Text>
        </Pressable>
      </View>
      {draft.attachments.length > 0 ? (
        <View style={styles.stagedList}>
          {draft.attachments.map((file, index) => (
            <View key={`${file.uri}-${index}`} style={styles.stagedItem}>
              <Text numberOfLines={1} style={styles.stagedName}>{file.name}</Text>
              <Text style={styles.stagedType}>{file.type}</Text>
            </View>
          ))}
        </View>
      ) : null}
      {captureError ? <Text style={styles.error}>{captureError}</Text> : null}
      <Pressable
        accessibilityRole="button"
        disabled={!hasAttachments}
        onPress={onNext}
        style={[styles.nextButton, !hasAttachments && styles.nextButtonDisabled]}
      >
        <Text style={styles.nextText}>Next</Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 22, gap: 18 },
  hero: { backgroundColor: '#fffaf0', borderColor: '#ded4c1', borderRadius: 8, borderWidth: 1, padding: 20 },
  heroLabel: { color: '#b7352d', fontSize: 13, fontWeight: '800' },
  heroTitle: { color: '#12312b', fontSize: 26, fontWeight: '900', lineHeight: 32, marginTop: 6 },
  heroMeta: { color: '#5c675e', fontSize: 14, marginTop: 14 },
  success: { backgroundColor: '#d9eadf', borderRadius: 8, color: '#12312b', fontWeight: '800', padding: 12 },
  queueCard: { backgroundColor: '#fff1c2', borderColor: '#e3b936', borderRadius: 8, borderWidth: 1, padding: 14 },
  queueTitle: { color: '#4d3900', fontSize: 15, fontWeight: '900' },
  queueText: { color: '#6a4a00', marginTop: 3 },
  actions: { gap: 12, marginTop: 'auto' },
  captureButton: { alignItems: 'center', backgroundColor: '#12312b', borderRadius: 8, paddingVertical: 22 },
  captureText: { color: '#fffaf0', fontSize: 18, fontWeight: '900' },
  secondaryButton: { alignItems: 'center', backgroundColor: '#f4c86a', borderRadius: 8, paddingVertical: 18 },
  secondaryText: { color: '#33230f', fontSize: 16, fontWeight: '900' },
  stagedList: { gap: 8 },
  stagedItem: { backgroundColor: '#fffaf0', borderColor: '#ded4c1', borderRadius: 8, borderWidth: 1, padding: 12 },
  stagedName: { color: '#12312b', fontWeight: '900' },
  stagedType: { color: '#5c675e', fontSize: 12, marginTop: 2 },
  error: { color: '#b7352d', fontWeight: '800' },
  nextButton: { alignItems: 'center', backgroundColor: '#b7352d', borderRadius: 8, paddingVertical: 16 },
  nextButtonDisabled: { backgroundColor: '#b9afa0' },
  nextText: { color: '#fffaf0', fontSize: 16, fontWeight: '900' }
});

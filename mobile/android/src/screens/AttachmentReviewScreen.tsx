import React, { useState } from 'react';
import { Pressable, StyleSheet, Text, View } from 'react-native';

import { clearDraft, saveQueuedSubmission } from '../storage/drafts';
import type { MobileApi, MobileSubmissionDraft, QueuedSubmission } from '../types';

interface AttachmentReviewScreenProps {
  api: MobileApi;
  sessionId: string;
  draft: MobileSubmissionDraft;
  onBack(): void;
  onRemoveAttachment(index: number): void;
  onQueued(submission: QueuedSubmission): void;
  onSubmitted(id: number): void;
}

const MAX_MOBILE_UPLOAD_BYTES = 50 * 1024 * 1024;
const ALLOWED_TYPES = new Set(['application/pdf', 'image/jpeg', 'image/png', 'text/plain']);

const sizeLabel = (bytes?: number) => {
  if (!bytes) return 'Size not reported';
  if (bytes >= 1024 * 1024) return `${Math.ceil(bytes / (1024 * 1024))} MB`;
  return `${Math.ceil(bytes / 1024)} KB`;
};

const validateDraft = (draft: MobileSubmissionDraft): string | null => {
  const invalidType = draft.attachments.find((file) => !ALLOWED_TYPES.has(file.type));
  if (invalidType) return `${invalidType.name} is not a supported upload type.`;
  const oversized = draft.attachments.find((file) => (file.sizeBytes ?? 0) > MAX_MOBILE_UPLOAD_BYTES);
  if (oversized) return `${oversized.name} exceeds the 50 MB mobile upload limit.`;
  return null;
};

export function AttachmentReviewScreen({
  api,
  sessionId,
  draft,
  onBack,
  onRemoveAttachment,
  onQueued,
  onSubmitted
}: AttachmentReviewScreenProps) {
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState(0);

  const submit = async () => {
    if (draft.attachments.length === 0) {
      setError('Add at least one attachment before submitting.');
      return;
    }
    const validationError = validateDraft(draft);
    if (validationError) {
      setError(validationError);
      return;
    }
    setBusy(true);
    setProgress(25);
    setError('');
    try {
      setProgress(60);
      const result = await api.createSubmission(sessionId, draft);
      setProgress(100);
      await clearDraft();
      onSubmitted(result.mobile_submission_id);
    } catch {
      const queued: QueuedSubmission = {
        clientSubmissionId: draft.clientSubmissionId,
        draft,
        attempts: 1,
        lastError: 'Could not submit. Check the office PC connection and try again.',
        syncStatus: 'pending',
        queuedAt: new Date().toISOString(),
        lastAttemptAt: new Date().toISOString()
      };
      await saveQueuedSubmission(queued);
      onQueued(queued);
      setError('Office PC offline. Saved to retry queue.');
    } finally {
      setBusy(false);
      setProgress(0);
    }
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Review Attachments</Text>
      <View style={styles.summary}>
        <Text style={styles.documentName}>{draft.documentName}</Text>
        <Text style={styles.meta}>{draft.dateReceived} · {draft.status}</Text>
        <Text style={styles.meta}>Client ID {draft.clientSubmissionId}</Text>
      </View>
      {draft.attachments.map((file, index) => (
        <View key={`${file.uri}-${index}`} style={styles.attachment}>
          <View>
            <Text style={styles.fileName}>{file.name}</Text>
            <Text style={styles.fileType}>{file.type} · {sizeLabel(file.sizeBytes)}</Text>
            <Text style={styles.previewText}>Preview ready before desktop review</Text>
          </View>
          <Pressable accessibilityRole="button" onPress={() => onRemoveAttachment(index)} style={styles.removeButton}>
            <Text style={styles.removeText}>Remove</Text>
          </Pressable>
        </View>
      ))}
      {busy ? (
        <View style={styles.progressTrack}>
          <View style={[styles.progressFill, { width: `${progress}%` }]} />
        </View>
      ) : null}
      {error ? <Text style={styles.error}>{error}</Text> : null}
      <View style={styles.footer}>
        <Pressable accessibilityRole="button" onPress={onBack} style={styles.secondaryButton}>
          <Text style={styles.secondaryText}>Back</Text>
        </Pressable>
        <Pressable accessibilityRole="button" onPress={submit} style={styles.primaryButton}>
          <Text style={styles.primaryText}>{busy ? 'Submitting...' : 'Submit to office PC'}</Text>
        </Pressable>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, gap: 14, padding: 22 },
  title: { color: '#12312b', fontSize: 22, fontWeight: '900' },
  summary: { backgroundColor: '#fffaf0', borderColor: '#ded4c1', borderRadius: 8, borderWidth: 1, padding: 16 },
  documentName: { color: '#12312b', fontSize: 18, fontWeight: '900' },
  meta: { color: '#5c675e', marginTop: 4 },
  attachment: {
    alignItems: 'center',
    backgroundColor: '#ffffff',
    borderColor: '#ded4c1',
    borderRadius: 8,
    borderWidth: 1,
    flexDirection: 'row',
    justifyContent: 'space-between',
    padding: 14
  },
  fileName: { color: '#1f2723', fontWeight: '900' },
  fileType: { color: '#5c675e', marginTop: 2 },
  previewText: { color: '#1f6b45', fontSize: 12, fontWeight: '700', marginTop: 4 },
  removeButton: { backgroundColor: '#ece3d2', borderRadius: 8, paddingHorizontal: 12, paddingVertical: 10 },
  removeText: { color: '#3a3328', fontWeight: '800' },
  error: { color: '#b7352d', fontWeight: '800' },
  progressTrack: { backgroundColor: '#ded4c1', borderRadius: 999, height: 8, overflow: 'hidden' },
  progressFill: { backgroundColor: '#12312b', height: 8 },
  footer: { flexDirection: 'row', gap: 10, marginTop: 'auto' },
  primaryButton: { alignItems: 'center', backgroundColor: '#b7352d', borderRadius: 8, flex: 1.4, padding: 15 },
  primaryText: { color: '#fffaf0', fontWeight: '900' },
  secondaryButton: { alignItems: 'center', borderColor: '#12312b', borderRadius: 8, borderWidth: 1, flex: 1, padding: 15 },
  secondaryText: { color: '#12312b', fontWeight: '900' }
});

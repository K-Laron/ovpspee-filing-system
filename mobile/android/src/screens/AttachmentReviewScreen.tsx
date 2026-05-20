import React, { useState } from 'react';
import { StyleSheet, Text, TouchableOpacity, View } from 'react-native';

import { clearDraft } from '../storage/drafts';
import type { MobileApi, MobileSubmissionDraft } from '../types';

interface AttachmentReviewScreenProps {
  api: MobileApi;
  sessionId: string;
  draft: MobileSubmissionDraft;
  onBack(): void;
  onRemoveAttachment(index: number): void;
  onSubmitted(id: number): void;
}

export function AttachmentReviewScreen({
  api,
  sessionId,
  draft,
  onBack,
  onRemoveAttachment,
  onSubmitted
}: AttachmentReviewScreenProps) {
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  const submit = async () => {
    if (draft.attachments.length === 0) {
      setError('Add at least one attachment before submitting.');
      return;
    }
    setBusy(true);
    setError('');
    try {
      const result = await api.createSubmission(sessionId, draft);
      await clearDraft();
      onSubmitted(result.mobile_submission_id);
    } catch {
      setError('Could not submit. Check the office PC connection and try again.');
    } finally {
      setBusy(false);
    }
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Review Attachments</Text>
      <View style={styles.summary}>
        <Text style={styles.documentName}>{draft.documentName}</Text>
        <Text style={styles.meta}>{draft.dateReceived} · {draft.status}</Text>
      </View>
      {draft.attachments.map((file, index) => (
        <View key={`${file.uri}-${index}`} style={styles.attachment}>
          <View>
            <Text style={styles.fileName}>{file.name}</Text>
            <Text style={styles.fileType}>{file.type}</Text>
          </View>
          <TouchableOpacity accessibilityRole="button" onPress={() => onRemoveAttachment(index)} style={styles.removeButton}>
            <Text style={styles.removeText}>Remove</Text>
          </TouchableOpacity>
        </View>
      ))}
      {error ? <Text style={styles.error}>{error}</Text> : null}
      <View style={styles.footer}>
        <TouchableOpacity accessibilityRole="button" onPress={onBack} style={styles.secondaryButton}>
          <Text style={styles.secondaryText}>Back</Text>
        </TouchableOpacity>
        <TouchableOpacity accessibilityRole="button" onPress={submit} style={styles.primaryButton}>
          <Text style={styles.primaryText}>{busy ? 'Submitting...' : 'Submit to office PC'}</Text>
        </TouchableOpacity>
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
  removeButton: { backgroundColor: '#ece3d2', borderRadius: 8, paddingHorizontal: 12, paddingVertical: 10 },
  removeText: { color: '#3a3328', fontWeight: '800' },
  error: { color: '#b7352d', fontWeight: '800' },
  footer: { flexDirection: 'row', gap: 10, marginTop: 'auto' },
  primaryButton: { alignItems: 'center', backgroundColor: '#b7352d', borderRadius: 8, flex: 1.4, padding: 15 },
  primaryText: { color: '#fffaf0', fontWeight: '900' },
  secondaryButton: { alignItems: 'center', borderColor: '#12312b', borderRadius: 8, borderWidth: 1, flex: 1, padding: 15 },
  secondaryText: { color: '#12312b', fontWeight: '900' }
});

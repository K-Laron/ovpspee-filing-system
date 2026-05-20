import React, { useEffect, useState } from 'react';
import { ScrollView, StyleSheet, Text, TextInput, TouchableOpacity, View } from 'react-native';

import type { LookupsPayload, MobileApi, MobileSubmissionDraft } from '../types';

interface MetadataWizardScreenProps {
  api: MobileApi;
  sessionId: string;
  draft: MobileSubmissionDraft;
  onBack(): void;
  onChange(draft: MobileSubmissionDraft): void;
  onNext(): void;
}

const toNumber = (value: string): number | null => {
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
};

export function MetadataWizardScreen({ api, sessionId, draft, onBack, onChange, onNext }: MetadataWizardScreenProps) {
  const [lookups, setLookups] = useState<LookupsPayload | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    api.getLookups(sessionId).then(setLookups).catch(() => setLookups(null));
  }, [api, sessionId]);

  const update = (patch: Partial<MobileSubmissionDraft>) => onChange({ ...draft, ...patch });
  const isComplete =
    draft.documentName.trim().length > 0 &&
    draft.categoryId !== null &&
    draft.folderId !== null &&
    draft.officeId !== null &&
    draft.dateReceived.trim().length > 0 &&
    draft.attachments.length > 0;

  const next = () => {
    if (!isComplete) {
      setError('Complete all required metadata before review.');
      return;
    }
    setError('');
    onNext();
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title}>Full Add Document metadata</Text>
      <TextInput
        accessibilityLabel="Document title"
        onChangeText={(documentName) => update({ documentName })}
        placeholder="Document title"
        style={styles.input}
        value={draft.documentName}
      />
      <Text style={styles.hint}>Category: {lookups?.categories.map((item) => `${item.category_id} ${item.category_name}`).join(', ') || 'Loading'}</Text>
      <TextInput
        accessibilityLabel="Category ID"
        inputMode="numeric"
        onChangeText={(value) => update({ categoryId: toNumber(value) })}
        placeholder="Category ID"
        style={styles.input}
        value={draft.categoryId?.toString() ?? ''}
      />
      <Text style={styles.hint}>Folder: {lookups?.folders.map((item) => `${item.folder_id} ${item.folder_name}`).join(', ') || 'Loading'}</Text>
      <TextInput
        accessibilityLabel="Folder ID"
        inputMode="numeric"
        onChangeText={(value) => update({ folderId: toNumber(value) })}
        placeholder="Folder ID"
        style={styles.input}
        value={draft.folderId?.toString() ?? ''}
      />
      <Text style={styles.hint}>Sender office: {lookups?.offices.map((item) => `${item.office_id} ${item.office_name}`).join(', ') || 'Loading'}</Text>
      <TextInput
        accessibilityLabel="Sender office ID"
        inputMode="numeric"
        onChangeText={(value) => update({ officeId: toNumber(value) })}
        placeholder="Sender office ID"
        style={styles.input}
        value={draft.officeId?.toString() ?? ''}
      />
      <TextInput
        accessibilityLabel="Date received"
        onChangeText={(dateReceived) => update({ dateReceived })}
        placeholder="YYYY-MM-DD"
        style={styles.input}
        value={draft.dateReceived}
      />
      <View style={styles.statusRow}>
        {(['Filed', 'Archived', 'Confidential', 'Other'] as const).map((status) => (
          <TouchableOpacity
            accessibilityRole="button"
            key={status}
            onPress={() => update({ status })}
            style={[styles.statusButton, draft.status === status && styles.statusButtonActive]}
          >
            <Text style={[styles.statusText, draft.status === status && styles.statusTextActive]}>{status}</Text>
          </TouchableOpacity>
        ))}
      </View>
      {draft.status === 'Confidential' ? <Text style={styles.warning}>Confidential files require desktop review before release.</Text> : null}
      <TextInput
        accessibilityLabel="Remarks"
        multiline
        onChangeText={(remarks) => update({ remarks })}
        placeholder="Remarks"
        style={[styles.input, styles.remarks]}
        value={draft.remarks}
      />
      {error ? <Text style={styles.error}>{error}</Text> : null}
      <View style={styles.footer}>
        <TouchableOpacity accessibilityRole="button" onPress={onBack} style={styles.secondaryButton}>
          <Text style={styles.secondaryText}>Back</Text>
        </TouchableOpacity>
        <TouchableOpacity accessibilityRole="button" onPress={next} style={styles.primaryButton}>
          <Text style={styles.primaryText}>Next</Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { gap: 12, padding: 22 },
  title: { color: '#12312b', fontSize: 22, fontWeight: '900' },
  input: { backgroundColor: '#fff', borderColor: '#c9bda6', borderRadius: 8, borderWidth: 1, fontSize: 16, minHeight: 50, paddingHorizontal: 14 },
  remarks: { minHeight: 92, paddingTop: 12, textAlignVertical: 'top' },
  hint: { color: '#5c675e', fontSize: 12 },
  statusRow: { flexDirection: 'row', flexWrap: 'wrap', gap: 8 },
  statusButton: { backgroundColor: '#ece3d2', borderRadius: 8, paddingHorizontal: 12, paddingVertical: 10 },
  statusButtonActive: { backgroundColor: '#12312b' },
  statusText: { color: '#3a3328', fontWeight: '800' },
  statusTextActive: { color: '#fffaf0' },
  warning: { backgroundColor: '#fff1c2', borderRadius: 8, color: '#6a4a00', fontWeight: '700', padding: 12 },
  error: { color: '#b7352d', fontWeight: '800' },
  footer: { flexDirection: 'row', gap: 10, marginTop: 8 },
  primaryButton: { alignItems: 'center', backgroundColor: '#b7352d', borderRadius: 8, flex: 1, padding: 15 },
  primaryText: { color: '#fffaf0', fontWeight: '900' },
  secondaryButton: { alignItems: 'center', borderColor: '#12312b', borderRadius: 8, borderWidth: 1, flex: 1, padding: 15 },
  secondaryText: { color: '#12312b', fontWeight: '900' }
});

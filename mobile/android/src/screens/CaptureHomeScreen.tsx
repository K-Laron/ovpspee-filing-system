import React from 'react';
import { StyleSheet, Text, TouchableOpacity, View } from 'react-native';

import type { MobileSubmissionDraft } from '../types';

interface CaptureHomeScreenProps {
  draft: MobileSubmissionDraft;
  lastSubmissionId: number | null;
  onAddFile(): void;
  onCapture(): void;
  onNext(): void;
}

export function CaptureHomeScreen({ draft, lastSubmissionId, onAddFile, onCapture, onNext }: CaptureHomeScreenProps) {
  return (
    <View style={styles.container}>
      <View style={styles.hero}>
        <Text style={styles.heroLabel}>Capture</Text>
        <Text style={styles.heroTitle}>Start with the document, then complete the filing details.</Text>
        <Text style={styles.heroMeta}>{draft.attachments.length} attachment(s) staged</Text>
      </View>
      {lastSubmissionId ? <Text style={styles.success}>Pending review #{lastSubmissionId}</Text> : null}
      <View style={styles.actions}>
        <TouchableOpacity accessibilityRole="button" onPress={onCapture} style={styles.captureButton}>
          <Text style={styles.captureText}>Camera capture</Text>
        </TouchableOpacity>
        <TouchableOpacity accessibilityRole="button" onPress={onAddFile} style={styles.secondaryButton}>
          <Text style={styles.secondaryText}>Add file</Text>
        </TouchableOpacity>
      </View>
      <TouchableOpacity accessibilityRole="button" onPress={onNext} style={styles.nextButton}>
        <Text style={styles.nextText}>Next</Text>
      </TouchableOpacity>
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
  actions: { gap: 12, marginTop: 'auto' },
  captureButton: { alignItems: 'center', backgroundColor: '#12312b', borderRadius: 8, paddingVertical: 22 },
  captureText: { color: '#fffaf0', fontSize: 18, fontWeight: '900' },
  secondaryButton: { alignItems: 'center', backgroundColor: '#f4c86a', borderRadius: 8, paddingVertical: 18 },
  secondaryText: { color: '#33230f', fontSize: 16, fontWeight: '900' },
  nextButton: { alignItems: 'center', backgroundColor: '#b7352d', borderRadius: 8, paddingVertical: 16 },
  nextText: { color: '#fffaf0', fontSize: 16, fontWeight: '900' }
});

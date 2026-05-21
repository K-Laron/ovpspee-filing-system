import React, { useEffect, useState } from 'react';
import { Pressable, ScrollView, StyleSheet, Text, View } from 'react-native';

import type { MobileApi, QueuedSubmission, SubmissionHistoryItem } from '../types';

interface SubmissionHistoryScreenProps {
  api: MobileApi;
  sessionId: string;
  history: SubmissionHistoryItem[];
  queue: QueuedSubmission[];
  onHistoryLoaded(history: SubmissionHistoryItem[]): void;
  onSyncQueue(): void;
}

export function SubmissionHistoryScreen({
  api,
  sessionId,
  history,
  queue,
  onHistoryLoaded,
  onSyncQueue
}: SubmissionHistoryScreenProps) {
  const [error, setError] = useState('');

  const load = async () => {
    try {
      setError('');
      onHistoryLoaded(await api.listSubmissions(sessionId));
    } catch {
      setError('Could not load mobile submission history.');
    }
  };

  useEffect(() => {
    load();
  }, []);

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <View style={styles.headerRow}>
        <Text style={styles.title}>History</Text>
        <Pressable accessibilityRole="button" onPress={load} style={styles.refreshButton}>
          <Text style={styles.refreshText}>Refresh</Text>
        </Pressable>
      </View>
      {error ? <Text style={styles.error}>{error}</Text> : null}
      {queue.length > 0 ? (
        <View style={styles.queuePanel}>
          <Text style={styles.queueTitle}>Pending sync</Text>
          {queue.map((item) => (
            <Text key={item.clientSubmissionId} style={styles.queueItem}>
              {item.draft.documentName || 'Untitled draft'} · attempt {item.attempts}
            </Text>
          ))}
          <Pressable accessibilityRole="button" onPress={onSyncQueue} style={styles.syncButton}>
            <Text style={styles.syncText}>Retry all</Text>
          </Pressable>
        </View>
      ) : null}
      {history.length === 0 ? <Text style={styles.empty}>No mobile submissions yet.</Text> : null}
      {history.map((item) => (
        <View key={item.mobile_submission_id} style={styles.item}>
          <Text style={styles.name}>{item.document_name}</Text>
          <Text style={[styles.status, item.review_status === 'Rejected' && styles.rejected]}>{item.review_status}</Text>
          {item.rejection_reason ? <Text style={styles.reason}>{item.rejection_reason}</Text> : null}
          <Text style={styles.date}>{item.created_at}</Text>
        </View>
      ))}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { gap: 12, padding: 22 },
  headerRow: { alignItems: 'center', flexDirection: 'row', justifyContent: 'space-between' },
  title: { color: '#12312b', fontSize: 22, fontWeight: '900' },
  refreshButton: { backgroundColor: '#12312b', borderRadius: 8, paddingHorizontal: 14, paddingVertical: 10 },
  refreshText: { color: '#fffaf0', fontWeight: '900' },
  empty: { color: '#5c675e' },
  error: { color: '#b7352d', fontWeight: '800' },
  queuePanel: { backgroundColor: '#fff1c2', borderColor: '#e3b936', borderRadius: 8, borderWidth: 1, gap: 8, padding: 14 },
  queueTitle: { color: '#4d3900', fontWeight: '900' },
  queueItem: { color: '#6a4a00' },
  syncButton: { alignItems: 'center', backgroundColor: '#12312b', borderRadius: 8, marginTop: 4, padding: 10 },
  syncText: { color: '#fffaf0', fontWeight: '900' },
  item: { backgroundColor: '#fffaf0', borderColor: '#ded4c1', borderRadius: 8, borderWidth: 1, padding: 16 },
  name: { color: '#12312b', fontSize: 17, fontWeight: '900' },
  status: { color: '#1f6b45', fontWeight: '900', marginTop: 6 },
  rejected: { color: '#b7352d' },
  reason: { color: '#5a342f', marginTop: 6 },
  date: { color: '#5c675e', marginTop: 8 }
});

import React, { useState } from 'react';
import { StyleSheet, Text, TextInput, TouchableOpacity, View } from 'react-native';

import { saveHubUrl } from '../storage/drafts';
import type { MobileApi } from '../types';

interface SettingsScreenProps {
  api: MobileApi;
  hubUrl: string;
  onHubUrlChange(value: string): void;
  onLogout(): void;
}

export function SettingsScreen({ api, hubUrl, onHubUrlChange, onLogout }: SettingsScreenProps) {
  const [message, setMessage] = useState('');

  const testConnection = async () => {
    try {
      await saveHubUrl(hubUrl);
      await api.health();
      setMessage('Office PC hub is online.');
    } catch {
      setMessage('Office PC hub is not reachable.');
    }
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Settings</Text>
      <TextInput
        accessibilityLabel="Hub URL"
        autoCapitalize="none"
        inputMode="url"
        onChangeText={onHubUrlChange}
        placeholder="http://<office-pc-ip>:1421"
        style={styles.input}
        value={hubUrl}
      />
      <TouchableOpacity accessibilityRole="button" onPress={testConnection} style={styles.secondaryButton}>
        <Text style={styles.secondaryText}>Test connection</Text>
      </TouchableOpacity>
      {message ? <Text style={styles.message}>{message}</Text> : null}
      <TouchableOpacity accessibilityRole="button" onPress={onLogout} style={styles.logoutButton}>
        <Text style={styles.logoutText}>Logout</Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, gap: 14, padding: 22 },
  title: { color: '#12312b', fontSize: 22, fontWeight: '900' },
  input: { backgroundColor: '#fff', borderColor: '#c9bda6', borderRadius: 8, borderWidth: 1, fontSize: 16, minHeight: 50, paddingHorizontal: 14 },
  secondaryButton: { alignItems: 'center', borderColor: '#12312b', borderRadius: 8, borderWidth: 1, padding: 14 },
  secondaryText: { color: '#12312b', fontWeight: '900' },
  message: { color: '#5c675e' },
  logoutButton: { alignItems: 'center', backgroundColor: '#b7352d', borderRadius: 8, marginTop: 'auto', padding: 15 },
  logoutText: { color: '#fffaf0', fontWeight: '900' }
});

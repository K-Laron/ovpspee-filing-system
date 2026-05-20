import React, { useState } from 'react';
import { Alert, StyleSheet, Text, TextInput, TouchableOpacity, View } from 'react-native';

import { saveHubUrl } from '../storage/drafts';
import type { MobileApi, SessionPayload } from '../types';

interface LoginScreenProps {
  api: MobileApi;
  hubUrl: string;
  onHubUrlChange(value: string): void;
  onLoggedIn(session: SessionPayload): void;
}

export function LoginScreen({ api, hubUrl, onHubUrlChange, onLoggedIn }: LoginScreenProps) {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);

  const testConnection = async () => {
    setBusy(true);
    setMessage('');
    try {
      await api.health();
      await saveHubUrl(hubUrl);
      setMessage('Office PC hub is online.');
    } catch {
      setMessage('Office PC hub is not reachable.');
    } finally {
      setBusy(false);
    }
  };

  const login = async () => {
    setBusy(true);
    setMessage('');
    try {
      await saveHubUrl(hubUrl);
      const session = await api.login(username.trim(), password);
      setPassword('');
      onLoggedIn(session);
    } catch {
      Alert.alert('Login failed', 'Check your account and office Wi-Fi.');
      setMessage('Login failed. Check your account and office Wi-Fi.');
    } finally {
      setBusy(false);
    }
  };

  return (
    <View style={styles.container}>
      <Text style={styles.sectionTitle}>Connect to the office PC</Text>
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
        <Text style={styles.secondaryText}>{busy ? 'Checking...' : 'Test connection'}</Text>
      </TouchableOpacity>
      <View style={styles.card}>
        <Text style={styles.cardTitle}>Secretary login</Text>
        <TextInput
          accessibilityLabel="Username"
          autoCapitalize="none"
          onChangeText={setUsername}
          placeholder="Username"
          style={styles.input}
          value={username}
        />
        <TextInput
          accessibilityLabel="Password"
          onChangeText={setPassword}
          placeholder="Password"
          secureTextEntry
          style={styles.input}
          value={password}
        />
        <TouchableOpacity accessibilityRole="button" onPress={login} style={styles.primaryButton}>
          <Text style={styles.primaryText}>{busy ? 'Signing in...' : 'Login'}</Text>
        </TouchableOpacity>
      </View>
      {message ? <Text style={styles.message}>{message}</Text> : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 22, gap: 14 },
  sectionTitle: { color: '#12312b', fontSize: 22, fontWeight: '800' },
  card: { backgroundColor: '#fffaf0', borderColor: '#ded4c1', borderRadius: 8, borderWidth: 1, padding: 16, gap: 12 },
  cardTitle: { color: '#3a3328', fontSize: 18, fontWeight: '800' },
  input: {
    backgroundColor: '#ffffff',
    borderColor: '#c9bda6',
    borderRadius: 8,
    borderWidth: 1,
    color: '#1f2723',
    fontSize: 16,
    minHeight: 50,
    paddingHorizontal: 14
  },
  primaryButton: { alignItems: 'center', backgroundColor: '#b7352d', borderRadius: 8, padding: 15 },
  primaryText: { color: '#fffaf0', fontSize: 16, fontWeight: '800' },
  secondaryButton: { alignItems: 'center', borderColor: '#12312b', borderRadius: 8, borderWidth: 1, padding: 14 },
  secondaryText: { color: '#12312b', fontWeight: '800' },
  message: { color: '#5a4b36', fontSize: 14 }
});

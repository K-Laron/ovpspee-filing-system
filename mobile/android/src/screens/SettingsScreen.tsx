import React, { useState } from 'react';
import { Pressable, StyleSheet, Text, TextInput, View } from 'react-native';

import { saveDeviceProfile, saveHubUrl } from '../storage/drafts';
import type { DeviceProfile, MobileApi } from '../types';

interface SettingsScreenProps {
  api: MobileApi;
  deviceProfile: DeviceProfile;
  hubUrl: string;
  pendingQueueCount: number;
  appVersion: string;
  onDeviceProfileChange(profile: DeviceProfile): void;
  onHubUrlChange(value: string): void;
  onLock(): void;
  onLogout(): void;
}

export function SettingsScreen({
  api,
  appVersion,
  deviceProfile,
  hubUrl,
  pendingQueueCount,
  onDeviceProfileChange,
  onHubUrlChange,
  onLock,
  onLogout
}: SettingsScreenProps) {
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

  const saveDevice = async (profile: DeviceProfile) => {
    onDeviceProfileChange(profile);
    await saveDeviceProfile(profile);
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
      <Pressable accessibilityRole="button" onPress={testConnection} style={styles.secondaryButton}>
        <Text style={styles.secondaryText}>Test connection</Text>
      </Pressable>
      <View style={styles.panel}>
        <Text style={styles.panelTitle}>Approved device</Text>
        <TextInput
          accessibilityLabel="Device ID"
          autoCapitalize="none"
          onChangeText={(deviceId) => void saveDevice({ ...deviceProfile, deviceId })}
          placeholder="Desktop-generated device ID"
          style={styles.input}
          value={deviceProfile.deviceId}
        />
        <TextInput
          accessibilityLabel="Device name"
          onChangeText={(deviceName) => void saveDevice({ ...deviceProfile, deviceName })}
          placeholder="Device name"
          style={styles.input}
          value={deviceProfile.deviceName}
        />
        <TextInput
          accessibilityLabel="Device token"
          autoCapitalize="none"
          onChangeText={(deviceToken) => void saveDevice({ ...deviceProfile, deviceToken })}
          placeholder="Office device token"
          secureTextEntry
          style={styles.input}
          value={deviceProfile.deviceToken}
        />
      </View>
      <View style={styles.panel}>
        <Text style={styles.panelTitle}>App status</Text>
        <Text style={styles.meta}>Version {appVersion}</Text>
        <Text style={styles.meta}>{pendingQueueCount} pending sync item(s)</Text>
        <Text style={styles.meta}>Session auto-locks after 15 minutes idle.</Text>
      </View>
      {message ? <Text style={styles.message}>{message}</Text> : null}
      <Pressable accessibilityRole="button" onPress={onLock} style={styles.secondaryButton}>
        <Text style={styles.secondaryText}>Lock app</Text>
      </Pressable>
      <Pressable accessibilityRole="button" onPress={onLogout} style={styles.logoutButton}>
        <Text style={styles.logoutText}>Logout</Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, gap: 14, padding: 22 },
  title: { color: '#12312b', fontSize: 22, fontWeight: '900' },
  input: { backgroundColor: '#fff', borderColor: '#c9bda6', borderRadius: 8, borderWidth: 1, fontSize: 16, minHeight: 50, paddingHorizontal: 14 },
  secondaryButton: { alignItems: 'center', borderColor: '#12312b', borderRadius: 8, borderWidth: 1, padding: 14 },
  secondaryText: { color: '#12312b', fontWeight: '900' },
  panel: { backgroundColor: '#fffaf0', borderColor: '#ded4c1', borderRadius: 8, borderWidth: 1, gap: 10, padding: 14 },
  panelTitle: { color: '#12312b', fontSize: 16, fontWeight: '900' },
  meta: { color: '#5c675e', fontSize: 13 },
  message: { color: '#5c675e' },
  logoutButton: { alignItems: 'center', backgroundColor: '#b7352d', borderRadius: 8, marginTop: 'auto', padding: 15 },
  logoutText: { color: '#fffaf0', fontWeight: '900' }
});

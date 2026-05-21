CREATE TABLE IF NOT EXISTS mobile_device (
    mobile_device_id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL UNIQUE,
    device_name TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    last_seen_at TEXT,
    created_by INTEGER REFERENCES user(user_id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_mobile_device_active ON mobile_device(is_active);

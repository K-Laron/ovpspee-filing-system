PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS role (
    role_id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS user (
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_id INTEGER NOT NULL REFERENCES role(role_id) ON DELETE RESTRICT,
    first_name TEXT NOT NULL,
    middle_name TEXT,
    last_name TEXT NOT NULL,
    username TEXT NOT NULL UNIQUE COLLATE NOCASE,
    email TEXT UNIQUE COLLATE NOCASE,
    contact_number TEXT,
    address TEXT,
    password_hash TEXT NOT NULL,
    profile_pic_path TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    last_login_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS session (
    session_id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES user(user_id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    expires_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_log (
    log_id INTEGER PRIMARY KEY AUTOINCREMENT,
    log_action TEXT NOT NULL CHECK(log_action IN ('INSERT', 'UPDATE', 'DELETE', 'MOVE', 'LOGIN', 'LOGOUT',
        'BACKUP', 'RESTORE', 'EXPORT', 'IMPORT', 'CLEANUP', 'HIDE', 'UNHIDE', 'TRASH', 'RESTORE_TRASH',
        'PURGE', 'SCAN')),
    table_affected TEXT,
    record_id INTEGER,
    description TEXT NOT NULL,
    user_id INTEGER REFERENCES user(user_id) ON DELETE SET NULL,
    ip_address TEXT,
    timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_session_user ON session(user_id);
CREATE INDEX IF NOT EXISTS idx_session_expires ON session(expires_at);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_log(log_action);

INSERT OR IGNORE INTO role (role_name) VALUES ('Admin'), ('Secretary');

-- TODO(Slice 2+): Add category, folder, office, settings, TRASH seed when master data begins.

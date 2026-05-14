CREATE TABLE IF NOT EXISTS category (
    category_id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    description TEXT,
    color_code TEXT NOT NULL DEFAULT '#64748B',
    icon TEXT,
    is_system INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS folder (
    folder_id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL REFERENCES category(category_id) ON DELETE RESTRICT,
    folder_name TEXT NOT NULL COLLATE NOCASE,
    description TEXT,
    folder_color TEXT NOT NULL DEFAULT '#64748B',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE (category_id, folder_name)
);

CREATE TABLE IF NOT EXISTS office (
    office_id INTEGER PRIMARY KEY AUTOINCREMENT,
    office_name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_folder_category ON folder(category_id);

INSERT OR IGNORE INTO category (category_name, description, color_code, icon, is_system)
VALUES ('TRASH', 'System trash - documents pending permanent deletion', '#64748B', 'Trash2', 1);

INSERT OR IGNORE INTO settings (key, value) VALUES
    ('audit_log_retention_months', '36'),
    ('trash_auto_purge_days', '30'),
    ('backup_schedule', 'disabled'),
    ('backup_time', '02:00'),
    ('backup_destination', 'local_app_data_backups'),
    ('backup_retention_count', '10'),
    ('deleted_scan_retention_days', '30'),
    ('storage_base_dir', '');

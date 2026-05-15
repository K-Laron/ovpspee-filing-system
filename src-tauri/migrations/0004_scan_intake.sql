CREATE TABLE IF NOT EXISTS scan_intake (
    scan_intake_id INTEGER PRIMARY KEY AUTOINCREMENT,
    original_file_name TEXT NOT NULL,
    stored_relative_path TEXT NOT NULL UNIQUE,
    mime_type TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending' CHECK(status IN ('Pending', 'Filed', 'Removed')),
    notes TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_by INTEGER NOT NULL REFERENCES user(user_id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    filed_document_id INTEGER REFERENCES document(document_id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_scan_intake_status ON scan_intake(status);
CREATE INDEX IF NOT EXISTS idx_scan_intake_pending ON scan_intake(status, is_deleted);
CREATE INDEX IF NOT EXISTS idx_scan_intake_deleted ON scan_intake(is_deleted, deleted_at);
CREATE INDEX IF NOT EXISTS idx_scan_intake_document ON scan_intake(filed_document_id);

CREATE TABLE IF NOT EXISTS document (
    document_id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_name TEXT NOT NULL,
    category_id INTEGER NOT NULL REFERENCES category(category_id) ON DELETE RESTRICT,
    folder_id INTEGER REFERENCES folder(folder_id) ON DELETE SET NULL,
    office_id INTEGER REFERENCES office(office_id) ON DELETE SET NULL,
    date_received TEXT NOT NULL,
    date_added TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    remarks TEXT,
    status TEXT NOT NULL DEFAULT 'Filed' CHECK(status IN ('Filed', 'Archived', 'Confidential', 'Other')),
    is_hidden INTEGER NOT NULL DEFAULT 0,
    is_trashed INTEGER NOT NULL DEFAULT 0,
    trashed_at TEXT,
    original_category_id INTEGER REFERENCES category(category_id) ON DELETE SET NULL,
    original_folder_id INTEGER REFERENCES folder(folder_id) ON DELETE SET NULL,
    created_by INTEGER NOT NULL REFERENCES user(user_id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS attachment (
    attachment_id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES document(document_id) ON DELETE CASCADE,
    original_file_name TEXT NOT NULL,
    stored_relative_path TEXT NOT NULL UNIQUE,
    mime_type TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_document_category ON document(category_id);
CREATE INDEX IF NOT EXISTS idx_document_folder ON document(folder_id);
CREATE INDEX IF NOT EXISTS idx_document_office ON document(office_id);
CREATE INDEX IF NOT EXISTS idx_document_date_received ON document(date_received);
CREATE INDEX IF NOT EXISTS idx_document_date_added ON document(date_added);
CREATE INDEX IF NOT EXISTS idx_document_is_hidden ON document(is_hidden);
CREATE INDEX IF NOT EXISTS idx_document_is_trashed ON document(is_trashed);
CREATE INDEX IF NOT EXISTS idx_attachment_document ON attachment(document_id);

CREATE VIRTUAL TABLE IF NOT EXISTS document_fts USING fts5(
    document_name,
    remarks,
    status,
    category_name,
    folder_name,
    office_name
);

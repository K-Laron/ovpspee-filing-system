CREATE TABLE IF NOT EXISTS mobile_submission (
    mobile_submission_id INTEGER PRIMARY KEY AUTOINCREMENT,
    submitted_by INTEGER NOT NULL REFERENCES user(user_id) ON DELETE RESTRICT,
    document_name TEXT NOT NULL,
    category_id INTEGER NOT NULL REFERENCES category(category_id) ON DELETE RESTRICT,
    folder_id INTEGER REFERENCES folder(folder_id) ON DELETE RESTRICT,
    office_id INTEGER REFERENCES office(office_id) ON DELETE RESTRICT,
    date_received TEXT NOT NULL,
    remarks TEXT,
    status TEXT NOT NULL CHECK(status IN ('Filed', 'Archived', 'Confidential', 'Other')),
    review_status TEXT NOT NULL DEFAULT 'Pending' CHECK(review_status IN ('Pending', 'Approved', 'Rejected', 'Removed')),
    rejection_reason TEXT,
    review_notes TEXT,
    reviewed_by INTEGER REFERENCES user(user_id) ON DELETE SET NULL,
    reviewed_at TEXT,
    resulting_document_id INTEGER REFERENCES document(document_id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS mobile_submission_attachment (
    mobile_submission_attachment_id INTEGER PRIMARY KEY AUTOINCREMENT,
    mobile_submission_id INTEGER NOT NULL REFERENCES mobile_submission(mobile_submission_id) ON DELETE CASCADE,
    original_file_name TEXT NOT NULL,
    stored_relative_path TEXT NOT NULL UNIQUE,
    mime_type TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_mobile_submission_review_status ON mobile_submission(review_status);
CREATE INDEX IF NOT EXISTS idx_mobile_submission_submitted_by ON mobile_submission(submitted_by);
CREATE INDEX IF NOT EXISTS idx_mobile_submission_result_document ON mobile_submission(resulting_document_id);
CREATE INDEX IF NOT EXISTS idx_mobile_submission_attachment_submission ON mobile_submission_attachment(mobile_submission_id);

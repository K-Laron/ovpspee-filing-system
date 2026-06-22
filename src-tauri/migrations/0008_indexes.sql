-- Missing FK indexes (SQLite does not auto-index foreign keys)
CREATE INDEX IF NOT EXISTS idx_user_role ON user(role_id);
CREATE INDEX IF NOT EXISTS idx_scan_intake_created_by ON scan_intake(created_by);
CREATE INDEX IF NOT EXISTS idx_mobile_submission_category ON mobile_submission(category_id);
CREATE INDEX IF NOT EXISTS idx_mobile_submission_folder ON mobile_submission(folder_id);
CREATE INDEX IF NOT EXISTS idx_mobile_submission_office ON mobile_submission(office_id);
CREATE INDEX IF NOT EXISTS idx_mobile_submission_reviewed_by ON mobile_submission(reviewed_by);
CREATE INDEX IF NOT EXISTS idx_mobile_device_created_by ON mobile_device(created_by);

-- Compound indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_document_category_date ON document(category_id, date_received DESC);
CREATE INDEX IF NOT EXISTS idx_document_trash_category_date ON document(is_trashed, category_id, date_received DESC);

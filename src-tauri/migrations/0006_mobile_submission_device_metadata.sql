ALTER TABLE mobile_submission ADD COLUMN client_submission_id TEXT;
ALTER TABLE mobile_submission ADD COLUMN submitted_device_id TEXT;
ALTER TABLE mobile_submission ADD COLUMN submitted_device_name TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_mobile_submission_client_submission_id
ON mobile_submission(client_submission_id)
WHERE client_submission_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_mobile_submission_device_id
ON mobile_submission(submitted_device_id);

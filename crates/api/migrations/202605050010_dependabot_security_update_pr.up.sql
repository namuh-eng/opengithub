ALTER TABLE dependabot_alerts
ADD COLUMN IF NOT EXISTS security_update_pull_request_id uuid REFERENCES pull_requests(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS dependabot_alerts_security_update_pr_idx
ON dependabot_alerts (security_update_pull_request_id)
WHERE security_update_pull_request_id IS NOT NULL;

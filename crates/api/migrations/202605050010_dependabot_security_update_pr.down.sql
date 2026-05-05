DROP INDEX IF EXISTS dependabot_alerts_security_update_pr_idx;

ALTER TABLE dependabot_alerts
DROP COLUMN IF EXISTS security_update_pull_request_id;

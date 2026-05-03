DROP INDEX IF EXISTS security_audit_events_signing_keys_idx;
DROP INDEX IF EXISTS gpg_key_emails_email_lower_idx;
DROP TABLE IF EXISTS gpg_key_emails;
DROP TABLE IF EXISTS gpg_keys;
DROP TABLE IF EXISTS ssh_keys;

ALTER TABLE users
    DROP COLUMN IF EXISTS vigilant_mode;

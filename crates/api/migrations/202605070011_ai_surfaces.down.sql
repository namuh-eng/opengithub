DROP INDEX IF EXISTS ai_outputs_scope_latest_idx;
DROP INDEX IF EXISTS ai_outputs_scope_cache_unique;
DROP TABLE IF EXISTS ai_outputs;

ALTER TABLE repositories
    DROP COLUMN IF EXISTS ai_features_enabled;

ALTER TABLE user_settings
    DROP COLUMN IF EXISTS ai_features_enabled;

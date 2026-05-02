DROP TRIGGER IF EXISTS user_reports_set_updated_at ON user_reports;
DROP TABLE IF EXISTS user_reports;

DROP TABLE IF EXISTS user_blocks;

DROP TABLE IF EXISTS user_achievements;

DROP TRIGGER IF EXISTS achievements_set_updated_at ON achievements;
DROP TABLE IF EXISTS achievements;

DROP TABLE IF EXISTS profile_contribution_events;

DROP TRIGGER IF EXISTS profile_contribution_days_set_updated_at ON profile_contribution_days;
DROP TABLE IF EXISTS profile_contribution_days;

DROP TRIGGER IF EXISTS profile_pins_set_updated_at ON profile_pins;
DROP TABLE IF EXISTS profile_pins;

DROP TRIGGER IF EXISTS user_profile_readmes_set_updated_at ON user_profile_readmes;
DROP TABLE IF EXISTS user_profile_readmes;

ALTER TABLE users
    DROP CONSTRAINT IF EXISTS users_profile_visibility_check,
    DROP COLUMN IF EXISTS profile_visibility,
    DROP COLUMN IF EXISTS website_url,
    DROP COLUMN IF EXISTS location,
    DROP COLUMN IF EXISTS company,
    DROP COLUMN IF EXISTS bio;

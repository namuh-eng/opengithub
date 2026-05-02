DROP TABLE IF EXISTS user_reports;
DROP TABLE IF EXISTS user_blocks;
DROP TABLE IF EXISTS user_achievements;
DROP TABLE IF EXISTS achievements;
DROP TABLE IF EXISTS profile_contribution_events;
DROP TABLE IF EXISTS profile_contribution_days;
DROP TABLE IF EXISTS profile_pins;
DROP TABLE IF EXISTS profile_readmes;

ALTER TABLE users
    DROP COLUMN IF EXISTS achievements_enabled,
    DROP COLUMN IF EXISTS private_profile,
    DROP COLUMN IF EXISTS website_url,
    DROP COLUMN IF EXISTS location,
    DROP COLUMN IF EXISTS company,
    DROP COLUMN IF EXISTS bio;

CREATE TABLE IF NOT EXISTS user_settings (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    theme text NOT NULL DEFAULT 'system',
    font_size text NOT NULL DEFAULT 'default',
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT user_settings_theme_check CHECK (
        theme IN ('light', 'dark', 'system', 'dark_dimmed', 'dark_high_contrast')
    ),
    CONSTRAINT user_settings_font_size_check CHECK (
        font_size IN ('small', 'default', 'large')
    )
);

CREATE INDEX IF NOT EXISTS user_settings_updated_at_idx
ON user_settings (updated_at DESC);

DROP TRIGGER IF EXISTS user_settings_set_updated_at ON user_settings;
CREATE TRIGGER user_settings_set_updated_at
BEFORE UPDATE ON user_settings
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

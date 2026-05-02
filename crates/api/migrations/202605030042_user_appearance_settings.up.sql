CREATE TABLE IF NOT EXISTS user_settings (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    theme text NOT NULL DEFAULT 'system',
    font_size text NOT NULL DEFAULT 'medium',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT user_settings_theme_check CHECK (theme IN ('light', 'dark', 'system', 'dark_dimmed', 'dark_high_contrast')),
    CONSTRAINT user_settings_font_size_check CHECK (font_size IN ('small', 'medium', 'large'))
);

CREATE TRIGGER user_settings_set_updated_at
BEFORE UPDATE ON user_settings
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

INSERT INTO user_settings (user_id, theme, font_size)
SELECT id, 'system', 'medium'
FROM users
ON CONFLICT (user_id) DO NOTHING;

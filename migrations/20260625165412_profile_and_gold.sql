-- Add profile and gold columns to users table
ALTER TABLE users ADD COLUMN display_name TEXT;
UPDATE users SET display_name = username WHERE display_name IS NULL;
ALTER TABLE users ADD COLUMN avatar_url TEXT;
ALTER TABLE users ADD COLUMN gold BIGINT NOT NULL DEFAULT 500000;
ALTER TABLE users ADD COLUMN last_weekly_bonus TIMESTAMP NOT NULL DEFAULT now();

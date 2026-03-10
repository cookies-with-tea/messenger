-- Add last_seen_at column to guest_user
ALTER TABLE guest_user ADD COLUMN last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

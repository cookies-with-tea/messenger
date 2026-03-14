-- Add is_online column to guest_user
ALTER TABLE guest_user ADD COLUMN is_online BOOLEAN NOT NULL DEFAULT FALSE;

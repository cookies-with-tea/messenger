-- Add 'audio' to media_type enum
ALTER TYPE media_type ADD VALUE IF NOT EXISTS 'audio';

-- Add media_uuid to message table
ALTER TABLE message ADD COLUMN IF NOT EXISTS media_uuid UUID REFERENCES media(uuid) ON DELETE SET NULL;

-- Index for media_uuid
CREATE INDEX IF NOT EXISTS idx_message_media_uuid ON message(media_uuid);

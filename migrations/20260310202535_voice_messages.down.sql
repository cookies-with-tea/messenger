-- Remove index
DROP INDEX IF EXISTS idx_message_media_uuid;

-- Remove media_uuid column
ALTER TABLE message DROP COLUMN IF EXISTS media_uuid;

-- Note: Removing values from ENUM is not directly supported in PostgreSQL without recreating the type.
-- Since this is a new type, we just leave it or accept it's there.

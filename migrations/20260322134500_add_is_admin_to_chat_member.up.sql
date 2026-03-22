DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='chat_member' AND column_name='is_admin') THEN
        ALTER TABLE chat_member ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT FALSE;
    END IF;
END $$;

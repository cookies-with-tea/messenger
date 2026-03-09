-- ============================================================
-- Messenger: chat, chat_member, message, message_status
-- ============================================================

CREATE TYPE chat_type AS ENUM ('direct', 'group');

-- ----------------------------------------------------------------
-- chat
-- ----------------------------------------------------------------
CREATE TABLE IF NOT EXISTS chat (
    uuid         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name         VARCHAR(128),
    description  TEXT,
    chat_type    chat_type   NOT NULL DEFAULT 'direct',
    created_by   UUID        NOT NULL REFERENCES guest_user(uuid) ON DELETE CASCADE,
    avatar       TEXT,
    is_archived  BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chat_created_by ON chat(created_by);
CREATE INDEX idx_chat_type       ON chat(chat_type);
CREATE INDEX idx_chat_created_at ON chat(created_at DESC);

-- ----------------------------------------------------------------
-- chat_member
-- ----------------------------------------------------------------
CREATE TABLE IF NOT EXISTS chat_member (
    uuid       UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    chat_uuid  UUID        NOT NULL REFERENCES chat(uuid) ON DELETE CASCADE,
    user_uuid  UUID        NOT NULL REFERENCES guest_user(uuid) ON DELETE CASCADE,
    is_admin   BOOLEAN     NOT NULL DEFAULT FALSE,
    joined_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at    TIMESTAMPTZ,
    UNIQUE (chat_uuid, user_uuid)
);

CREATE INDEX idx_chat_member_chat ON chat_member(chat_uuid);
CREATE INDEX idx_chat_member_user ON chat_member(user_uuid);

-- ----------------------------------------------------------------
-- message
-- ----------------------------------------------------------------
CREATE TABLE IF NOT EXISTS message (
    uuid          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    chat_uuid     UUID        NOT NULL REFERENCES chat(uuid) ON DELETE CASCADE,
    sender_uuid   UUID        NOT NULL REFERENCES guest_user(uuid) ON DELETE CASCADE,
    reply_to_uuid UUID        REFERENCES message(uuid) ON DELETE SET NULL,
    body          TEXT        NOT NULL CHECK (char_length(body) BETWEEN 1 AND 4000),
    is_edited     BOOLEAN     NOT NULL DEFAULT FALSE,
    is_deleted    BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_message_chat     ON message(chat_uuid, created_at DESC);
CREATE INDEX idx_message_sender   ON message(sender_uuid);
CREATE INDEX idx_message_reply_to ON message(reply_to_uuid);

-- ----------------------------------------------------------------
-- message_status  (delivered + read per user)
-- ----------------------------------------------------------------
CREATE TYPE delivery_status AS ENUM ('delivered', 'read');

CREATE TABLE IF NOT EXISTS message_status (
    uuid         UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    message_uuid UUID            NOT NULL REFERENCES message(uuid) ON DELETE CASCADE,
    user_uuid    UUID            NOT NULL REFERENCES guest_user(uuid) ON DELETE CASCADE,
    status       delivery_status NOT NULL DEFAULT 'delivered',
    created_at   TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    UNIQUE (message_uuid, user_uuid)
);

CREATE INDEX idx_msg_status_message ON message_status(message_uuid);
CREATE INDEX idx_msg_status_user    ON message_status(user_uuid);

-- ----------------------------------------------------------------
-- Trigger: auto-update updated_at
-- ----------------------------------------------------------------
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN NEW.updated_at = NOW(); RETURN NEW; END;
$$;

CREATE TRIGGER trg_chat_updated_at
    BEFORE UPDATE ON chat FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_message_updated_at
    BEFORE UPDATE ON message FOR EACH ROW EXECUTE FUNCTION set_updated_at();

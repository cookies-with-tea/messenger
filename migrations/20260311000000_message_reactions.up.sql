-- ============================================================
-- Message Reactions
-- ============================================================

CREATE TABLE IF NOT EXISTS message_reaction (
    uuid         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    message_uuid UUID        NOT NULL REFERENCES message(uuid) ON DELETE CASCADE,
    user_uuid    UUID        NOT NULL REFERENCES guest_user(uuid) ON DELETE CASCADE,
    emoji        VARCHAR(32) NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (message_uuid, user_uuid, emoji)
);

CREATE INDEX idx_reaction_message ON message_reaction(message_uuid);
CREATE INDEX idx_reaction_user    ON message_reaction(user_uuid);

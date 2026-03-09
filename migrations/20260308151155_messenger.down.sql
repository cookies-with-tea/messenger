DROP TRIGGER  IF EXISTS trg_message_updated_at ON message;
DROP TRIGGER  IF EXISTS trg_chat_updated_at    ON chat;
DROP FUNCTION IF EXISTS set_updated_at();
DROP TABLE    IF EXISTS message_status;
DROP TABLE    IF EXISTS message;
DROP TABLE    IF EXISTS chat_member;
DROP TABLE    IF EXISTS chat;
DROP TYPE     IF EXISTS delivery_status;
DROP TYPE     IF EXISTS chat_type;

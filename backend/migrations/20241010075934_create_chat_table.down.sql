-- Add down migration script here
DROP INDEX IF EXISTS idx_chat_messages_user_name;

DROP TABLE IF EXISTS chat_messages;


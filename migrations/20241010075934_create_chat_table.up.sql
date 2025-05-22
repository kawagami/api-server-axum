-- Add up migration script here
CREATE TABLE chat_messages(
    id serial PRIMARY KEY,
    message_type varchar(16) NOT NULL,
    to_type varchar(16) NOT NULL,
    user_name varchar(16) NOT NULL,
    message text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_chat_messages_user_name ON chat_messages(user_name);


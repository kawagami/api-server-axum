-- Add down migration script here
-- Down migration: add html column back
ALTER TABLE blogs ADD COLUMN html TEXT NOT NULL DEFAULT '';

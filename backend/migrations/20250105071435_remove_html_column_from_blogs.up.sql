-- Add up migration script here
-- Up migration: remove html column
ALTER TABLE blogs DROP COLUMN html;

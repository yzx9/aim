-- Revert ICS optional changes - restore path-based storage
-- This migration:
-- 1. Removes backend_kind column from events and todos tables
-- 2. Drops resources table
-- 3. Restores path column to events and todos tables

-- Drop resources table first (we need data from it)
DROP INDEX IF EXISTS idx_resources_backend_kind;
DROP TABLE IF EXISTS resources;

-- Add path column to events table by rebuilding
CREATE TABLE events_new (
    uid TEXT PRIMARY KEY,
    path TEXT NOT NULL DEFAULT '',
    summary TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    start TEXT NOT NULL,
    end TEXT NOT NULL
);

INSERT INTO events_new (uid, path, summary, description, status, start, end)
SELECT uid, '', summary, description, status, start, end
FROM events;

DROP TABLE events;
ALTER TABLE events_new RENAME TO events;

-- Add path column to todos table by rebuilding
CREATE TABLE todos_new (
    uid TEXT PRIMARY KEY,
    path TEXT NOT NULL DEFAULT '',
    completed TEXT NOT NULL,
    description TEXT NOT NULL,
    percent INTEGER,
    priority INTEGER NOT NULL,
    status TEXT NOT NULL,
    summary TEXT NOT NULL,
    due TEXT NOT NULL
);

INSERT INTO todos_new (uid, path, completed, description, percent, priority, status, summary, due)
SELECT uid, '', completed, description, percent, priority, status, summary, due
FROM todos;

DROP TABLE todos;
ALTER TABLE todos_new RENAME TO todos;

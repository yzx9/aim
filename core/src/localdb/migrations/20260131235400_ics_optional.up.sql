-- Make ICS support optional with unified resources table for multi-backend support
-- This migration:
-- 1. Adds backend_kind column to events and todos tables
-- 2. Creates unified resources table for all backends
-- 3. Migrates existing path data to resources table
-- 4. Removes path column from events and todos tables

-- Add backend_kind to events table
ALTER TABLE events ADD COLUMN backend_kind TINYINT NOT NULL DEFAULT 0;

-- Add backend_kind to todos table
ALTER TABLE todos ADD COLUMN backend_kind TINYINT NOT NULL DEFAULT 0;

-- Create unified resources table for all backends (local, caldav, jcal, etc.)
CREATE TABLE IF NOT EXISTS resources (
    uid TEXT NOT NULL,
    backend_kind TINYINT NOT NULL,
    resource_id TEXT NOT NULL,
    metadata TEXT,
    PRIMARY KEY (uid, backend_kind)
);

-- Create index for performance
CREATE INDEX IF NOT EXISTS idx_resources_backend_kind ON resources(backend_kind);

-- Migrate existing paths to resources table for local backend
INSERT OR IGNORE INTO resources (uid, backend_kind, resource_id)
SELECT uid, 0, 'file://' || path
FROM events
WHERE path IS NOT NULL AND path != '';

INSERT OR IGNORE INTO resources (uid, backend_kind, resource_id)
SELECT uid, 0, 'file://' || path
FROM todos
WHERE path IS NOT NULL AND path != '';

-- Remove path column from events table by rebuilding
CREATE TABLE events_new (
    uid TEXT PRIMARY KEY,
    summary TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    start TEXT NOT NULL,
    end TEXT NOT NULL,
    backend_kind TINYINT NOT NULL DEFAULT 0
);

INSERT INTO events_new (uid, summary, description, status, start, end, backend_kind)
SELECT uid, summary, description, status, start, end, backend_kind
FROM events;

DROP TABLE events;
ALTER TABLE events_new RENAME TO events;

-- Remove path column from todos table by rebuilding
CREATE TABLE todos_new (
    uid TEXT PRIMARY KEY,
    completed TEXT NOT NULL,
    description TEXT NOT NULL,
    percent INTEGER,
    priority INTEGER NOT NULL,
    status TEXT NOT NULL,
    summary TEXT NOT NULL,
    due TEXT NOT NULL,
    backend_kind TINYINT NOT NULL DEFAULT 0
);

INSERT INTO todos_new (uid, completed, description, percent, priority, status, summary, due, backend_kind)
SELECT uid, completed, description, percent, priority, status, summary, due, backend_kind
FROM todos;

DROP TABLE todos;
ALTER TABLE todos_new RENAME TO todos;

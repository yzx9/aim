-- Add multi-calendar support
-- This migration:
-- 1. Creates calendars table (mirror of config)
-- 2. Adds calendar_id to events/todos (replaces backend_kind)
-- 3. Changes resources PK to (uid, calendar_id)

-- Create calendars table
CREATE TABLE calendars (
    id TEXT PRIMARY KEY,           -- User-defined: 'personal', 'work', etc.
    name TEXT NOT NULL,            -- Display name
    kind TEXT NOT NULL,            -- 'local' or 'caldav'
    priority INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Create index for calendar listing by priority
CREATE INDEX idx_calendars_priority ON calendars(priority);

-- Add calendar_id to events
CREATE TABLE events_new (
    uid TEXT PRIMARY KEY,
    calendar_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    start TEXT NOT NULL,
    end TEXT NOT NULL
);

INSERT INTO events_new (uid, calendar_id, summary, description, status, start, end)
SELECT uid, 'default', summary, description, status, start, end
FROM events;

DROP TABLE events;
ALTER TABLE events_new RENAME TO events;

-- Add calendar_id to todos
CREATE TABLE todos_new (
    uid TEXT PRIMARY KEY,
    calendar_id TEXT NOT NULL,
    completed TEXT NOT NULL,
    description TEXT NOT NULL,
    percent INTEGER,
    priority INTEGER NOT NULL,
    status TEXT NOT NULL,
    summary TEXT NOT NULL,
    due TEXT NOT NULL
);

INSERT INTO todos_new (uid, calendar_id, completed, description, percent, priority, status, summary, due)
SELECT uid, 'default', completed, description, percent, priority, status, summary, due
FROM todos;

DROP TABLE todos;
ALTER TABLE todos_new RENAME TO todos;

-- Rebuild resources with (uid, calendar_id) as PK
CREATE TABLE resources_new (
    uid TEXT NOT NULL,
    calendar_id TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    metadata TEXT,
    PRIMARY KEY (uid, calendar_id)
);

INSERT INTO resources_new (uid, calendar_id, resource_id, metadata)
SELECT uid, 'default', resource_id, metadata
FROM resources;

DROP TABLE resources;
ALTER TABLE resources_new RENAME TO resources;

-- Create index for resource lookup by calendar
CREATE INDEX idx_resources_calendar ON resources(calendar_id);

-- Create default calendar for existing data
INSERT INTO calendars (id, name, kind, priority, enabled, created_at, updated_at)
VALUES ('default', 'Default', 'local', 0, 1, datetime('now'), datetime('now'));

-- Revert multi-calendar support
-- This migration:
-- 1. Drops calendars table
-- 2. Restores backend_kind to events/todos (removes calendar_id)
-- 3. Restores resources PK to (uid, backend_kind)

-- Drop calendars table
DROP TABLE calendars;

-- Restore events with backend_kind (remove calendar_id)
CREATE TABLE events_old (
    uid TEXT PRIMARY KEY,
    summary TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    start TEXT NOT NULL,
    end TEXT NOT NULL,
    backend_kind TINYINT NOT NULL DEFAULT 0
);

INSERT INTO events_old (uid, summary, description, status, start, end, backend_kind)
SELECT uid, summary, description, status, start, end, 0
FROM events;

DROP TABLE events;
ALTER TABLE events_old RENAME TO events;

-- Restore todos with backend_kind (remove calendar_id)
CREATE TABLE todos_old (
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

INSERT INTO todos_old (uid, completed, description, percent, priority, status, summary, due, backend_kind)
SELECT uid, completed, description, percent, priority, status, summary, due, 0
FROM todos;

DROP TABLE todos;
ALTER TABLE todos_old RENAME TO todos;

-- Restore resources with (uid, backend_kind) as PK
CREATE TABLE resources_old (
    uid TEXT NOT NULL,
    backend_kind TINYINT NOT NULL,
    resource_id TEXT NOT NULL,
    metadata TEXT,
    PRIMARY KEY (uid, backend_kind)
);

INSERT INTO resources_old (uid, backend_kind, resource_id, metadata)
SELECT uid, 0, resource_id, metadata
FROM resources;

DROP TABLE resources;
ALTER TABLE resources_old RENAME TO resources;

-- Recreate index for backend_kind
CREATE INDEX idx_resources_backend_kind ON resources(backend_kind);

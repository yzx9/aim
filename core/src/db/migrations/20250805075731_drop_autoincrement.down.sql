-- Add down migration script here

-- Rename the old table
ALTER TABLE short_ids RENAME TO short_ids_no_autoinc;

-- Create the new table with AUTOINCREMENT
CREATE TABLE short_ids (
    short_id INTEGER PRIMARY KEY AUTOINCREMENT,
    uid      TEXT UNIQUE NOT NULL,
    kind     TEXT NOT NULL
);

-- Copy data over
INSERT INTO short_ids (short_id, uid, kind)
SELECT short_id, uid, kind FROM short_ids_no_autoinc;

-- Drop the old table
DROP TABLE short_ids_no_autoinc;

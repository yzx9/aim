-- Add up migration script here

-- Create the events table
CREATE TABLE IF NOT EXISTS events (
    uid         TEXT PRIMARY KEY,
    path        TEXT NOT NULL,
    summary     TEXT NOT NULL,
    description TEXT NOT NULL,
    status      TEXT NOT NULL,
    start       TEXT NOT NULL,
    end         TEXT NOT NULL
);

-- Create the todos table
CREATE TABLE IF NOT EXISTS todos (
    uid         TEXT PRIMARY KEY,
    path        TEXT NOT NULL,
    completed   TEXT NOT NULL,
    description TEXT NOT NULL,
    percent     INTEGER,
    priority    INTEGER NOT NULL,
    status      TEXT NOT NULL,
    summary     TEXT NOT NULL,
    due         TEXT NOT NULL
);

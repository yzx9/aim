-- Add up migration script here

CREATE TABLE short_ids (
    short_id INTEGER PRIMARY KEY AUTOINCREMENT,
    uid      TEXT UNIQUE NOT NULL,
    kind     TEXT NOT NULL
);

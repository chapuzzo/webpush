-- Add migration script here
CREATE TABLE IF NOT EXISTS subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    endpoint TEXT NOT NULL,
    keys JSON NOT NULL,
    user_id TEXT
);

-- Your SQL goes here
CREATE TABLE records (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  chat_type TEXT NOT NULL,
  owner_id TEXT NOT NULL,
  group_id TEXT NOT NULL,
  sender_id TEXT NOT NULL,
  sender_name TEXT NOT NULL,
  content TEXT NOT NULL,
  timestamp BIGINT NOT NULL,
  metadata BLOB
);
CREATE INDEX "records_idx" ON "records" ("chat_type", "owner_id", "group_id", "timestamp");
-- Your SQL goes here
CREATE TABLE attachments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  record_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  hash BIGINT NOT NULL
);
CREATE TABLE blobs (
  hash BIGINT PRIMARY KEY NOT NULL,
  blob BLOB NOT NULL
);
CREATE INDEX "attachments_idx" ON "attachments" ("record_id", "hash");
CREATE INDEX "blobs_idx" ON "blobs" ("hash");
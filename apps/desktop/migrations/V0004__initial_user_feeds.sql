CREATE TABLE user_feeds (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT,
  pinned INTEGER NOT NULL DEFAULT 0,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  feed_id INTEGER NOT NULL REFERENCES feeds (id) ON DELETE RESTRICT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (user_id, feed_id)
);

CREATE TABLE user_feed_entries (
  id TEXT NOT NULL PRIMARY KEY,
  has_read INTEGER NOT NULL DEFAULT 0,
  user_feed_id TEXT NOT NULL REFERENCES user_feeds (id) ON DELETE CASCADE,
  feed_entry_id INTEGER NOT NULL REFERENCES feed_entries (id) ON DELETE RESTRICT,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (user_feed_id, feed_entry_id)
);
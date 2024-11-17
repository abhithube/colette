CREATE TABLE feeds (
  id INTEGER NOT NULL PRIMARY KEY,
  link TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  url TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE feed_entries (
  id INTEGER NOT NULL PRIMARY KEY,
  link TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  published_at TEXT NOT NULL,
  description TEXT,
  author TEXT,
  thumbnail_url TEXT,
  feed_id INTEGER NOT NULL REFERENCES feeds (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (feed_id, link)
);
CREATE TABLE users (
  id BLOB NOT NULL PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  display_name TEXT,
  image_url TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE accounts (
  id BLOB NOT NULL PRIMARY KEY,
  sub TEXT NOT NULL,
  provider TEXT NOT NULL,
  password_hash TEXT,
  user_id BLOB NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (provider, sub)
);

CREATE TABLE api_keys (
  id BLOB NOT NULL PRIMARY KEY,
  lookup_hash TEXT NOT NULL UNIQUE,
  verification_hash TEXT NOT NULL,
  title TEXT NOT NULL,
  preview TEXT NOT NULL,
  user_id BLOB NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE jobs (
  id BLOB NOT NULL PRIMARY KEY,
  job_type TEXT NOT NULL,
  data_json TEXT,
  status TEXT NOT NULL DEFAULT 'pending',
  group_identifier TEXT,
  message TEXT,
  created_at TEXT NOT NULL,
  completed_at TEXT
);

CREATE TABLE feeds (
  id BLOB NOT NULL PRIMARY KEY,
  source_url TEXT NOT NULL UNIQUE,
  link TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT,
  refresh_interval_min INTEGER NOT NULL DEFAULT 60,
  is_refreshing INTEGER NOT NULL DEFAULT 0,
  refreshed_at TEXT,
  is_custom INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE feed_entries (
  id BLOB NOT NULL PRIMARY KEY,
  link TEXT NOT NULL,
  title TEXT NOT NULL,
  published_at TEXT NOT NULL,
  description TEXT,
  author TEXT,
  thumbnail_url TEXT,
  feed_id BLOB NOT NULL REFERENCES feeds (id) ON DELETE CASCADE,
  UNIQUE (feed_id, link)
);

CREATE TABLE subscriptions (
  id BLOB NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  user_id BLOB NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  feed_id BLOB NOT NULL REFERENCES feeds (id) ON DELETE RESTRICT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (user_id, feed_id)
);

CREATE TABLE read_entries (
  subscription_id BLOB NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
  feed_entry_id BLOB NOT NULL REFERENCES feed_entries (id) ON DELETE RESTRICT,
  user_id BLOB NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  PRIMARY KEY (subscription_id, feed_entry_id)
);

CREATE TABLE bookmarks (
  id BLOB NOT NULL PRIMARY KEY,
  link TEXT NOT NULL,
  title TEXT NOT NULL,
  thumbnail_url TEXT,
  published_at TEXT,
  author TEXT,
  archived_path TEXT,
  user_id BLOB NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (user_id, link)
);

CREATE TABLE tags (
  id BLOB NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  user_id BLOB NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (user_id, title)
);

CREATE TABLE subscription_tags (
  subscription_id BLOB NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
  tag_id BLOB NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  user_id BLOB NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  PRIMARY KEY (subscription_id, tag_id)
);

CREATE TABLE bookmark_tags (
  bookmark_id BLOB NOT NULL REFERENCES bookmarks (id) ON DELETE CASCADE,
  tag_id BLOB NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  user_id BLOB NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  PRIMARY KEY (bookmark_id, tag_id)
);

CREATE TABLE streams (
  id BLOB NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  filter_json TEXT NOT NULL,
  user_id BLOB NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (user_id, title)
);

CREATE TABLE collections (
  id BLOB NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  filter_json TEXT NOT NULL,
  user_id BLOB NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (user_id, title)
);

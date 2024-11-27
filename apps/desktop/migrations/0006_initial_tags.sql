CREATE TABLE tags (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  profile_id TEXT NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (profile_id, title)
);

CREATE TABLE profile_feed_tags (
  profile_feed_id TEXT NOT NULL REFERENCES profile_feeds (id) ON DELETE CASCADE,
  tag_id TEXT NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  profile_id TEXT NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (profile_feed_id, tag_id)
);

CREATE TABLE profile_bookmark_tags (
  profile_bookmark_id TEXT NOT NULL REFERENCES profile_bookmarks (id) ON DELETE CASCADE,
  tag_id TEXT NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  profile_id TEXT NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (profile_bookmark_id, tag_id)
);
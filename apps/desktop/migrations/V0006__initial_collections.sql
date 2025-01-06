CREATE TABLE collections (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  folder_id TEXT REFERENCES folders (id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_bookmarks (
  id TEXT NOT NULL PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  bookmark_id INTEGER NOT NULL REFERENCES bookmarks (id) ON DELETE RESTRICT,
  collection_id UUID REFERENCES collections (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (user_id, bookmark_id)
);

CREATE UNIQUE INDEX user_bookmarks_collection_id_bookmark_id_key ON user_bookmarks (collection_id, bookmark_id) WHERE collection_id IS NOT NULL;
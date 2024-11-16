CREATE TABLE users (
  id TEXT NOT NULL PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE profiles (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  image_url TEXT,
  is_default INTEGER NOT NULL DEFAULT 0,
  user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (user_id, title)
);

CREATE UNIQUE INDEX profiles_user_id_is_default_key ON profiles (user_id, is_default) WHERE is_default;
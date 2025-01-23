CREATE TABLE users (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  email TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE sessions (
  id TEXT NOT NULL PRIMARY KEY,
  data BYTEA NOT NULL,
  expiry_date TIMESTAMPTZ NOT NULL
);

CREATE TABLE feeds (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  link TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  xml_url TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE feed_entries (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  link TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  published_at TIMESTAMPTZ NOT NULL,
  description TEXT,
  author TEXT,
  thumbnail_url TEXT,
  feed_id UUID NOT NULL REFERENCES feeds (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (feed_id, link)
);

CREATE TABLE bookmarks (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  link TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  thumbnail_url TEXT,
  published_at TIMESTAMPTZ,
  author TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE folders (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT NOT NULL,
  parent_id UUID REFERENCES folders (id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, parent_id, title)
);

CREATE TABLE user_feeds (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT,
  folder_id UUID REFERENCES folders (id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  feed_id UUID NOT NULL REFERENCES feeds (id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, feed_id)
);

CREATE TABLE user_feed_entries (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  has_read BOOLEAN NOT NULL DEFAULT FALSE,
  user_feed_id UUID NOT NULL REFERENCES user_feeds (id) ON DELETE CASCADE,
  feed_entry_id UUID NOT NULL REFERENCES feed_entries (id) ON DELETE RESTRICT,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_feed_id, feed_entry_id)
);

CREATE TABLE user_bookmarks (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT,
  thumbnail_url TEXT,
  published_at TIMESTAMPTZ,
  author TEXT,
  folder_id UUID REFERENCES folders (id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  bookmark_id UUID NOT NULL REFERENCES bookmarks (id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, bookmark_id)
);

CREATE TABLE tags (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT NOT NULL,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, title)
);

CREATE TABLE user_feed_tags (
  user_feed_id UUID NOT NULL REFERENCES user_feeds (id) ON DELETE CASCADE,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_feed_id, tag_id)
);

CREATE TABLE user_bookmark_tags (
  user_bookmark_id UUID NOT NULL REFERENCES user_bookmarks (id) ON DELETE CASCADE,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_bookmark_id, tag_id)
);
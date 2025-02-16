CREATE FUNCTION set_updated_at () returns trigger AS $$ begin
  NEW.updated_at := now();
  return NEW;
end; $$ language plpgsql;

CREATE TABLE users (
  id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  email TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER set_updated_at_users before
UPDATE ON users FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TABLE api_keys (
  id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  lookup_hash TEXT NOT NULL UNIQUE,
  verification_hash TEXT NOT NULL,
  title TEXT NOT NULL,
  preview TEXT NOT NULL,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER set_updated_at_api_keys before
UPDATE ON api_keys FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TABLE feeds (
  id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  link TEXT NOT NULL UNIQUE,
  xml_url TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER set_updated_at_feeds before
UPDATE ON feeds FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TABLE feed_entries (
  id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  link TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  published_at TIMESTAMPTZ NOT NULL,
  description TEXT,
  author TEXT,
  thumbnail_url TEXT,
  feed_id uuid NOT NULL REFERENCES feeds (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (feed_id, link)
);

CREATE TRIGGER set_updated_at_feed_entries before
UPDATE ON feed_entries FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TABLE folders (
  id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  title TEXT NOT NULL,
  parent_id uuid REFERENCES folders (id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, parent_id, title)
);

CREATE TRIGGER set_updated_at_folders before
UPDATE ON folders FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TABLE user_feeds (
  id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  title TEXT NOT NULL,
  folder_id uuid REFERENCES folders (id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  feed_id uuid NOT NULL REFERENCES feeds (id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, feed_id)
);

CREATE TRIGGER set_updated_at_user_feeds before
UPDATE ON user_feeds FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TABLE user_feed_entries (
  id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  has_read BOOLEAN NOT NULL DEFAULT FALSE,
  user_feed_id uuid NOT NULL REFERENCES user_feeds (id) ON DELETE CASCADE,
  feed_entry_id uuid NOT NULL REFERENCES feed_entries (id) ON DELETE RESTRICT,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_feed_id, feed_entry_id)
);

CREATE TRIGGER set_updated_at_user_feed_entries before
UPDATE ON user_feed_entries FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TABLE bookmarks (
  id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  link TEXT NOT NULL,
  title TEXT NOT NULL,
  thumbnail_url TEXT,
  published_at TIMESTAMPTZ,
  author TEXT,
  archived_path TEXT,
  folder_id uuid REFERENCES folders (id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, link)
);

CREATE TRIGGER set_updated_at_bookmarks before
UPDATE ON bookmarks FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TABLE tags (
  id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  title TEXT NOT NULL,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, title)
);

CREATE TRIGGER set_updated_at_tags before
UPDATE ON tags FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TABLE user_feed_tags (
  user_feed_id uuid NOT NULL REFERENCES user_feeds (id) ON DELETE CASCADE,
  tag_id uuid NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_feed_id, tag_id)
);

CREATE TRIGGER set_updated_at_user_feed_tags before
UPDATE ON user_feed_tags FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TABLE bookmark_tags (
  bookmark_id uuid NOT NULL REFERENCES bookmarks (id) ON DELETE CASCADE,
  tag_id uuid NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (bookmark_id, tag_id)
);

CREATE TRIGGER set_updated_at_bookmark_tags before
UPDATE ON bookmark_tags FOR each ROW
EXECUTE procedure set_updated_at ();

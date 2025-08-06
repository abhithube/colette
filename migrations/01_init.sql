CREATE OR REPLACE FUNCTION set_updated_at () RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TABLE users (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuidv7 (),
  email TEXT NOT NULL UNIQUE,
  display_name TEXT,
  image_url TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER users_updated_at BEFORE
UPDATE ON users FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE accounts (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuidv7 (),
  sub TEXT NOT NULL,
  provider TEXT NOT NULL,
  password_hash TEXT,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (provider, sub)
);

CREATE TRIGGER accounts_updated_at BEFORE
UPDATE ON accounts FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE api_keys (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuidv7 (),
  lookup_hash TEXT NOT NULL UNIQUE,
  verification_hash TEXT NOT NULL,
  title TEXT NOT NULL,
  preview TEXT NOT NULL,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER api_keys_updated_at BEFORE
UPDATE ON api_keys FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE jobs (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuidv7 (),
  job_type TEXT NOT NULL,
  data_json JSONB NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  group_identifier TEXT,
  message TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER jobs_updated_at BEFORE
UPDATE ON jobs FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE feeds (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuidv7 (),
  source_url TEXT NOT NULL UNIQUE,
  link TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT,
  refresh_interval_min INTEGER NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  refreshed_at TIMESTAMPTZ,
  is_custom BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER feeds_updated_at BEFORE
UPDATE ON feeds FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE feed_entries (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuidv7 (),
  link TEXT NOT NULL,
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

CREATE TRIGGER feed_entries_updated_at BEFORE
UPDATE ON feed_entries FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE subscriptions (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuidv7 (),
  title TEXT NOT NULL,
  description TEXT,
  feed_id UUID NOT NULL REFERENCES feeds (id) ON DELETE RESTRICT,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, feed_id)
);

CREATE TRIGGER subscriptions_updated_at BEFORE
UPDATE ON subscriptions FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE subscription_entries (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuidv7 (),
  has_read BOOLEAN NOT NULL DEFAULT FALSE,
  read_at TIMESTAMPTZ,
  subscription_id UUID NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
  feed_entry_id UUID NOT NULL REFERENCES feed_entries (id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (subscription_id, feed_entry_id)
);

CREATE TRIGGER subscription_entries_updated_at BEFORE
UPDATE ON subscription_entries FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE bookmarks (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuidv7 (),
  link TEXT NOT NULL,
  title TEXT NOT NULL,
  thumbnail_url TEXT,
  published_at TIMESTAMPTZ,
  author TEXT,
  archived_path TEXT,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, link)
);

CREATE TRIGGER bookmarks_updated_at BEFORE
UPDATE ON bookmarks FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE tags (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuidv7 (),
  title TEXT NOT NULL,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, title)
);

CREATE TRIGGER tags_updated_at BEFORE
UPDATE ON tags FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE subscription_tags (
  subscription_id UUID NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (subscription_id, tag_id)
);

CREATE TRIGGER subscription_tags_updated_at BEFORE
UPDATE ON subscription_tags FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE bookmark_tags (
  bookmark_id UUID NOT NULL REFERENCES bookmarks (id) ON DELETE CASCADE,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (bookmark_id, tag_id)
);

CREATE TRIGGER bookmark_tags_updated_at BEFORE
UPDATE ON bookmark_tags FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

CREATE TABLE collections (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuidv7 (),
  title TEXT NOT NULL,
  description TEXT,
  filter_json JSONB NOT NULL,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, title)
);

CREATE TRIGGER collections_updated_at BEFORE
UPDATE ON collections FOR EACH ROW
EXECUTE PROCEDURE set_updated_at ();

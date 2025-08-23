CREATE OR REPLACE FUNCTION set_updated_at () RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TABLE users (
  id UUID NOT NULL PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  verified BOOL NOT NULL DEFAULT FALSE,
  display_name TEXT,
  image_url TEXT,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE otp_codes (
  code TEXT NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  used_at TIMESTAMPTZ,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (user_id, code)
);

CREATE TABLE social_accounts (
  provider TEXT NOT NULL,
  sub TEXT NOT NULL,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (provider, sub),
  UNIQUE (user_id, provider)
);

CREATE TABLE personal_access_tokens (
  id UUID NOT NULL PRIMARY KEY,
  lookup_hash TEXT NOT NULL UNIQUE,
  verification_hash TEXT NOT NULL,
  title TEXT NOT NULL,
  preview TEXT NOT NULL,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE feeds (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
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
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
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
  id UUID NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  feed_id UUID NOT NULL REFERENCES feeds (id) ON DELETE RESTRICT,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  UNIQUE (user_id, feed_id)
);

CREATE TABLE subscription_entries (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  has_read BOOLEAN NOT NULL DEFAULT FALSE,
  read_at TIMESTAMPTZ,
  subscription_id UUID NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
  feed_entry_id UUID NOT NULL REFERENCES feed_entries (id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (subscription_id, feed_entry_id)
);

CREATE TABLE bookmarks (
  id UUID NOT NULL PRIMARY KEY,
  link TEXT NOT NULL,
  title TEXT NOT NULL,
  thumbnail_url TEXT,
  published_at TIMESTAMPTZ,
  author TEXT,
  archived_path TEXT,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  UNIQUE (user_id, link)
);

CREATE TABLE tags (
  id UUID NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  UNIQUE (user_id, title)
);

CREATE TABLE subscription_tags (
  subscription_id UUID NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (subscription_id, tag_id)
);

CREATE TABLE bookmark_tags (
  bookmark_id UUID NOT NULL REFERENCES bookmarks (id) ON DELETE CASCADE,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (bookmark_id, tag_id)
);

CREATE TABLE collections (
  id UUID NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  filter_json JSONB NOT NULL,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  UNIQUE (user_id, title)
);

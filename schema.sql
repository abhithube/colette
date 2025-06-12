CREATE TABLE users (
  id uuid NOT NULL PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  display_name TEXT,
  image_url TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE accounts (
  id uuid NOT NULL PRIMARY KEY,
  sub TEXT NOT NULL,
  provider TEXT NOT NULL,
  password_hash TEXT,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (provider, sub)
);

CREATE TABLE api_keys (
  id uuid NOT NULL PRIMARY KEY,
  lookup_hash TEXT NOT NULL UNIQUE,
  verification_hash TEXT NOT NULL,
  title TEXT NOT NULL,
  preview TEXT NOT NULL,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE jobs (
  id uuid NOT NULL PRIMARY KEY,
  job_type TEXT NOT NULL,
  data_json JSONB NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  group_identifier TEXT,
  message TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  completed_at TIMESTAMPTZ
);

CREATE TABLE feeds (
  id uuid NOT NULL PRIMARY KEY,
  source_url TEXT NOT NULL UNIQUE,
  link TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT,
  refreshed_at TIMESTAMPTZ,
  is_custom BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE feed_entries (
  id uuid NOT NULL PRIMARY KEY,
  link TEXT NOT NULL,
  title TEXT NOT NULL,
  published_at TIMESTAMPTZ NOT NULL,
  description TEXT,
  author TEXT,
  thumbnail_url TEXT,
  feed_id uuid NOT NULL REFERENCES feeds (id) ON DELETE CASCADE,
  UNIQUE (feed_id, link)
);

CREATE TABLE subscriptions (
  id uuid NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  feed_id uuid NOT NULL REFERENCES feeds (id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, feed_id)
);

CREATE TABLE read_entries (
  subscription_id uuid NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
  feed_entry_id uuid NOT NULL REFERENCES feed_entries (id) ON DELETE RESTRICT,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (subscription_id, feed_entry_id)
);

CREATE TABLE bookmarks (
  id uuid NOT NULL PRIMARY KEY,
  link TEXT NOT NULL,
  title TEXT NOT NULL,
  thumbnail_url TEXT,
  published_at TIMESTAMPTZ,
  author TEXT,
  archived_path TEXT,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, link)
);

CREATE TABLE tags (
  id uuid NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, title)
);

CREATE TABLE subscription_tags (
  subscription_id uuid NOT NULL REFERENCES subscriptions (id) ON DELETE CASCADE,
  tag_id uuid NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  PRIMARY KEY (subscription_id, tag_id)
);

CREATE TABLE bookmark_tags (
  bookmark_id uuid NOT NULL REFERENCES bookmarks (id) ON DELETE CASCADE,
  tag_id uuid NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  PRIMARY KEY (bookmark_id, tag_id)
);

CREATE TABLE streams (
  id uuid NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  filter_json JSONB NOT NULL,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, title)
);

CREATE TABLE collections (
  id uuid NOT NULL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  filter_json JSONB NOT NULL,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, title)
);

CREATE TABLE feeds (
  id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  link text NOT NULL UNIQUE,
  title text NOT NULL,
  url text
);

CREATE TABLE entries (
  id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  link text NOT NULL UNIQUE,
  title text NOT NULL,
  published_at timestamptz,
  description text,
  author text,
  thumbnail_url text
);

CREATE TABLE feed_entries (
  id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  feed_id integer NOT NULL REFERENCES feeds (id) ON DELETE cascade,
  entry_id integer NOT NULL REFERENCES entries (id) ON DELETE cascade,
  UNIQUE (feed_id, entry_id)
);

CREATE TABLE users (
  id text NOT NULL PRIMARY KEY,
  email text NOT NULL UNIQUE,
  password text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE profiles (
  id text NOT NULL PRIMARY KEY,
  title text NOT NULL,
  image_url text,
  is_default boolean NOT NULL DEFAULT FALSE,
  user_id text NOT NULL REFERENCES users (id) ON DELETE cascade,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (user_id, is_default)
);

CREATE TABLE profile_feeds (
  id text NOT NULL PRIMARY KEY,
  custom_title text,
  profile_id text NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  feed_id integer NOT NULL REFERENCES feeds (id) ON DELETE restrict,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (profile_id, feed_id)
);

CREATE TABLE profile_feed_entries (
  id text NOT NULL PRIMARY KEY,
  has_read boolean NOT NULL DEFAULT FALSE,
  profile_feed_id text NOT NULL REFERENCES profile_feeds (id) ON DELETE cascade,
  feed_entry_id integer NOT NULL REFERENCES feed_entries (id) ON DELETE restrict,
  UNIQUE (profile_feed_id, feed_entry_id)
);
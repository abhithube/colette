CREATE TABLE feeds (
  id integer NOT NULL PRIMARY KEY,
  link text NOT NULL UNIQUE,
  title text NOT NULL,
  url text
);

CREATE TABLE entries (
  id integer NOT NULL PRIMARY KEY,
  link text NOT NULL UNIQUE,
  title text NOT NULL,
  published_at text,
  description text,
  author text,
  thumbnail_url text
);

CREATE TABLE feed_entries (
  id integer NOT NULL PRIMARY KEY,
  feed_id integer NOT NULL REFERENCES feeds (id) ON DELETE cascade,
  entry_id integer NOT NULL REFERENCES entries (id) ON DELETE cascade,
  UNIQUE (feed_id, entry_id)
);

CREATE TABLE users (
  id text NOT NULL PRIMARY KEY,
  email text NOT NULL UNIQUE,
  password text NOT NULL,
  created_at text NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at text NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE profiles (
  id text NOT NULL PRIMARY KEY,
  title text NOT NULL,
  image_url text,
  is_default integer NOT NULL DEFAULT 0,
  user_id text NOT NULL REFERENCES users (id) ON DELETE cascade,
  created_at text NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at text NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE index profiles_user_id_is_default_unq ON profiles (user_id, is_default)
WHERE
  is_default;

CREATE TABLE profile_feeds (
  id text NOT NULL PRIMARY KEY,
  custom_title text,
  profile_id text NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  feed_id integer NOT NULL REFERENCES feeds (id) ON DELETE restrict,
  created_at text NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at text NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (profile_id, feed_id)
);

CREATE TABLE profile_feed_entries (
  id text NOT NULL PRIMARY KEY,
  has_read integer NOT NULL DEFAULT 0,
  profile_feed_id text NOT NULL REFERENCES profile_feeds (id) ON DELETE cascade,
  feed_entry_id integer NOT NULL REFERENCES feed_entries (id) ON DELETE restrict,
  UNIQUE (profile_feed_id, feed_entry_id)
);

CREATE TABLE collections (
  id text NOT NULL PRIMARY KEY,
  title text NOT NULL,
  is_default integer NOT NULL DEFAULT 0,
  profile_id text NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  created_at text NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at text NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE index collections_profile_id_is_default_unq ON collections (profile_id, is_default)
WHERE
  is_default;

CREATE TABLE bookmarks (
  id text NOT NULL PRIMARY KEY,
  link text NOT NULL,
  title text NOT NULL,
  thumbnail_url text,
  published_at text,
  author text,
  custom_title text,
  custom_thumbnail_url text,
  custom_published_at text,
  custom_author text,
  collection_id text NOT NULL REFERENCES collections (id) ON DELETE cascade,
  created_at text NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at text NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (collection_id, link)
);

CREATE TABLE tags (
  id text NOT NULL PRIMARY KEY,
  title text NOT NULL,
  profile_id text NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  created_at text NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at text NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER users_updated_at
AFTER
UPDATE ON users FOR each ROW
BEGIN
UPDATE users
SET
  updated_at = CURRENT_TIMESTAMP
WHERE
  id = new.id;

END;

CREATE TRIGGER profiles_updated_at
AFTER
UPDATE ON profiles FOR each ROW
BEGIN
UPDATE profiles
SET
  updated_at = CURRENT_TIMESTAMP
WHERE
  id = new.id;

END;

CREATE TRIGGER profile_feeds_updated_at
AFTER
UPDATE ON profile_feeds FOR each ROW
BEGIN
UPDATE profile_feeds
SET
  updated_at = CURRENT_TIMESTAMP
WHERE
  id = new.id;

END;

CREATE TRIGGER collections_updated_at
AFTER
UPDATE ON collections FOR each ROW
BEGIN
UPDATE collections
SET
  updated_at = CURRENT_TIMESTAMP
WHERE
  id = new.id;

END;

CREATE TRIGGER bookmarks_updated_at
AFTER
UPDATE ON bookmarks FOR each ROW
BEGIN
UPDATE bookmarks
SET
  updated_at = CURRENT_TIMESTAMP
WHERE
  id = new.id;

END;

CREATE TRIGGER tags_updated_at
AFTER
UPDATE ON tags FOR each ROW
BEGIN
UPDATE tags
SET
  updated_at = CURRENT_TIMESTAMP
WHERE
  id = new.id;

END;

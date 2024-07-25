CREATE TABLE feeds (
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  link text NOT NULL UNIQUE,
  title text NOT NULL,
  url text
);

CREATE TABLE entries (
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  link text NOT NULL UNIQUE,
  title text NOT NULL,
  published_at timestamptz,
  description text,
  author text,
  thumbnail_url text
);

CREATE TABLE feed_entries (
  id bigint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  feed_id bigint NOT NULL REFERENCES feeds (id) ON DELETE cascade,
  entry_id bigint NOT NULL REFERENCES entries (id) ON DELETE cascade,
  UNIQUE (feed_id, entry_id)
);

CREATE TABLE users (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  email text NOT NULL UNIQUE,
  password text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE profiles (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  title text NOT NULL,
  image_url text,
  is_default boolean NOT NULL DEFAULT FALSE,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE cascade,
  created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE index ON profiles (user_id, is_default)
WHERE
  is_default;

CREATE TABLE profile_feeds (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  custom_title text,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  feed_id bigint NOT NULL REFERENCES feeds (id) ON DELETE restrict,
  created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (profile_id, feed_id)
);

CREATE TABLE profile_feed_entries (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  has_read boolean NOT NULL DEFAULT FALSE,
  profile_feed_id UUID NOT NULL REFERENCES profile_feeds (id) ON DELETE cascade,
  feed_entry_id bigint NOT NULL REFERENCES feed_entries (id) ON DELETE restrict,
  UNIQUE (profile_feed_id, feed_entry_id)
);

CREATE TABLE collections (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  title text NOT NULL,
  is_default boolean NOT NULL DEFAULT FALSE,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE index ON collections (profile_id, is_default)
WHERE
  is_default;

CREATE TABLE bookmarks (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  link text NOT NULL,
  title text NOT NULL,
  thumbnail_url text,
  published_at timestamptz,
  author text,
  custom_title text,
  custom_thumbnail_url text,
  custom_published_at timestamptz,
  custom_author text,
  collection_id UUID NOT NULL REFERENCES collections (id) ON DELETE cascade,
  created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (collection_id, link)
);

CREATE TABLE tags (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  title text NOT NULL,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE profile_feed_tags (
  profile_feed_id UUID NOT NULL REFERENCES profile_feeds (id) ON DELETE cascade,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE cascade,
  PRIMARY KEY (profile_feed_id, tag_id)
);

CREATE TABLE bookmark_tags (
  bookmark_id UUID NOT NULL REFERENCES bookmarks (id) ON DELETE cascade,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE cascade,
  PRIMARY KEY (bookmark_id, tag_id)
);

CREATE
OR REPLACE function handle_updated_at () returns trigger AS $$
BEGIN
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$ language plpgsql;

CREATE TRIGGER users_updated_at before
UPDATE ON users FOR each ROW
EXECUTE procedure handle_updated_at ();

CREATE TRIGGER profiles_updated_at before
UPDATE ON profiles FOR each ROW
EXECUTE procedure handle_updated_at ();

CREATE TRIGGER profile_feeds_updated_at before
UPDATE ON profile_feeds FOR each ROW
EXECUTE procedure handle_updated_at ();

CREATE TRIGGER collections_updated_at before
UPDATE ON collections FOR each ROW
EXECUTE procedure handle_updated_at ();

CREATE TRIGGER bookmarks_updated_at before
UPDATE ON bookmarks FOR each ROW
EXECUTE procedure handle_updated_at ();

CREATE TRIGGER tags_updated_at before
UPDATE ON tags FOR each ROW
EXECUTE procedure handle_updated_at ();
CREATE TABLE IF NOT EXISTS feeds (
  id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  link text NOT NULL UNIQUE,
  title text NOT NULL,
  url text
);

CREATE TABLE IF NOT EXISTS entries (
  id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  link text NOT NULL UNIQUE,
  title text NOT NULL,
  published_at timestamptz,
  description text,
  author text,
  thumbnail_url text
);

CREATE TABLE IF NOT EXISTS feed_entries (
  id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  feed_id int NOT NULL REFERENCES feeds (id) ON DELETE cascade,
  entry_id int NOT NULL REFERENCES entries (id) ON DELETE cascade,
  UNIQUE (feed_id, entry_id)
);
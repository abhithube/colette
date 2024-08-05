CREATE TABLE IF NOT EXISTS bookmarks (
  id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  link text NOT NULL UNIQUE,
  title text NOT NULL,
  thumbnail_url text,
  published_at timestamptz,
  author text
);
CREATE TABLE IF NOT EXISTS bookmarks (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  link text NOT NULL,
  title text NOT NULL,
  thumbnail_url text,
  published_at timestamptz,
  author text,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  UNIQUE (profile_id, link)
);
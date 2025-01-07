CREATE TABLE bookmarks (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  link TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  thumbnail_url TEXT,
  published_at TIMESTAMPTZ,
  author TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
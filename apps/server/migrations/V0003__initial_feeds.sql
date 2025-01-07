CREATE TABLE feeds (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  link TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  url TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE feed_entries (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  link TEXT NOT NULL UNIQUE,
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
CREATE TABLE smart_feeds (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT NOT NULL,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (profile_id, title)
);

CREATE TYPE field AS ENUM (
  'link',
  'title',
  'published_at',
  'description',
  'author',
  'has_read'
);

CREATE TYPE operation AS ENUM (
  '=',
  '!=',
  'LIKE',
  'NOT LIKE',
  '>',
  '<',
  'in_last_x_sec'
);

CREATE TABLE smart_feed_filters (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  field FIELD NOT NULL,
  operation OPERATION NOT NULL,
  value TEXT NOT NULL,
  smart_feed_id UUID NOT NULL REFERENCES smart_feeds (id) ON DELETE CASCADE,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
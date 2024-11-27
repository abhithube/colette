CREATE TABLE profile_feeds (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT,
  pinned BOOLEAN NOT NULL DEFAULT FALSE,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  feed_id INTEGER NOT NULL REFERENCES feeds (id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (profile_id, feed_id)
);

CREATE TABLE profile_feed_entries (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  has_read BOOLEAN NOT NULL DEFAULT FALSE,
  profile_feed_id UUID NOT NULL REFERENCES profile_feeds (id) ON DELETE CASCADE,
  feed_entry_id INTEGER NOT NULL REFERENCES feed_entries (id) ON DELETE RESTRICT,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (profile_feed_id, feed_entry_id)
);
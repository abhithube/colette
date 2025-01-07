CREATE TABLE user_feeds (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT,
  folder_id UUID REFERENCES folders (id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  feed_id UUID NOT NULL REFERENCES feeds (id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, feed_id)
);

CREATE TABLE user_feed_entries (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  has_read BOOLEAN NOT NULL DEFAULT FALSE,
  user_feed_id UUID NOT NULL REFERENCES user_feeds (id) ON DELETE CASCADE,
  feed_entry_id UUID NOT NULL REFERENCES feed_entries (id) ON DELETE RESTRICT,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_feed_id, feed_entry_id)
);
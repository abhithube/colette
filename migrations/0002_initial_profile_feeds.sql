CREATE TABLE IF NOT EXISTS profile_feeds (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  feed_id int NOT NULL REFERENCES feeds (id) ON DELETE restrict,
  UNIQUE (profile_id, feed_id)
);

CREATE TABLE IF NOT EXISTS profile_feed_entries (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  has_read boolean NOT NULL DEFAULT FALSE,
  profile_feed_id UUID NOT NULL REFERENCES profile_feeds (id) ON DELETE cascade,
  feed_entry_id int NOT NULL REFERENCES feed_entries (id) ON DELETE restrict,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  UNIQUE (profile_feed_id, feed_entry_id)
);
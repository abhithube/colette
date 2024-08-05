CREATE TABLE IF NOT EXISTS tags (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  title text NOT NULL,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  UNIQUE (profile_id, title)
);

CREATE TABLE IF NOT EXISTS profile_feed_tags (
  profile_feed_id UUID NOT NULL REFERENCES profile_feeds (id) ON DELETE cascade,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE cascade,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  PRIMARY KEY (profile_feed_id, tag_id)
);

CREATE TABLE IF NOT EXISTS profile_bookmark_tags (
  profile_bookmark_id UUID NOT NULL REFERENCES profile_bookmarks (id) ON DELETE cascade,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE cascade,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  PRIMARY KEY (profile_bookmark_id, tag_id)
);
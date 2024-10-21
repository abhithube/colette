CREATE TABLE tags (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT NOT NULL,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (profile_id, title)
);

CREATE TABLE profile_feed_tags (
  profile_feed_id UUID NOT NULL REFERENCES profile_feeds (id) ON DELETE CASCADE,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (profile_feed_id, tag_id)
);

CREATE TABLE profile_bookmark_tags (
  profile_bookmark_id UUID NOT NULL REFERENCES profile_bookmarks (id) ON DELETE CASCADE,
  tag_id UUID NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (profile_bookmark_id, tag_id)
);
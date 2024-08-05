CREATE TABLE IF NOT EXISTS profile_bookmarks (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE cascade,
  bookmark_id int NOT NULL REFERENCES bookmarks (id) ON DELETE restrict,
  UNIQUE (profile_id, bookmark_id)
);
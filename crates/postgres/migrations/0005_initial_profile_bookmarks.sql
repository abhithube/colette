CREATE TABLE profile_bookmarks (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  profile_id UUID NOT NULL REFERENCES profiles (id) ON DELETE CASCADE,
  bookmark_id INTEGER NOT NULL REFERENCES bookmarks (id) ON DELETE RESTRICT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (profile_id, bookmark_id)
);
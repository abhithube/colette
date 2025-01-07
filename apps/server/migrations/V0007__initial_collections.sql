CREATE TABLE collections (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT NOT NULL,
  folder_id UUID REFERENCES folders (id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE user_bookmarks (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  bookmark_id UUID NOT NULL REFERENCES bookmarks (id) ON DELETE RESTRICT,
  collection_id UUID REFERENCES collections (id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (user_id, bookmark_id)
);

CREATE UNIQUE INDEX user_bookmarks_collection_id_bookmark_id_key ON user_bookmarks (collection_id, bookmark_id) WHERE collection_id IS NOT NULL;
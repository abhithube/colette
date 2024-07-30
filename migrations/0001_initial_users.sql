CREATE TABLE IF NOT EXISTS users (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  email text NOT NULL UNIQUE,
  password text NOT NULL
);

CREATE TABLE IF NOT EXISTS profiles (
  id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid (),
  title text NOT NULL,
  image_url text,
  is_default boolean NOT NULL DEFAULT FALSE,
  user_id UUID NOT NULL REFERENCES users (id) ON DELETE cascade
);

CREATE UNIQUE index if NOT EXISTS profiles_user_id_is_default_idx ON profiles (user_id, is_default)
WHERE
  is_default;
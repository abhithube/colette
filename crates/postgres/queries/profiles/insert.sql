INSERT INTO
  profiles (id, title, image_url, is_default, user_id)
VALUES
  ($1, $2, $3, $4, $5)
RETURNING
  id,
  title,
  image_url,
  created_at,
  updated_at;

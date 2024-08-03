INSERT INTO
  profiles (title, image_url, is_default, user_id)
VALUES
  ($1, $2, $3, $4)
RETURNING
  id,
  title,
  image_url,
  user_id;
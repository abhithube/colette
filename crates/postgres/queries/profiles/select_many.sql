SELECT
  id,
  title,
  image_url,
  created_at,
  updated_at
FROM
  profiles
WHERE
  user_id = $1;
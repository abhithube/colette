SELECT
  id,
  title,
  image_url,
  user_id,
  created_at,
  updated_at
FROM
  profiles
WHERE
  id = $1
  AND user_id = $2;

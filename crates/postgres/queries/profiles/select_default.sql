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
  user_id = $1
  AND is_default = TRUE;

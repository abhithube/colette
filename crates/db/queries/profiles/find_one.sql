SELECT
  id,
  title,
  image_url,
  is_default,
  user_id
FROM
  profiles
WHERE
  id = $1
  AND user_id = $2;
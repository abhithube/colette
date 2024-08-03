SELECT
  id,
  title,
  image_url,
  user_id
FROM
  profiles
WHERE
  id = $1
  AND user_id = $2;
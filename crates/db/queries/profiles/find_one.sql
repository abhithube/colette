SELECT
  id,
  title,
  image_url,
  is_default,
  user_id
FROM
  profile
WHERE
  id = $1
  AND user_id = $2;
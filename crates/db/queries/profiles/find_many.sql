SELECT
  id,
  title,
  image_url,
  is_default,
  user_id
FROM
  profiles
WHERE
  user_id = $1
ORDER BY
  title ASC;
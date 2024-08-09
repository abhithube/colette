SELECT
  id,
  title,
  image_url,
  is_default,
  user_id
FROM
  profile
WHERE
  user_id = $1
ORDER BY
  title ASC;
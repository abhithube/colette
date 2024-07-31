SELECT
  id,
  title,
  image_url,
  user_id
FROM
  profiles
WHERE
  user_id = $1;
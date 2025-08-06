SELECT
  id,
  email,
  display_name,
  image_url AS "image_url: DbUrl",
  created_at,
  updated_at
FROM
  users
WHERE
  id = $1

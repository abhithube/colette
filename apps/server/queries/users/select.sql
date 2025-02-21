SELECT
  id,
  email,
  display_name,
  created_at,
  updated_at
FROM
  users
WHERE
  id = $1

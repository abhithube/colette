SELECT
  id,
  title,
  user_id,
  created_at,
  updated_at
FROM
  tags
WHERE
  id = $1
  AND user_id = $2
